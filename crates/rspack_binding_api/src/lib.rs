#![recursion_limit = "256"]
#![allow(deprecated)]
#![allow(unused)]

//! `rspack_binding_api` is the core binding layer in the Rspack project, responsible for exposing Rspack core functionality written in Rust to JavaScript/TypeScript environments. It provides complete API interfaces for compilation, building, module processing, and other functionalities.
//!
//! ## Features
//!
//! - `browser`: Enable browser environment support
//! - `color-backtrace`: Enable colored error backtraces
//! - `debug_tool`: Enable debug tools
//! - `plugin`: Enable SWC plugin support
//! - `sftrace-setup`: Enable performance tracing setup
//!
//! ## Important Notice
//!
//! ⚠️ **Version Compatibility Warning**
//!
//! **This repository does not follow Semantic Versioning (SemVer).**
//!
//! - Any version update may contain breaking changes
//! - It is recommended to lock specific version numbers in production environments
//! - Please thoroughly test all functionality before upgrading
//!
//! ## API Usage Warning
//!
//! 🚨 **This package's API should NOT be used as a public Rust API**
//!
//! This crate is designed to be linked as a **C dynamic library** during Rspack binding generation, not as a public Rust API for external consumption.
//!
//! ### For Developers
//!
//! If you're working on Rspack itself:
//! - This crate is safe to use within the Rspack project
//! - Changes should be coordinated with the binding generation process
//! - Test thoroughly when making changes
//!
//! If you're an external developer:
//! - Do not depend on this crate directly
//! - Use the official Rspack Node.js package instead
//! - Report issues through the main Rspack repository
//!
//! If you're a user of Rspack custom binding:
//! - Do not depend on this crate directly
//! - Use [`rspack_binding_builder`](https://crates.io/crates/rspack_binding_builder) to build your own binding

#[macro_use]
extern crate napi_derive;
extern crate rspack_allocator;

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
mod virtual_modules;

use std::{
  cell::RefCell,
  mem::ManuallyDrop,
  sync::{Arc, RwLock},
};

use napi::{CallContext, bindgen_prelude::*};
pub use raw_options::{CustomPluginBuilder, register_custom_plugin};
use rspack_collections::UkeyMap;
use rspack_core::{
  BoxDependency, Compilation, CompilerId, EntryOptions, ModuleIdentifier, PluginExt,
};
use rspack_error::Diagnostic;
use rspack_fs::{IntermediateFileSystem, NativeFileSystem, ReadableFileSystem};
use rspack_tasks::{CURRENT_COMPILER_CONTEXT, CompilerContext, within_compiler_context_sync};
use rustc_hash::FxHashMap;
use swc_core::common::util::take::Take;

use crate::{
  async_dependency_block::AsyncDependenciesBlockWrapper,
  chunk::ChunkWrapper,
  chunk_group::ChunkGroupWrapper,
  compilation::JsCompilationWrapper,
  compiler::{Compiler, CompilerState, CompilerStateGuard},
  dependency::DependencyWrapper,
  error::{ErrorCode, RspackResultToNapiResultExt},
  fs_node::{HybridFileSystem, NodeFileSystem, ThreadsafeNodeFS},
  module::ModuleObject,
  plugins::{
    JsCleanupPlugin, JsHooksAdapterPlugin, RegisterJsTapKind, RegisterJsTaps, buildtime_plugins,
  },
  raw_options::{BuiltinPlugin, RawOptions, WithFalse},
  resolver_factory::JsResolverFactory,
  trace_event::RawTraceEvent,
  utils::callbackify,
  virtual_modules::{
    JsVirtualFileStore, TrieVirtualFileStore, VirtualFileStore, VirtualFileSystem,
  },
};

// Export expected @rspack/core version
/// Expected version of @rspack/core to the current binding version
/// @internal
#[napi]
pub const EXPECTED_RSPACK_CORE_VERSION: &str = rspack_workspace::rspack_pkg_version!();

thread_local! {
  static COMPILER_REFERENCES: RefCell<UkeyMap<CompilerId, WeakReference<JsCompiler>>> = Default::default();
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
struct JsCompiler {
  // whether to skip drop compiler in finalize
  unsafe_fast_drop: bool,
  js_hooks_plugin: JsHooksAdapterPlugin,
  // call drop manually to avoid unnecessary drop overhead in cli build
  compiler: ManuallyDrop<Compiler>,
  state: CompilerState,
  include_dependencies_map: FxHashMap<String, FxHashMap<EntryOptions, BoxDependency>>,
  entry_dependencies_map: FxHashMap<String, FxHashMap<EntryOptions, BoxDependency>>,
  compiler_context: Arc<CompilerContext>,
  virtual_file_store: Option<Arc<RwLock<dyn VirtualFileStore>>>,
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
    unsafe_fast_drop: bool,
  ) -> Result<Self> {
    tracing::info!(name:"rspack_version", version = rspack_workspace::rspack_pkg_version!());
    tracing::info!(name:"raw_options", options=?&options);
    let compiler_context = Arc::new(CompilerContext::new());
    CURRENT_COMPILER_CONTEXT.sync_scope(compiler_context.clone(), || {
      let mut plugins = Vec::with_capacity(builtin_plugins.len());
      let js_hooks_plugin = JsHooksAdapterPlugin::from_js_hooks(env, register_js_taps)?;
      plugins.push(js_hooks_plugin.clone().boxed());

      // Register builtin loader plugins
      plugins.push(Box::new(
        rspack_loader_lightningcss::LightningcssLoaderPlugin::new(),
      ));
      plugins.push(Box::new(rspack_loader_swc::SwcLoaderPlugin::new()));
      plugins.push(Box::new(rspack_plugin_rsc::ClientEntryLoaderPlugin::new()));
      plugins.push(Box::new(rspack_plugin_rsc::ActionEntryLoaderPlugin::new()));
      plugins.push(Box::new(
        rspack_loader_react_refresh::ReactRefreshLoaderPlugin::new(),
      ));
      plugins.push(Box::new(
        rspack_loader_preact_refresh::PreactRefreshLoaderPlugin::new(),
      ));

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

      let pnp = options.resolve.pnp.unwrap_or(false);
      let virtual_files = options.__virtual_files.take();
      let use_input_fs = options.experiments.use_input_file_system.take();
      let compiler_options: rspack_core::CompilerOptions = options.try_into().to_napi_result()?;

      tracing::debug!(name:"normalized_options", options=?&compiler_options);

      let mut input_file_system: Option<Arc<dyn ReadableFileSystem>> =
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

      let mut virtual_file_store: Option<Arc<RwLock<dyn VirtualFileStore>>> = None;
      if let Some(list) = virtual_files {
        let store = {
          let mut store = TrieVirtualFileStore::new();
          for f in list {
            store.write_file(f.path.as_str().into(), f.content.into());
          }
          Arc::new(RwLock::new(store))
        };
        input_file_system = input_file_system
          .map(|real_fs| {
            let binding: Arc<dyn ReadableFileSystem> =
              Arc::new(VirtualFileSystem::new(real_fs, store.clone()));
            binding
          })
          .or_else(|| {
            let binding: Arc<dyn ReadableFileSystem> = Arc::new(VirtualFileSystem::new(
              Arc::new(NativeFileSystem::new(pnp)),
              store.clone(),
            ));
            Some(binding)
          });
        virtual_file_store = Some(store);
      }

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
        compiler: ManuallyDrop::new(Compiler::from(rspack)),
        state: CompilerState::init(),
        js_hooks_plugin,
        include_dependencies_map: Default::default(),
        entry_dependencies_map: Default::default(),
        compiler_context,
        virtual_file_store,
        unsafe_fast_drop,
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

  #[napi]
  pub fn get_virtual_file_store(&self) -> Option<JsVirtualFileStore> {
    self
      .virtual_file_store
      .as_ref()
      .map(|store| JsVirtualFileStore::new(store.clone()))
  }

  #[napi]
  pub fn get_compiler_id(&self) -> External<CompilerId> {
    External::new(self.compiler.id())
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
  fn finalize(mut self, _env: Env) -> Result<()> {
    let compiler_id = self.compiler.id();

    COMPILER_REFERENCES.with(|ref_cell| {
      let mut references = ref_cell.borrow_mut();
      references.remove(&compiler_id);
    });

    ModuleObject::cleanup_by_compiler_id(&compiler_id);
    if (!self.unsafe_fast_drop) {
      unsafe {
        ManuallyDrop::drop(&mut self.compiler);
      }
    }
    Ok(())
  }
}

fn concurrent_compiler_error() -> Error<ErrorCode> {
  Error::new(
    ErrorCode::Napi(Status::GenericFailure),
    "ConcurrentCompilationError: You ran rspack twice. Each instance only supports a single concurrent compilation at a time.",
  )
}

#[cfg(not(target_family = "wasm"))]
#[napi::ctor::ctor(crate_path = ::napi::ctor)]
fn init() {
  use std::{
    sync::atomic::{AtomicUsize, Ordering},
    thread,
  };

  #[cfg(feature = "tracy-client")]
  {
    use tracy_client::register_demangler;
    tracy_client::Client::start();
    register_demangler!();
  }
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
  let default_blocking_threads = 4;
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
fn register_global_trace(
  filter: String,
  #[napi(ts_arg_type = " \"logger\" | \"perfetto\" ")] layer: String,
  output: String,
) -> anyhow::Result<()> {
  #[cfg(not(feature = "browser"))]
  trace_event::register_global_trace(filter, layer, output)?;
  Ok(())
}

#[napi]
// only the first call would take effect, the following calls would be ignored
pub fn cleanup_global_trace() {
  #[cfg(not(feature = "browser"))]
  trace_event::cleanup_global_trace();
}

// sync Node.js event to Rust side
#[napi]
fn sync_trace_event(events: Vec<RawTraceEvent>) {
  #[cfg(not(feature = "browser"))]
  trace_event::sync_trace_event(events);
}

fn node_init(mut _exports: Object, env: Env) -> Result<()> {
  rspack_core::set_thread_local_allocator(Box::new(allocator::NapiAllocatorImpl::new(env)));
  Ok(())
}

#[napi(module_exports)]
fn rspack_module_exports(exports: Object, env: Env) -> Result<()> {
  #[cfg(target_family = "wasm")]
  {
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

  node_init(exports, env)?;
  module::export_symbols(exports, env)?;
  build_info::export_symbols(exports, env)?;
  error::export_symbols(exports, env)?;
  Ok(())
}
