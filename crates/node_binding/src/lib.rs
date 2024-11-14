#![recursion_limit = "256"]
#![feature(let_chains)]
#![feature(try_blocks)]
#[macro_use]
extern crate napi_derive;
extern crate rspack_allocator;

use std::sync::Mutex;
use std::{pin::Pin, str::FromStr as _};

use compiler::{Compiler, CompilerState, CompilerStateGuard};
use napi::bindgen_prelude::*;
use rspack_binding_options::BuiltinPlugin;
use rspack_core::{Compilation, PluginExt};
use rspack_error::Diagnostic;
use rspack_fs_node::{AsyncNodeWritableFileSystem, ThreadsafeNodeFS};
use rspack_napi::napi::bindgen_prelude::within_runtime_if_available;

mod compiler;
mod diagnostic;
mod panic;
mod plugins;
mod resolver_factory;

pub use diagnostic::*;
use plugins::*;
use resolver_factory::*;
use rspack_binding_options::*;
use rspack_binding_values::*;
use rspack_tracing::{ChromeTracer, OtelTracer, StdoutTracer, TokioConsoleTracer, Tracer};
use tracing::Level;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt as _, Layer, Registry};

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
    register_js_taps: RegisterJsTaps,
    output_filesystem: ThreadsafeNodeFS,
    mut resolver_factory_reference: Reference<JsResolverFactory>,
  ) -> Result<Self> {
    tracing::info!("raw_options: {:#?}", &options);

    let mut plugins = Vec::new();
    let js_plugin = JsHooksAdapterPlugin::from_js_hooks(env, register_js_taps)?;
    plugins.push(js_plugin.clone().boxed());
    for bp in builtin_plugins {
      bp.append_to(env, &mut plugins)
        .map_err(|e| Error::from_reason(format!("{e}")))?;
    }

    let compiler_options: rspack_core::CompilerOptions = options
      .try_into()
      .map_err(|e| Error::from_reason(format!("{e}")))?;

    tracing::info!("normalized_options: {:#?}", &compiler_options);

    let resolver_factory =
      (*resolver_factory_reference).get_resolver_factory(compiler_options.resolve.clone());
    let loader_resolver_factory = (*resolver_factory_reference)
      .get_loader_resolver_factory(compiler_options.resolve_loader.clone());
    let rspack = rspack_core::Compiler::new(
      compiler_options,
      plugins,
      rspack_binding_options::buildtime_plugins::buildtime_plugins(),
      Some(Box::new(
        AsyncNodeWritableFileSystem::new(output_filesystem)
          .map_err(|e| Error::from_reason(format!("Failed to create writable filesystem: {e}",)))?,
      )),
      None,
      Some(resolver_factory),
      Some(loader_resolver_factory),
    );

    Ok(Self {
      compiler: Box::pin(Compiler::from(rspack)),
      state: CompilerState::init(),
      js_plugin,
    })
  }

  #[napi]
  pub fn set_non_skippable_registers(&self, kinds: Vec<RegisterJsTapKind>) {
    self.js_plugin.set_non_skippable_registers(kinds)
  }

  /// Build with the given option passed to the constructor
  #[napi(ts_args_type = "callback: (err: null | Error) => void")]
  pub fn build(&mut self, env: Env, reference: Reference<Rspack>, f: Function) -> Result<()> {
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
    f: Function,
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

    self.cleanup_last_compilation(&compiler.compilation);

    // SAFETY:
    // 1. `Compiler` is pinned and stored on the heap.
    // 2. `JsReference` (NAPI internal mechanism) keeps `Compiler` alive until its instance getting garbage collected.
    f(
      unsafe { std::mem::transmute::<&mut Compiler, &'static mut Compiler>(*compiler) },
      _guard,
    )
  }

  fn cleanup_last_compilation(&self, compilation: &Compilation) {
    JsCompilationWrapper::cleanup_last_compilation(compilation.id());
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
  On(Box<dyn Tracer>),
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

thread_local! {
  static GLOBAL_TRACE_STATE: Mutex<TraceState> = const { Mutex::new(TraceState::Off) };
}

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
  #[napi(ts_arg_type = "\"chrome\" | \"logger\"| \"console\" | \"otel\"")] layer: String,
  output: String,
) -> anyhow::Result<()> {
  GLOBAL_TRACE_STATE.with(|state| {
    let mut state = state.lock().expect("Failed to lock GLOBAL_TRACE_STATE");
    if let TraceState::Off = *state {
      let mut tracer: Box<dyn Tracer> = match layer.as_str() {
        "chrome" => Box::new(ChromeTracer::default()),
        "otel" => Box::new(within_runtime_if_available(OtelTracer::default)),
        "console" => Box::new(TokioConsoleTracer),
        "logger" => Box::new(StdoutTracer),
        _ => anyhow::bail!(
          "Unexpected layer: {}, supported layers: 'chrome', 'logger', 'console' and 'otel' ",
          layer
        ),
      };
      if let Some(layer) = tracer.setup(&output) {
        if let Ok(default_level) = Level::from_str(&filter) {
          let filter = tracing_subscriber::filter::Targets::new()
            .with_target("rspack_core", default_level)
            .with_target("node_binding", default_level)
            .with_target("rspack_loader_swc", default_level)
            .with_target("rspack_loader_runner", default_level)
            .with_target("rspack_plugin_javascript", default_level)
            .with_target("rspack_resolver", Level::WARN);
          tracing_subscriber::registry()
            .with(<_ as Layer<Registry>>::with_filter(layer, filter))
            .init();
        } else {
          // SAFETY: we know that trace_var is `Ok(String)` now,
          // for the second unwrap, if we can't parse the directive, then the tracing result would be
          // unexpected, then panic is reasonable
          let filter = EnvFilter::builder()
            .with_regex(true)
            .parse(filter)
            .expect("Parse tracing directive syntax failed, for details about the directive syntax you could refer https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives");
          tracing_subscriber::registry()
            .with(<_ as Layer<Registry>>::with_filter(layer, filter))
            .init();
        }
      }
      let new_state = TraceState::On(tracer);
      *state = new_state;
    }
    Ok(())
  })
}

#[napi]
pub fn cleanup_global_trace() {
  GLOBAL_TRACE_STATE.with(|state| {
    let mut state = state.lock().expect("Failed to lock GLOBAL_TRACE_STATE");
    if let TraceState::On(ref mut tracer) = *state {
      tracer.teardown();
    }
    *state = TraceState::Off;
  });
}
