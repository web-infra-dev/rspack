#![recursion_limit = "256"]
#![feature(let_chains)]
#![feature(try_blocks)]
#[macro_use]
extern crate napi_derive;
// extern crate rspack_allocator;

use std::sync::Mutex;
use std::{mem::ManuallyDrop, pin::Pin};

use compiler::Compiler;
use napi::bindgen_prelude::*;
use rspack_binding_options::BuiltinPlugin;
use rspack_core::PluginExt;
use rspack_error::Diagnostic;
use rspack_fs_node::{AsyncNodeWritableFileSystem, ThreadsafeNodeFS};

mod compiler;
mod hook;
mod loader;
mod panic;
mod plugins;

use hook::*;
pub use loader::run_builtin_loader;
use plugins::*;
use rspack_binding_options::*;
use rspack_binding_values::*;
use rspack_tracing::chrome::FlushGuard;

#[cfg(not(target_os = "linux"))]
#[global_allocator]
static GLOBAL: mimalloc_rust::GlobalMiMalloc = mimalloc_rust::GlobalMiMalloc;

#[cfg(all(
  target_os = "linux",
  target_env = "gnu",
  any(target_arch = "x86_64", target_arch = "aarch64")
))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[napi(custom_finalize)]
pub struct Rspack {
  js_plugin: JsHooksAdapterPlugin,
  compiler: ManuallyDrop<Pin<Box<Compiler>>>,
  running: bool,
}

impl ObjectFinalize for Rspack {
  fn finalize(self, _env: Env) -> Result<()> {
    Ok(())
  }
}

#[napi]
impl Rspack {
  #[napi(
    constructor,
    ts_args_type = "options: RawOptions, builtinPlugins: Array<BuiltinPlugin>, jsHooks: JsHooks, registerJsTaps: RegisterJsTaps, outputFilesystem: ThreadsafeNodeFS, jsLoaderRunner: (ctx: JsLoaderContext) => Promise<JsLoaderResult | void>"
  )]
  pub fn new(
    env: Env,
    options: RawOptions,
    builtin_plugins: Vec<BuiltinPlugin>,
    js_hooks: JsHooks,
    register_js_taps: RegisterJsTaps,
    output_filesystem: ThreadsafeNodeFS,
    js_loader_runner: JsLoaderRunner,
  ) -> Result<Self> {
    tracing::info!("raw_options: {:#?}", &options);

    let disabled_hooks: DisabledHooks = Default::default();
    let mut plugins = Vec::new();
    let js_plugin =
      JsHooksAdapterPlugin::from_js_hooks(env, js_hooks, disabled_hooks, register_js_taps)?;
    plugins.push(js_plugin.clone().boxed());
    for bp in builtin_plugins {
      bp.append_to(&mut plugins)
        .map_err(|e| Error::from_reason(format!("{e}")))?;
    }
    plugins.push(JsLoaderResolver { js_loader_runner }.boxed());

    let compiler_options = options
      .apply(&mut plugins)
      .map_err(|e| Error::from_reason(format!("{e}")))?;

    tracing::info!("normalized_options: {:#?}", &compiler_options);

    let rspack = rspack_core::Compiler::new(
      compiler_options,
      plugins,
      AsyncNodeWritableFileSystem::new(output_filesystem)
        .map_err(|e| Error::from_reason(format!("Failed to create writable filesystem: {e}",)))?,
    );

    Ok(Self {
      compiler: ManuallyDrop::new(Box::pin(Compiler::from(rspack))),
      running: false,
      js_plugin,
    })
  }

  #[napi(ts_args_type = "hooks: Array<string>")]
  pub fn set_disabled_hooks(&self, _env: Env, hooks: Vec<String>) -> Result<()> {
    self.js_plugin.set_disabled_hooks(hooks)
  }

  /// Build with the given option passed to the constructor
  #[napi(ts_args_type = "callback: (err: null | Error) => void")]
  pub fn build(
    &mut self,
    env: Env,
    // reference: Reference<Rspack>,
    f: JsFunction,
  ) -> Result<()> {
    unsafe {
      self.run(
        // env, reference,
        |compiler| {
          callbackify(env, f, async {
            compiler.build().await.map_err(|e| {
              Error::new(
                napi::Status::GenericFailure,
                print_error_diagnostic(e, compiler.options.stats.colors),
              )
            })?;
            tracing::info!("build ok");
            Ok(())
          })
        },
      )
    }
  }

  /// Rebuild with the given option passed to the constructor
  #[napi(
    ts_args_type = "changed_files: string[], removed_files: string[], callback: (err: null | Error) => void"
  )]
  pub fn rebuild(
    &mut self,
    env: Env,
    // reference: Reference<Rspack>,
    changed_files: Vec<String>,
    removed_files: Vec<String>,
    f: JsFunction,
  ) -> Result<()> {
    use std::collections::HashSet;

    unsafe {
      self.run(
        // env, reference,
        |compiler| {
          callbackify(env, f, async {
            compiler
              .rebuild(
                HashSet::from_iter(changed_files.into_iter()),
                HashSet::from_iter(removed_files.into_iter()),
              )
              .await
              .map_err(|e| {
                Error::new(
                  napi::Status::GenericFailure,
                  print_error_diagnostic(e, compiler.options.stats.colors),
                )
              })?;
            tracing::info!("rebuild ok");
            Ok(())
          })
        },
      )
    }
  }
}

impl Rspack {
  /// Run the given function with the compiler.
  ///
  /// ## Safety
  /// 1. The caller must ensure that the `Compiler` is not moved or dropped during the lifetime of the function.
  /// 2. The mutable reference to `self.running` should never been exposed to the callback.
  ///    Otherwise, this would lead to potential race condition for `Compiler`,
  ///    especially when `build` or `rebuild` was called on JS side and its previous `build` or `rebuild` was yet to finish.
  unsafe fn run<R>(
    &mut self,
    // env: Env,
    // reference: Reference<Rspack>,
    f: impl FnOnce(&'static mut Compiler) -> Result<R>,
  ) -> Result<R> {
    if self.running {
      return Err(concurrent_compiler_error());
    }
    self.running = true;

    // let mut compiler = reference.share_with(env, |s| {
    //   // SAFETY: The mutable reference to `Compiler` is exclusive. It's guaranteed by the running state guard.
    //   Ok(unsafe { s.compiler.as_mut().get_unchecked_mut() })
    // })?;
    let compiler = &mut self.compiler.as_mut().get_unchecked_mut();
    // SAFETY: `Compiler` will not be moved, as it's stored on the heap.
    // `Compiler` is valid through the lifetime before it's closed by calling `Compiler.close()` or gc-ed.
    let result =
      f(unsafe { std::mem::transmute::<&mut Compiler, &'static mut Compiler>(*compiler) });

    self.running = false;
    result
  }
}

fn concurrent_compiler_error() -> Error {
  Error::new(
    napi::Status::GenericFailure,
    "ConcurrentCompilationError: You ran rspack twice. Each instance only supports a single concurrent compilation at a time.",
  )
}

#[derive(Default)]
enum TraceState {
  On(Option<FlushGuard>),
  #[default]
  Off,
}

#[ctor]
fn init() {
  panic::install_panic_handler();
}

fn print_error_diagnostic(e: rspack_error::Error, colored: bool) -> String {
  Diagnostic::from(e)
    .render_report(colored)
    .expect("should print diagnostics")
}

static GLOBAL_TRACE_STATE: Mutex<TraceState> = Mutex::new(TraceState::Off);

/**
 * Some code is modified based on
 * https://github.com/swc-project/swc/blob/d1d0607158ab40463d1b123fed52cc526eba8385/bindings/binding_core_node/src/util.rs#L29-L58
 * Apache-2.0 licensed
 * Author Donny/강동윤
 * Copyright (c)
 */
#[napi]
pub fn register_global_trace(
  filter: String,
  #[napi(ts_arg_type = "\"chrome\" | \"logger\"")] layer: String,
  output: String,
) {
  let mut state = GLOBAL_TRACE_STATE
    .lock()
    .expect("Failed to lock GLOBAL_TRACE_STATE");
  if matches!(&*state, TraceState::Off) {
    let guard = match layer.as_str() {
      "chrome" => rspack_tracing::enable_tracing_by_env_with_chrome_layer(&filter, &output),
      "logger" => {
        rspack_tracing::enable_tracing_by_env(&filter, &output);
        None
      }
      _ => panic!("not supported layer type:{layer}"),
    };
    let new_state = TraceState::On(guard);
    *state = new_state;
  }
}

#[napi]
pub fn cleanup_global_trace() {
  let mut state = GLOBAL_TRACE_STATE
    .lock()
    .expect("Failed to lock GLOBAL_TRACE_STATE");
  if let TraceState::On(guard) = &mut *state
    && let Some(g) = guard.take()
  {
    g.flush();
    drop(g);
    let new_state = TraceState::Off;
    *state = new_state;
  }
}
