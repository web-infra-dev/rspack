#![recursion_limit = "256"]
#![allow(deprecated)]

#[macro_use]
extern crate napi_derive;
extern crate rspack_allocator;
use std::{cell::RefCell, sync::Arc};

use compiler::{Compiler, CompilerState, CompilerStateGuard};
use fs_node::HybridFileSystem;
use napi::{CallContext, bindgen_prelude::*};
use rspack_collections::UkeyMap;
use rspack_core::{
  BoxDependency, Compilation, CompilerId, EntryOptions, ModuleIdentifier, PluginExt,
};
use rspack_error::Diagnostic;
use rspack_fs::{IntermediateFileSystem, NativeFileSystem, ReadableFileSystem};
use rspack_tasks::{CURRENT_COMPILER_CONTEXT, CompilerContext, within_compiler_context_sync};
use rspack_util::tracing_preset::{
  TRACING_ALL_PRESET, TRACING_BENCH_TARGET, TRACING_OVERVIEW_PRESET,
};

use crate::{
  fs_node::{NodeFileSystem, ThreadsafeNodeFS},
  trace_event::RawTraceEvent,
};

mod allocator;
mod asset;
mod asset_condition;
mod async_dependency_block;
mod browserslist;
mod build_info;
mod chunk;
mod chunk_graph;
mod chunk_group;
mod clean_options;
mod codegen_result;
mod compilation;
mod compiler;
mod context_module_factory;
mod define_symbols;
mod dependencies;
mod dependency;
mod diagnostic;
mod error;
mod exports_info;
mod filename;
mod fs_node;
mod html;
mod identifier;
mod location;
mod module;
mod module_graph;
mod module_graph_connection;
mod modules;
mod native_watcher;
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
mod rslib;
mod rspack_resolver;
mod rstest;
mod runtime;
mod source;
mod stats;
mod swc;
mod trace_event;
mod utils;

pub use asset::*;
pub use asset_condition::*;
pub use async_dependency_block::*;
pub use browserslist::*;
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
pub use location::*;
pub use module::*;
pub use module_graph::*;
pub use module_graph_connection::*;
pub use modules::*;
pub use native_watcher::*;
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
pub use rslib::*;
pub use rspack_resolver::*;
use rspack_tracing::{PerfettoTracer, StdoutTracer, TraceEvent, Tracer};
pub use rstest::*;
pub use runtime::*;
use rustc_hash::FxHashMap;
pub use source::*;
pub use stats::*;
pub use swc::*;
use swc_core::common::util::take::Take;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
  EnvFilter, Layer, Registry, layer::SubscriberExt, reload, util::SubscriberInitExt,
};
pub use utils::*;

// Export expected @rspack/core version
/// Expected version of @rspack/core to the current binding version
/// @internal
#[napi]
pub const EXPECTED_RSPACK_CORE_VERSION: &str = rspack_workspace::rspack_pkg_version!();

thread_local! {
  pub static COMPILER_REFERENCES: RefCell<UkeyMap<CompilerId, WeakReference<JsCompiler>>> = Default::default();
}

#[js_function(1)]
fn cleanup_revoked_modules(ctx: CallContext) -> Result<()> {
  let external = ctx.get::<&mut External<(CompilerId, Vec<ModuleIdentifier>)>>(0)?;
  let compiler_id = external.0;
  let revoked_modules = external.1.take();
  ModuleObject::cleanup_by_module_identifiers(&compiler_id, &revoked_modules);
  Ok(())
}

#[napi(custom_finalize)]
pub struct JsCompiler {
  js_hooks_plugin: JsHooksAdapterPlugin,
  compiler: Compiler,
  state: CompilerState,
  include_dependencies_map: FxHashMap<String, FxHashMap<EntryOptions, BoxDependency>>,
  entry_dependencies_map: FxHashMap<String, FxHashMap<EntryOptions, BoxDependency>>,
  compiler_context: Arc<CompilerContext>,
}

#[napi]
impl JsCompiler {
  #[allow(clippy::too_many_arguments)]
  #[napi(constructor)]
  pub fn new(
    env: Env,
    mut this: This,
    compiler_path: String,
    mut options: RawOptions,
    builtin_plugins: Vec<BuiltinPlugin>,
    register_js_taps: RegisterJsTaps,
    output_filesystem: ThreadsafeNodeFS,
    intermediate_filesystem: Option<ThreadsafeNodeFS>,
    input_filesystem: Option<ThreadsafeNodeFS>,
    mut resolver_factory_reference: Reference<JsResolverFactory>,
  ) -> Result<Self> {
    tracing::info!(name:"rspack_version", version = rspack_workspace::rspack_pkg_version!());
    tracing::info!(name:"raw_options", options=?&options);
    let compiler_context = Arc::new(CompilerContext::new());
    CURRENT_COMPILER_CONTEXT.sync_scope(compiler_context.clone(), || {
      let mut plugins = Vec::with_capacity(builtin_plugins.len());
      let js_hooks_plugin = JsHooksAdapterPlugin::from_js_hooks(env, register_js_taps)?;
      plugins.push(js_hooks_plugin.clone().boxed());

      let tsfn = env
        .create_function("cleanup_revoked_modules", cleanup_revoked_modules)?
        .build_threadsafe_function::<External<(CompilerId, Vec<ModuleIdentifier>)>>()
        .weak::<true>()
        .callee_handled::<false>()
        .max_queue_size::<1>()
        .build()?;
      let js_cleanup_plugin = JsCleanupPlugin::new(tsfn);
      plugins.push(js_cleanup_plugin.boxed());

      for bp in builtin_plugins {
        bp.append_to(env, &mut this, &mut plugins)?;
      }

      let use_input_fs = options.experiments.use_input_file_system.take();
      let compiler_options: rspack_core::CompilerOptions = options.try_into().to_napi_result()?;

      tracing::debug!(name:"normalized_options", options=?&compiler_options);

      let input_file_system: Option<Arc<dyn ReadableFileSystem>> =
        input_filesystem.and_then(|fs| {
          use_input_fs.and_then(|use_input_file_system| {
            let node_fs = NodeFileSystem::new(fs).expect("Failed to create readable filesystem");

            match use_input_file_system {
              WithFalse::False => None,
              WithFalse::True(allowlist) => {
                if allowlist.is_empty() {
                  return None;
                }
                let binding: Arc<dyn ReadableFileSystem> = Arc::new(HybridFileSystem::new(
                  allowlist,
                  node_fs,
                  NativeFileSystem::new(compiler_options.resolve.pnp.unwrap_or(false)),
                ));
                Some(binding)
              }
            }
          })
        });

      resolver_factory_reference.update_options(
        input_file_system.clone(),
        compiler_options.resolve.clone(),
        compiler_options.resolve_loader.clone(),
      );
      let resolver_factory = resolver_factory_reference.get_resolver_factory();
      let loader_resolver_factory = resolver_factory_reference.get_loader_resolver_factory();

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
          NodeFileSystem::new(output_filesystem).to_napi_result_with_message(|e| {
            format!("Failed to create writable filesystem: {e}")
          })?,
        )),
        intermediate_filesystem,
        input_file_system,
        Some(resolver_factory),
        Some(loader_resolver_factory),
        Some(compiler_context.clone()),
      );

      Ok(Self {
        compiler: Compiler::from(rspack),
        state: CompilerState::init(),
        js_hooks_plugin,
        include_dependencies_map: Default::default(),
        entry_dependencies_map: Default::default(),
        compiler_context,
      })
    })
  }

  #[napi]
  pub fn set_non_skippable_registers(&self, kinds: Vec<RegisterJsTapKind>) {
    self.js_hooks_plugin.set_non_skippable_registers(kinds)
  }

  /// Build with the given option passed to the constructor
  #[napi(ts_args_type = "callback: (err: null | Error) => void")]
  pub fn build(
    &mut self,
    reference: Reference<JsCompiler>,
    f: Function<'static>,
  ) -> Result<(), ErrorCode> {
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
          Some(|| drop(guard)),
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
    f: Function<'static>,
  ) -> Result<(), ErrorCode> {
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
          Some(|| drop(guard)),
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
    f: impl FnOnce(&'static mut Compiler, RunGuard) -> Result<R, ErrorCode>,
  ) -> Result<R, ErrorCode> {
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
    within_compiler_context_sync(self.compiler_context.clone(), || f(compiler, guard))
  }

  fn cleanup_last_compilation(&self, compilation: &Compilation) {
    let compilation_id = compilation.id();

    JsCompilationWrapper::cleanup_last_compilation(compilation_id);
    ChunkWrapper::cleanup_last_compilation(compilation_id);
    ChunkGroupWrapper::cleanup_last_compilation(compilation_id);
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

fn concurrent_compiler_error() -> Error<ErrorCode> {
  Error::new(
    ErrorCode::Napi(Status::GenericFailure),
    "ConcurrentCompilationError: You ran rspack twice. Each instance only supports a single concurrent compilation at a time.",
  )
}

#[derive(Default)]
enum TraceState {
  Uninitialized,                                            // uninitialized
  On(Box<dyn Tracer>, reload::Handle<EnvFilter, Registry>), // initialized and turned on
  #[default]
  Off,                                          // initialized but turned off
}

#[cfg(target_family = "wasm")]
const _: () = {
  #[used]
  #[unsafe(link_section = ".init_array")]
  static __CTOR: unsafe extern "C" fn() = init;

  unsafe extern "C" fn init() {
    #[cfg(feature = "browser")]
    rspack_browser::panic::install_panic_handler();
    #[cfg(not(feature = "browser"))]
    panic::install_panic_handler();
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

  #[cfg(feature = "sftrace-setup")]
  if std::env::var_os("SFTRACE_OUTPUT_FILE").is_some() {
    unsafe {
      sftrace_setup::setup();
    }
  }

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
      format!("tokio-{id}")
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
  static GLOBAL_TRACE_STATE: RefCell<TraceState> = const { RefCell::new(TraceState::Uninitialized) };
}
/**
 * this is a process level tracing, which means it would be shared by all compilers in the same process
 * only the first call would take effect, the following calls would be ignored
 * Some code is modified based on
 * https://github.com/swc-project/swc/blob/d1d0607158ab40463d1b123fed52cc526eba8385/bindings/binding_core_node/src/util.rs#L29-L58
 * Apache-2.0 licensed
 * Author Donny/강동윤
 * Copyright (c)
 */
#[napi]
pub fn register_global_trace(
  filter: String,
  #[napi(ts_arg_type = " \"logger\" | \"perfetto\" ")] layer: String,
  output: String,
) -> anyhow::Result<()> {
  let filter = match filter.as_str() {
    "OVERVIEW" => TRACING_OVERVIEW_PRESET,
    "ALL" => TRACING_ALL_PRESET,
    "BENCH" => TRACING_BENCH_TARGET,
    _ => filter.as_str(),
  };
  GLOBAL_TRACE_STATE.with(|state| {
    let mut state = state.borrow_mut();
    if let TraceState::Uninitialized = *state {
      let mut tracer: Box<dyn Tracer> = match layer.as_str() {
        "logger" => Box::new(StdoutTracer),
        "perfetto"=> Box::new(PerfettoTracer::default()),
        _ => anyhow::bail!(
          "Unexpected layer: {}, supported layers:'logger', 'perfetto' ",
          layer
        ),
      };
      if let Some(layer) = tracer.setup(&output) {
        // SAFETY: we know that trace_var is `Ok(String)` now,
        // for the second unwrap, if we can't parse the directive, then the tracing result would be
        // unexpected, then panic is reasonable
        let (filter,reload_handle) = reload::Layer::new(EnvFilter::builder()
          .with_default_directive(LevelFilter::INFO.into())
          .with_regex(true)
          .parse(filter)
          .expect("Parse tracing directive syntax failed, for details about the directive syntax you could refer https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives")
      );
        tracing_subscriber::registry()
          .with(<_ as Layer<Registry>>::with_filter(layer, filter))
          .init();
        let new_state = TraceState::On(tracer, reload_handle);
        *state = new_state;
      };
    }
    Ok(())
  })
}

#[napi]
// only the first call would take effect, the following calls would be ignored
pub fn cleanup_global_trace() {
  GLOBAL_TRACE_STATE.with(|state| {
    let mut state = state.borrow_mut();
    match *state {
      TraceState::Uninitialized => {
        panic!("Global trace is not initialized, please call register_global_trace first");
      }
      TraceState::Off => {
        // do nothing, already cleaned up
      }
      TraceState::On(ref mut tracer, ref mut reload_handle) => {
        tracer.teardown();
        // turn off the tracing event
        let _ = reload_handle.modify(|filter| *filter = EnvFilter::new("off"));
        *state = TraceState::Off;
      }
    }
  });
}
// sync Node.js event to Rust side
#[napi]
pub fn sync_trace_event(events: Vec<RawTraceEvent>) {
  use std::borrow::BorrowMut;
  GLOBAL_TRACE_STATE.with(|state| {
    let mut state = state.borrow_mut();
    if let TraceState::On(tracer, _) = &mut **state.borrow_mut() {
      tracer.sync_trace(
        events
          .into_iter()
          .map(|event| TraceEvent {
            name: event.name,
            track_name: event.track_name,
            process_name: event.process_name,
            args: event
              .args
              .map(|args| args.into_iter().map(|(k, v)| (k, v.to_string())).collect()),
            uuid: event.uuid,
            ts: event.ts.get_u64().1,
            ph: event.ph,
            categories: event.categories,
          })
          .collect(),
      );
    }
  });
}

fn node_init(mut _exports: Object, env: Env) -> Result<()> {
  rspack_core::set_thread_local_allocator(Box::new(allocator::NapiAllocatorImpl::new(env)));
  Ok(())
}

#[napi(module_exports)]
pub fn rspack_module_exports(exports: Object, env: Env) -> Result<()> {
  node_init(exports, env)?;
  module::export_symbols(exports, env)?;
  build_info::export_symbols(exports, env)?;
  error::export_symbols(exports, env)?;
  Ok(())
}
