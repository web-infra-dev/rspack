#![recursion_limit = "256"]
#![feature(let_chains)]
#![feature(try_blocks)]
#[macro_use]
extern crate napi_derive;
extern crate rspack_allocator;

use std::{
  cell::RefCell,
  collections::HashMap,
  pin::Pin,
  str::FromStr,
  sync::{Arc, Mutex, RwLock},
};

use compiler::{Compiler, CompilerState, CompilerStateGuard};
use napi::{
  bindgen_prelude::*,
  threadsafe_function::{ThreadsafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode},
  CallContext, Env,
};
use once_cell::sync::Lazy;
use rspack_collections::UkeyMap;
use rspack_core::{
  BoxDependency, Compilation, CompilerId, EntryOptions, ModuleIdentifier, PluginExt,
};
use rspack_error::Diagnostic;
use rspack_fs::IntermediateFileSystem;

use crate::fs_node::{NodeFileSystem, ThreadsafeNodeFS};

mod asset;
mod asset_condition;
mod async_dependency_block;
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
mod runtime;
mod source;
mod stats;
mod utils;

pub use asset::*;
pub use asset_condition::*;
pub use async_dependency_block::*;
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
use rspack_plugin_schemes::{HttpClient, HttpResponse};
use rspack_tracing::{ChromeTracer, StdoutTracer, Tracer};
pub use runtime::*;
use rustc_hash::FxHashMap;
pub use source::*;
pub use stats::*;
use swc_core::common::util::take::Take;
use tracing::Level;
use tracing_subscriber::{
  layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer, Registry,
};
pub use utils::*;

thread_local! {
  pub static COMPILER_REFERENCES: RefCell<UkeyMap<CompilerId, WeakReference<JsCompiler>>> = Default::default();
}

static HTTP_CLIENT_FUNCTION: Lazy<RwLock<Option<ThreadsafeFunction<(), Unknown>>>> =
  Lazy::new(|| RwLock::new(None));

#[derive(Debug)]
struct JsHttpClientImpl;

#[async_trait::async_trait]
impl HttpClient for JsHttpClientImpl {
  async fn get(
    &self,
    url: &str,
    headers: &HashMap<String, String>,
  ) -> anyhow::Result<HttpResponse> {
    let tsfn = match HTTP_CLIENT_FUNCTION.read() {
      Ok(guard) => match guard.as_ref() {
        Some(tsfn) => tsfn.clone(),
        None => {
          return Err(anyhow::anyhow!(
            "HTTP client not registered. Make sure HttpUriPlugin is properly initialized."
          ))
        }
      },
      Err(_) => return Err(anyhow::anyhow!("Failed to access HTTP client")),
    };

    let (tx, rx) = tokio::sync::oneshot::channel();

    let sender = Arc::new(Mutex::new(Some(tx)));

    let url_str = url.to_string();
    let headers_map = headers.clone();

    tokio::task::spawn_blocking(move || {
      let cb = move |ctx: ThreadsafeCallContext<Unknown>| {
        let mut sender_guard = match sender.lock() {
          Ok(guard) => guard,
          Err(_) => return Ok(()),
        };

        let sender = match sender_guard.take() {
          Some(s) => s,
          None => return Ok(()),
        };

        let result = ctx.value;
        let env = ctx.env;
        let result_obj = match result.coerce_to_object() {
          Ok(obj) => obj,
          Err(err) => {
            let _ = sender.send(Err(anyhow::anyhow!(
              "Failed to convert result to object: {}",
              err
            )));
            return Ok(());
          }
        };

        let status = match result_obj.get_named_property::<u16>("status") {
          Ok(s) => s,
          Err(_) => 200,
        };

        let mut response_headers = HashMap::new();
        if let Ok(headers_obj) = result_obj.get_named_property::<Object>("headers") {
          if let Ok(props) = headers_obj.get_property_names() {
            let props_len = props.get_array_length().unwrap_or(0);
            for i in 0..props_len {
              if let (Ok(key_js), Ok(val_js)) = (
                props
                  .get_element::<Unknown>(i)
                  .and_then(|k| k.coerce_to_string()),
                headers_obj
                  .get_element::<Unknown>(i)
                  .and_then(|v| v.coerce_to_string()),
              ) {
                let key = key_js
                  .into_utf8()
                  .ok()
                  .and_then(|s| s.as_str().ok())
                  .unwrap_or_default()
                  .to_string();
                let val = val_js
                  .into_utf8()
                  .ok()
                  .and_then(|s| s.as_str().ok())
                  .unwrap_or_default()
                  .to_string();
                if !key.is_empty() {
                  response_headers.insert(key, val);
                }
              }
            }
          }
        }

        let body = if let Ok(body_str) = result_obj.get_named_property::<String>("body") {
          body_str.as_bytes().to_vec()
        } else {
          Vec::new()
        };

        let http_response = HttpResponse {
          status,
          headers: response_headers,
          body,
        };

        let _ = sender.send(Ok(http_response));
        Ok(())
      };

      if let Ok(env) = napi::Env::get_current_env() {
        if let Ok(js_url) = env.create_string(&url_str) {
          if let Ok(js_headers) = env.create_object() {
            for (key, value) in headers_map {
              if let Ok(js_value) = env.create_string(&value) {
                let _ = js_headers.set_named_property(&key, js_value);
              }
            }

            if let Err(e) = env.run_in_scope(|scope| {
              let args = vec![js_url.into_unknown(scope), js_headers.into_unknown(scope)];
              tsfn.call(args, ThreadsafeFunctionCallMode::Blocking, cb);
              Ok(())
            }) {
              tracing::error!("Failed to call JavaScript HTTP client: {}", e);
            }
          } else {
            tracing::error!("Failed to create headers object");
          }
        } else {
          tracing::error!("Failed to create URL string");
        }
      } else {
        tracing::error!("Failed to get current env");
      }
    });

    match tokio::time::timeout(std::time::Duration::from_secs(30), rx).await {
      Ok(result) => match result {
        Ok(response) => response,
        Err(e) => Err(anyhow::anyhow!("Channel error: {}", e)),
      },
      Err(_) => Err(anyhow::anyhow!("HTTP request timed out after 30 seconds")),
    }
  }
}

static HTTP_CLIENT: Lazy<Arc<dyn HttpClient>> = Lazy::new(|| Arc::new(JsHttpClientImpl));

pub fn get_http_client() -> Option<Arc<dyn HttpClient>> {
  match HTTP_CLIENT_FUNCTION.read() {
    Ok(guard) => {
      if guard.is_some() {
        Some(HTTP_CLIENT.clone())
      } else {
        None
      }
    }
    Err(_) => None,
  }
}

#[napi]
pub fn register_http_client(_env: Env, client: Function) -> Result<()> {
  let tsfn =
    ThreadsafeFunction::create(client.raw(), 0, |ctx: ThreadsafeCallContext<()>| Ok(vec![]))?;

  if let Ok(mut guard) = HTTP_CLIENT_FUNCTION.write() {
    *guard = Some(tsfn);
    tracing::info!("HTTP client registered successfully");
  }

  Ok(())
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
  compiler: Pin<Box<Compiler>>,
  state: CompilerState,
  include_dependencies_map: FxHashMap<String, FxHashMap<EntryOptions, BoxDependency>>,
}

#[napi]
impl JsCompiler {
  #[allow(clippy::too_many_arguments)]
  #[napi(constructor)]
  pub fn new(
    env: Env,
    compiler_path: String,
    options: RawOptions,
    builtin_plugins: Vec<BuiltinPlugin>,
    register_js_taps: RegisterJsTaps,
    output_filesystem: ThreadsafeNodeFS,
    intermediate_filesystem: Option<ThreadsafeNodeFS>,
    mut resolver_factory_reference: Reference<JsResolverFactory>,
  ) -> Result<Self> {
    tracing::info!("raw_options: {:#?}", &options);

    let mut plugins = Vec::new();
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

    let intermediate_filesystem: Option<Arc<dyn IntermediateFileSystem>> =
      if let Some(fs) = intermediate_filesystem {
        Some(Arc::new(NodeFileSystem::new(fs).map_err(|e| {
          Error::from_reason(format!("Failed to create intermediate filesystem: {e}",))
        })?))
      } else {
        None
      };

    let rspack = rspack_core::Compiler::new(
      compiler_path,
      compiler_options,
      plugins,
      buildtime_plugins::buildtime_plugins(),
      Some(Arc::new(NodeFileSystem::new(output_filesystem).map_err(
        |e| Error::from_reason(format!("Failed to create writable filesystem: {e}",)),
      )?)),
      intermediate_filesystem,
      None,
      Some(resolver_factory),
      Some(loader_resolver_factory),
    );

    Ok(Self {
      compiler: Box::pin(Compiler::from(rspack)),
      state: CompilerState::init(),
      js_hooks_plugin,
      include_dependencies_map: Default::default(),
    })
  }

  #[napi]
  pub fn set_non_skippable_registers(&self, kinds: Vec<RegisterJsTapKind>) {
    self.js_hooks_plugin.set_non_skippable_registers(kinds)
  }

  /// Build with the given option passed to the constructor
  #[napi(ts_args_type = "callback: (err: null | Error) => void")]
  pub fn build(&mut self, env: Env, reference: Reference<JsCompiler>, f: Function) -> Result<()> {
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
    reference: Reference<JsCompiler>,
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

impl JsCompiler {
  /// Run the given function with the compiler.
  ///
  /// ## Safety
  /// 1. The caller must ensure that the `Compiler` is not moved or dropped during the lifetime of the callback.
  /// 2. `CompilerStateGuard` should and only be dropped so soon as each `Compiler` is free of use.
  ///    Accessing `Compiler` beyond the lifetime of `CompilerStateGuard` would lead to potential race condition.
  unsafe fn run<R>(
    &mut self,
    env: Env,
    reference: Reference<JsCompiler>,
    f: impl FnOnce(&'static mut Compiler, CompilerStateGuard) -> Result<R>,
  ) -> Result<R> {
    COMPILER_REFERENCES.with(|ref_cell| {
      let mut references = ref_cell.borrow_mut();
      references.insert(reference.compiler.id(), reference.downgrade());
    });

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

#[cfg(not(target_family = "wasm"))]
#[ctor]
fn init() {
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
  #[napi(ts_arg_type = "\"chrome\" | \"logger\" | \"otel\"")] layer: String,
  output: String,
) -> anyhow::Result<()> {
  GLOBAL_TRACE_STATE.with(|state| {
    let mut state = state.lock().expect("Failed to lock GLOBAL_TRACE_STATE");
    if let TraceState::Off = *state {
      let mut tracer: Box<dyn Tracer> = match layer.as_str() {
        "chrome" => Box::new(ChromeTracer::default()),
        #[cfg(not(target_family = "wasm"))]
        "otel" => {
          use rspack_tracing::OtelTracer;
          use rspack_napi::napi::bindgen_prelude::within_runtime_if_available;
          Box::new(within_runtime_if_available(OtelTracer::default))
        },
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
