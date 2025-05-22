#![recursion_limit = "256"]
#![feature(let_chains)]
#![feature(try_blocks)]
#[macro_use]
extern crate napi_derive;
extern crate rspack_allocator;
use std::{cell::RefCell, sync::Arc};

use compiler::{Compiler, CompilerState, CompilerStateGuard};
use napi::{bindgen_prelude::*, CallContext};
use rspack_collections::UkeyMap;
use rspack_core::{
  BoxDependency, Compilation, CompilerId, EntryOptions, ModuleIdentifier, PluginExt,
};
use rspack_error::Diagnostic;
use rspack_fs::IntermediateFileSystem;

use crate::fs_node::{NodeFileSystem, ThreadsafeNodeFS};

mod allocator;
mod asset;
mod asset_condition;
mod async_dependency_block;
mod build_info;
mod chunk;
mod chunk_graph;
mod chunk_group;
mod clean_options;
mod codegen_result;
mod compilation;
mod compiler;
mod context_module_factory;
mod dependencies;
mod dependency;
mod diagnostic;
mod error;
mod exports_info;
mod filename;
mod fs_node;
mod html;
mod identifier;
mod module;
mod module_graph;
mod module_graph_connection;
mod modules;
mod normal_module_factory;
mod options;
mod panic;
mod path_data;
mod plugins;
mod raw_options;
mod resolver;
mod resolver_factory;
mod resource_data;
mod rsdoctor;
mod rstest;
mod runtime;
mod source;
mod stats;
mod swc;
mod utils;

pub use asset::*;
pub use asset_condition::*;
pub use async_dependency_block::*;
pub use build_info::*;
pub use chunk::*;
pub use chunk_graph::*;
pub use chunk_group::*;
pub use clean_options::*;
pub use codegen_result::*;
pub use compilation::*;
pub use context_module_factory::*;
pub use dependencies::*;
pub use dependency::*;
pub use diagnostic::*;
pub use error::*;
pub use exports_info::*;
pub use filename::*;
pub use html::*;
pub use module::*;
pub use module_graph::*;
pub use module_graph_connection::*;
pub use modules::*;
pub use normal_module_factory::*;
pub use options::*;
pub use path_data::*;
pub use plugins::buildtime_plugins;
use plugins::*;
pub use raw_options::*;
pub use resolver::*;
use resolver_factory::*;
pub use resource_data::*;
pub use rsdoctor::*;
use rspack_tracing::{ChromeTracer, StdoutTracer, Tracer};
pub use rstest::*;
pub use runtime::*;
use rustc_hash::FxHashMap;
pub use source::*;
pub use stats::*;
pub use swc::*;
use swc_core::common::util::take::Take;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
  layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer, Registry,
};
pub use utils::*;

thread_local! {
  pub static COMPILER_REFERENCES: RefCell<UkeyMap<CompilerId, WeakReference<JsCompiler>>> = Default::default();
}

#[js_function(1)]
fn cleanup_revoked_modules(ctx: CallContext) -> Result<()> {
  let external = ctx.get::<&mut External<Vec<ModuleIdentifier>>>(0)?;
  let revoked_modules = external.take();
  ModuleObject::cleanup_by_module_identifiers(&revoked_modules);
  Ok(())
}

#[napi(custom_finalize)]
pub struct JsCompiler {
  js_hooks_plugin: JsHooksAdapterPlugin,
  compiler: Compiler,
  state: CompilerState,
  include_dependencies_map: FxHashMap<String, FxHashMap<EntryOptions, BoxDependency>>,
  entry_dependencies_map: FxHashMap<String, FxHashMap<EntryOptions, BoxDependency>>,
}

#[napi]
impl JsCompiler {
  #[allow(clippy::too_many_arguments)]
  #[napi(constructor)]
  pub fn new(
    env: Env,
    mut this: This,
    compiler_path: String,
    options: RawOptions,
    builtin_plugins: Vec<BuiltinPlugin>,
    register_js_taps: RegisterJsTaps,
    output_filesystem: ThreadsafeNodeFS,
    intermediate_filesystem: Option<ThreadsafeNodeFS>,
    mut resolver_factory_reference: Reference<JsResolverFactory>,
  ) -> Result<Self> {
    tracing::debug!("raw_options: {:#?}", &options);

    let mut plugins = Vec::with_capacity(builtin_plugins.len());
    let js_hooks_plugin = JsHooksAdapterPlugin::from_js_hooks(env, register_js_taps)?;
    plugins.push(js_hooks_plugin.clone().boxed());

    let tsfn = env
      .create_function("cleanup_revoked_modules", cleanup_revoked_modules)?
      .build_threadsafe_function::<External<Vec<ModuleIdentifier>>>()
      .weak::<true>()
      .callee_handled::<false>()
      .max_queue_size::<1>()
      .build()?;
    let js_cleanup_plugin = JsCleanupPlugin::new(tsfn);
    plugins.push(js_cleanup_plugin.boxed());

    for bp in builtin_plugins {
      bp.append_to(env, &mut this, &mut plugins)?;
    }

    let compiler_options: rspack_core::CompilerOptions = options.try_into().to_napi_result()?;

    tracing::debug!("normalized_options: {:#?}", &compiler_options);

    let resolver_factory =
      (*resolver_factory_reference).get_resolver_factory(compiler_options.resolve.clone());
    let loader_resolver_factory = (*resolver_factory_reference)
      .get_loader_resolver_factory(compiler_options.resolve_loader.clone());

    let intermediate_filesystem: Option<Arc<dyn IntermediateFileSystem>> =
      if let Some(fs) = intermediate_filesystem {
        Some(Arc::new(
          NodeFileSystem::new(fs).to_napi_result_with_message(|e| {
            format!("Failed to create intermediate filesystem: {e}")
          })?,
        ))
      } else {
        None
      };

    let rspack = rspack_core::Compiler::new(
      compiler_path,
      compiler_options,
      plugins,
      buildtime_plugins::buildtime_plugins(),
      Some(Arc::new(
        NodeFileSystem::new(output_filesystem)
          .to_napi_result_with_message(|e| format!("Failed to create writable filesystem: {e}"))?,
      )),
      intermediate_filesystem,
      None,
      Some(resolver_factory),
      Some(loader_resolver_factory),
    );

    Ok(Self {
      compiler: Compiler::from(rspack),
      state: CompilerState::init(),
      js_hooks_plugin,
      include_dependencies_map: Default::default(),
      entry_dependencies_map: Default::default(),
    })
  }

  #[napi]
  pub fn set_non_skippable_registers(&self, kinds: Vec<RegisterJsTapKind>) {
    self.js_hooks_plugin.set_non_skippable_registers(kinds)
  }

  /// Build with the given option passed to the constructor
  #[napi(ts_args_type = "callback: (err: null | Error) => void")]
  pub fn build(&mut self, reference: Reference<JsCompiler>, f: Function) -> Result<()> {
    unsafe {
      self.run(reference, |compiler, guard| {
        callbackify(
          f,
          async move {
            compiler.build().await.to_napi_result_with_message(|e| {
              print_error_diagnostic(e, compiler.options.stats.colors)
            })?;
            tracing::debug!("build ok");
            Ok(())
          },
          || drop(guard),
        )
      })
    }
  }

  /// Rebuild with the given option passed to the constructor
  #[napi(
    ts_args_type = "changed_files: string[], removed_files: string[], callback: (err: null | Error) => void"
  )]
  pub fn rebuild(
    &mut self,
    reference: Reference<JsCompiler>,
    changed_files: Vec<String>,
    removed_files: Vec<String>,
    f: Function,
  ) -> Result<()> {
    use std::collections::HashSet;

    unsafe {
      self.run(reference, |compiler, guard| {
        callbackify(
          f,
          async move {
            compiler
              .rebuild(
                HashSet::from_iter(changed_files.into_iter()),
                HashSet::from_iter(removed_files.into_iter()),
              )
              .await
              .to_napi_result_with_message(|e| {
                print_error_diagnostic(e, compiler.options.stats.colors)
              })?;
            tracing::debug!("rebuild ok");
            Ok(())
          },
          || drop(guard),
        )
      })
    }
  }

  #[napi]
  pub async fn close(&self) -> Result<()> {
    self
      .compiler
      .close()
      .await
      .to_napi_result_with_message(|e| {
        print_error_diagnostic(e, self.compiler.options.stats.colors)
      })?;
    Ok(())
  }
}

struct RunGuard {
  _compiler_state_guard: CompilerStateGuard,
  _reference: Reference<JsCompiler>,
}

impl JsCompiler {
  /// Run the given function with the compiler.
  ///
  /// ## Safety
  /// 1. The caller must ensure that the `Compiler` is not moved or dropped during the lifetime of the callback.
  /// 2. `CompilerStateGuard` should and only be dropped so soon as each `Compiler` is free of use.
  ///    Accessing `Compiler` beyond the lifetime of `CompilerStateGuard` would lead to potential race condition.
  unsafe fn run<R>(
    &mut self,
    mut reference: Reference<JsCompiler>,
    f: impl FnOnce(&'static mut Compiler, RunGuard) -> Result<R>,
  ) -> Result<R> {
    if self.state.running() {
      return Err(concurrent_compiler_error());
    }

    let compiler_state_guard = self.state.enter();
    let weak_reference = reference.downgrade();
    // SAFETY:
    // We ensure the lifetime of JsCompiler by holding a Reference<JsCompiler> until the JS callback function completes.
    // Therefore, we can safely transmute the lifetime of Compiler to 'static here.
    let compiler = unsafe {
      std::mem::transmute::<&mut Compiler, &'static mut Compiler>(&mut reference.compiler)
    };
    let guard = RunGuard {
      _compiler_state_guard: compiler_state_guard,
      _reference: reference,
    };
    COMPILER_REFERENCES.with(|ref_cell| {
      let mut references = ref_cell.borrow_mut();
      references.insert(compiler.id(), weak_reference);
    });

    self.cleanup_last_compilation(&compiler.compilation);

    f(compiler, guard)
  }

  fn cleanup_last_compilation(&self, compilation: &Compilation) {
    let compilation_id = compilation.id();

    JsCompilationWrapper::cleanup_last_compilation(compilation_id);
    JsChunkWrapper::cleanup_last_compilation(compilation_id);
    JsChunkGroupWrapper::cleanup_last_compilation(compilation_id);
    DependencyWrapper::cleanup_last_compilation(compilation_id);
    AsyncDependenciesBlockWrapper::cleanup_last_compilation(compilation_id);
  }
}

impl ObjectFinalize for JsCompiler {
  fn finalize(self, _env: Env) -> Result<()> {
    let compiler_id = self.compiler.id();

    COMPILER_REFERENCES.with(|ref_cell| {
      let mut references = ref_cell.borrow_mut();
      references.remove(&compiler_id);
    });

    ModuleObject::cleanup_by_compiler_id(&compiler_id);
    Ok(())
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

#[cfg(target_family = "wasm")]
const _: () = {
  #[used]
  #[link_section = ".init_array"]
  static __CTOR: unsafe extern "C" fn() = init;

  unsafe extern "C" fn init() {
    let rt = tokio::runtime::Builder::new_multi_thread()
      .max_blocking_threads(1)
      .enable_all()
      .build()
      .expect("Create tokio runtime failed");
    create_custom_tokio_runtime(rt);
  }
};

#[cfg(not(target_family = "wasm"))]
#[napi::ctor::ctor(crate_path = ::napi::ctor)]
fn init() {
  use std::{
    sync::atomic::{AtomicUsize, Ordering},
    thread,
  };

  panic::install_panic_handler();
  // control the number of blocking threads, similar as https://github.com/tokio-rs/tokio/blob/946401c345d672d357693740bc51f77bc678c5c4/tokio/src/loom/std/mod.rs#L93
  const ENV_BLOCKING_THREADS: &str = "RSPACK_BLOCKING_THREADS";
  // reduce default blocking threads on macOS cause macOS holds IORWLock on every file open
  // reference from https://github.com/oven-sh/bun/pull/17577/files#diff-c9bc275f9466e5179bb80454b6445c7041d2a0fb79932dd5de7a5c3196bdbd75R144
  let default_blocking_threads = if std::env::consts::OS == "macos" {
    8
  } else {
    512
  };
  let blocking_threads = std::env::var(ENV_BLOCKING_THREADS)
    .ok()
    .and_then(|v| v.parse::<usize>().ok())
    .unwrap_or(default_blocking_threads);
  let rt = tokio::runtime::Builder::new_multi_thread()
    .max_blocking_threads(blocking_threads)
    .thread_name_fn(|| {
      static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
      let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
      format!("tokio-{}", id)
    })
    .enable_all()
    .build()
    .expect("Create tokio runtime failed");
  create_custom_tokio_runtime(rt);
}

fn print_error_diagnostic(e: rspack_error::Error, colored: bool) -> String {
  Diagnostic::from(e)
    .render_report(colored)
    .expect("should print diagnostics")
}

thread_local! {
  static GLOBAL_TRACE_STATE: RefCell<TraceState> = const { RefCell::new(TraceState::Off) };
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
  #[napi(ts_arg_type = "\"chrome\" | \"logger\" ")] layer: String,
  output: String,
) -> anyhow::Result<()> {
  GLOBAL_TRACE_STATE.with(|state| {
    let mut state = state.borrow_mut();
    if let TraceState::Off = *state {
      let mut tracer: Box<dyn Tracer> = match layer.as_str() {
        "chrome" => Box::new(ChromeTracer::default()),
        "logger" => Box::new(StdoutTracer),
        _ => anyhow::bail!(
          "Unexpected layer: {}, supported layers: 'chrome', 'logger', 'console' ",
          layer
        ),
      };
      if let Some(layer) = tracer.setup(&output) {
        // SAFETY: we know that trace_var is `Ok(String)` now,
        // for the second unwrap, if we can't parse the directive, then the tracing result would be
        // unexpected, then panic is reasonable
        let filter = EnvFilter::builder()
          .with_default_directive(LevelFilter::INFO.into())
          .with_regex(true)
          .parse(filter)
          .expect("Parse tracing directive syntax failed, for details about the directive syntax you could refer https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives");
        tracing_subscriber::registry()
          .with(<_ as Layer<Registry>>::with_filter(layer, filter))
          .init();
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
    let mut state = state.borrow_mut();
    if let TraceState::On(ref mut tracer) = *state {
      tracer.teardown();
    }
    *state = TraceState::Off;
  });
}

#[module_exports]
fn node_init(mut _exports: Object, env: Env) -> Result<()> {
  rspack_core::set_thread_local_allocator(Box::new(allocator::NapiAllocatorImpl::new(env)));
  Ok(())
}

#[napi]
/// Shutdown the tokio runtime manually.
///
/// This is required for the wasm target with `tokio_unstable` cfg.
/// In the wasm runtime, the `park` threads will hang there until the tokio::Runtime is shutdown.
pub fn shutdown_async_runtime() {
  #[cfg(all(target_family = "wasm", tokio_unstable))]
  napi::bindgen_prelude::shutdown_async_runtime();
}

#[napi]
/// Start the async runtime manually.
///
/// This is required when the async runtime is shutdown manually.
/// Usually it's used in test.
pub fn start_async_runtime() {
  #[cfg(all(target_family = "wasm", tokio_unstable))]
  napi::bindgen_prelude::start_async_runtime();
}
