#![recursion_limit = "256"]
#![feature(let_chains)]
#![feature(try_blocks)]
#[macro_use]
extern crate napi_derive;
extern crate rspack_allocator;

use std::pin::Pin;
use std::sync::Mutex;

use compiler::{Compiler, CompilerState, CompilerStateGuard};
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

#[napi]
pub struct Rspack {
  js_plugin: JsHooksAdapterPlugin,
  compiler: Pin<Box<Compiler>>,
  state: CompilerState,
}

#[napi]
impl Rspack {
  #[napi(constructor)]
  pub fn new(
    env: Env,
    options: RawOptions,
    builtin_plugins: Vec<BuiltinPlugin>,
    js_hooks: JsHooks,
    register_js_taps: RegisterJsTaps,
    output_filesystem: ThreadsafeNodeFS,
  ) -> Result<Self> {
    tracing::info!("raw_options: {:#?}", &options);

    let mut plugins = Vec::new();
    let js_plugin = JsHooksAdapterPlugin::from_js_hooks(env, js_hooks, register_js_taps)?;
    plugins.push(js_plugin.clone().boxed());
    for bp in builtin_plugins {
      bp.append_to(&mut plugins)
        .map_err(|e| Error::from_reason(format!("{e}")))?;
    }

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
      compiler: Box::pin(Compiler::from(rspack)),
      state: CompilerState::init(),
      js_plugin,
    })
  }

  #[napi]
  pub fn set_disabled_hooks(&self, hooks: Vec<String>) {
    self.js_plugin.set_disabled_hooks(hooks)
  }

  #[napi]
  pub fn set_non_skippable_registers(&self, kinds: Vec<RegisterJsTapKind>) {
    self.js_plugin.set_non_skippable_registers(kinds)
  }

  /// Build with the given option passed to the constructor
  #[napi(ts_args_type = "callback: (err: null | Error) => void")]
  pub fn build(&mut self, env: Env, reference: Reference<Rspack>, f: JsFunction) -> Result<()> {
    unsafe {
      self.run(env, reference, |compiler, _guard| {
        callbackify(env, f, async move {
          compiler.build().await.map_err(|e| {
            Error::new(
              napi::Status::GenericFailure,
              print_error_diagnostic(e, compiler.options.stats.colors),
            )
          })?;
          tracing::info!("build ok");
          drop(_guard);
          Ok(())
        })
      })
    }
  }

  /// Rebuild with the given option passed to the constructor
  #[napi(
    ts_args_type = "changed_files: string[], removed_files: string[], callback: (err: null | Error) => void"
  )]
  pub fn rebuild(
    &mut self,
    env: Env,
    reference: Reference<Rspack>,
    changed_files: Vec<String>,
    removed_files: Vec<String>,
    f: JsFunction,
  ) -> Result<()> {
    use std::collections::HashSet;

    unsafe {
      self.run(env, reference, |compiler, _guard| {
        callbackify(env, f, async move {
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
          drop(_guard);
          Ok(())
        })
      })
    }
  }
}

impl Rspack {
  /// Run the given function with the compiler.
  ///
  /// ## Safety
  /// 1. The caller must ensure that the `Compiler` is not moved or dropped during the lifetime of the callback.
  /// 2. `CompilerStateGuard` should and only be dropped so soon as each `Compiler` is free of use.
  ///    Accessing `Compiler` beyond the lifetime of `CompilerStateGuard` would lead to potential race condition.
  unsafe fn run<R>(
    &mut self,
    env: Env,
    reference: Reference<Rspack>,
    f: impl FnOnce(&'static mut Compiler, CompilerStateGuard) -> Result<R>,
  ) -> Result<R> {
    if self.state.running() {
      return Err(concurrent_compiler_error());
    }
    let _guard = self.state.enter();
    let mut compiler = reference.share_with(env, |s| {
      // SAFETY: The mutable reference to `Compiler` is exclusive. It's guaranteed by the running state guard.
      Ok(unsafe { s.compiler.as_mut().get_unchecked_mut() })
    })?;
    // SAFETY:
    // 1. `Compiler` is pinned and stored on the heap.
    // 2. `JsReference` (NAPI internal mechanism) keeps `Compiler` alive until its instance getting garbage collected.
    f(
      unsafe { std::mem::transmute::<&mut Compiler, &'static mut Compiler>(*compiler) },
      _guard,
    )
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
