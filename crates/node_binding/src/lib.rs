#![recursion_limit = "256"]
#![feature(let_chains)]
#![feature(try_blocks)]
#[macro_use]
extern crate napi_derive;

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

use napi::bindgen_prelude::*;
use once_cell::sync::Lazy;
use rspack_binding_options::BuiltinPlugin;
use rspack_binding_values::SingleThreadedHashMap;
use rspack_core::PluginExt;
use rspack_error::Diagnostic;
use rspack_fs_node::{AsyncNodeWritableFileSystem, ThreadsafeNodeFS};

mod hook;
mod loader;
mod panic;
mod plugins;

use hook::*;
// Napi macro registered this successfully
#[allow(unused)]
use loader::run_builtin_loader;
use plugins::*;
use rspack_binding_options::*;
use rspack_binding_values::*;
use rspack_napi_shared::set_napi_env;
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

static COMPILERS: Lazy<
  SingleThreadedHashMap<CompilerId, Pin<Box<rspack_core::Compiler<AsyncNodeWritableFileSystem>>>>,
> = Lazy::new(Default::default);

static NEXT_COMPILER_ID: AtomicU32 = AtomicU32::new(0);

type CompilerId = u32;

#[napi(custom_finalize)]
pub struct Rspack {
  id: CompilerId,
  js_plugin: JsHooksAdapterPlugin,
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
    js_loader_runner: JsFunction,
  ) -> Result<Self> {
    Self::prepare_environment(&env);
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

    let js_loader_runner: JsLoaderRunner = JsLoaderRunner::try_from(js_loader_runner)?;
    plugins.push(JsLoaderResolver { js_loader_runner }.boxed());

    let compiler_options = options
      .apply(&mut plugins)
      .map_err(|e| Error::from_reason(format!("{e}")))?;

    tracing::info!("normalized_options: {:#?}", &compiler_options);

    let rspack = rspack_core::Compiler::new(
      compiler_options,
      plugins,
      AsyncNodeWritableFileSystem::new(env, output_filesystem)
        .map_err(|e| Error::from_reason(format!("Failed to create writable filesystem: {e}",)))?,
    );

    let id = NEXT_COMPILER_ID.fetch_add(1, Ordering::SeqCst);
    unsafe { COMPILERS.insert_if_vacant(id, Box::pin(rspack)) }?;

    Ok(Self { id, js_plugin })
  }

  #[allow(clippy::unwrap_in_result, clippy::unwrap_used)]
  #[napi(
    js_name = "unsafe_set_disabled_hooks",
    ts_args_type = "hooks: Array<string>"
  )]
  pub fn set_disabled_hooks(&self, _env: Env, hooks: Vec<String>) -> Result<()> {
    self.js_plugin.set_disabled_hooks(hooks)
  }

  /// Build with the given option passed to the constructor
  ///
  /// Warning:
  /// Calling this method recursively might cause a deadlock.
  #[napi(
    js_name = "unsafe_build",
    ts_args_type = "callback: (err: null | Error) => void"
  )]
  pub fn build(&self, env: Env, f: JsFunction) -> Result<()> {
    let handle_build = |compiler: &mut Pin<Box<rspack_core::Compiler<_>>>| {
      // Safety: compiler is stored in a global hashmap, so it's guaranteed to be alive.
      let compiler: &'static mut Pin<Box<rspack_core::Compiler<AsyncNodeWritableFileSystem>>> =
        unsafe { std::mem::transmute::<&'_ mut _, &'static mut _>(compiler) };

      callbackify(env, f, async move {
        compiler.build().await.map_err(|e| {
          Error::new(
            napi::Status::GenericFailure,
            print_error_diagnostic(e, compiler.options.stats.colors),
          )
        })?;
        tracing::info!("build ok");
        Ok(())
      })
    };
    unsafe { COMPILERS.borrow_mut(&self.id, handle_build) }
  }

  /// Rebuild with the given option passed to the constructor
  ///
  /// Warning:
  /// Calling this method recursively will cause a deadlock.
  #[napi(
    js_name = "unsafe_rebuild",
    ts_args_type = "changed_files: string[], removed_files: string[], callback: (err: null | Error) => void"
  )]
  pub fn rebuild(
    &self,
    env: Env,
    changed_files: Vec<String>,
    removed_files: Vec<String>,
    f: JsFunction,
  ) -> Result<()> {
    let handle_rebuild = |compiler: &mut Pin<Box<rspack_core::Compiler<_>>>| {
      // Safety: compiler is stored in a global hashmap, so it's guaranteed to be alive.
      // The reason why use Box<Compiler> here instead of Compiler itself is that:
      // Compilers may expand and change its layout underneath, make Compiler layout change.
      // Use Box to make sure the Compiler layout won't change
      let compiler: &'static mut Pin<Box<rspack_core::Compiler<AsyncNodeWritableFileSystem>>> =
        unsafe { std::mem::transmute::<&'_ mut _, &'static mut _>(compiler) };

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
        Ok(())
      })
    };

    unsafe { COMPILERS.borrow_mut(&self.id, handle_rebuild) }
  }

  /// Get the last compilation
  ///
  /// Warning:
  ///
  /// Calling this method under the build or rebuild method might cause a deadlock.
  ///
  /// **Note** that this method is not safe if you cache the _JsCompilation_ on the Node side, as it will be invalidated by the next build and accessing a dangling ptr is a UB.
  #[napi(js_name = "unsafe_last_compilation")]
  pub fn unsafe_last_compilation<F: Fn(JsCompilation) -> Result<()>>(&self, f: F) -> Result<()> {
    let handle_last_compilation = |compiler: &mut Pin<Box<rspack_core::Compiler<_>>>| {
      // Safety: compiler is stored in a global hashmap, and compilation is only available in the callback of this function, so it is safe to cast to a static lifetime. See more in the warning part of this method.
      // The reason why use Box<Compiler> here instead of Compiler itself is that:
      // Compilers may expand and change its layout underneath, make Compiler layout change.
      // Use Box to make sure the Compiler layout won't change
      let compiler: &'static mut Pin<Box<rspack_core::Compiler<AsyncNodeWritableFileSystem>>> =
        unsafe { std::mem::transmute::<&'_ mut _, &'static mut _>(compiler) };
      f(JsCompilation::from_compilation(&mut compiler.compilation))
    };

    unsafe { COMPILERS.borrow_mut(&self.id, handle_last_compilation) }
  }

  /// Destroy the compiler
  ///
  /// Warning:
  ///
  /// Anything related to this compiler will be invalidated after this method is called.
  #[napi(js_name = "unsafe_drop")]
  pub fn drop(&self) -> Result<()> {
    unsafe { COMPILERS.remove(&self.id) };

    Ok(())
  }
}

impl ObjectFinalize for Rspack {
  fn finalize(self, _env: Env) -> Result<()> {
    // WARNING: Don't try to destroy the compiler from the finalize method. The background thread may still be working and it's a COMPLETELY unsafe way.
    Ok(())
  }
}

impl Rspack {
  fn prepare_environment(env: &Env) {
    set_napi_env(env.raw());
  }
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
