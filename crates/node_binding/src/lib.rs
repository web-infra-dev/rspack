#[macro_use]
extern crate napi_derive;

use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::JsObject;

use tokio::sync::RwLock;

mod js_values;
mod plugins;
mod utils;

use js_values::*;
use plugins::*;
use rspack_binding_options::*;
use utils::*;

#[cfg(all(not(all(target_os = "linux", target_arch = "aarch64", target_env = "musl"))))]
#[global_allocator]
static ALLOC: mimalloc_rust::GlobalMiMalloc = mimalloc_rust::GlobalMiMalloc;

#[napi]
pub struct Rspack {
  inner: Pin<Arc<RwLock<rspack::Compiler>>>,
}

#[napi]
impl Rspack {
  #[napi(constructor)]
  pub fn new(
    env: Env,
    mut options: RawOptions,
    plugin_callbacks: Option<PluginCallbacks>,
  ) -> Result<Self> {
    Self::prepare_environment(&env, &mut options);
    rspack_tracing::enable_tracing_by_env();
    tracing::info!("raw_options: {:?}", &options);
    let compiler_options = create_node_adapter_from_plugin_callbacks(env, plugin_callbacks)
      .and_then(|node_adapter| {
        let mut compiler_options =
          normalize_bundle_options(options).map_err(|e| Error::from_reason(format!("{:?}", e)))?;

        if let Some(node_adapter) = node_adapter {
          compiler_options
            .plugins
            .push(Box::new(node_adapter) as Box<dyn rspack_core::Plugin>);
        }

        compiler_options
          .module
          .rules
          .iter_mut()
          .try_for_each(|rule| {
            rule.uses.iter_mut().try_for_each(|loader| {
              let casted = loader.as_any_mut();
              if let Some(adapter) = casted.downcast_mut::<NodeLoaderAdapter>() {
                adapter.unref(&env)
              } else {
                Ok(())
              }
            })
          })
          .map_err(|e| Error::from_reason(format!("failed to unref tsfn {:?}", e)))?;

        Ok(compiler_options)
      })?;
    tracing::info!("normalized_options: {:?}", &compiler_options);
    let rspack = rspack::rspack(compiler_options, vec![]);
    Ok(Self {
      inner: Arc::pin(RwLock::new(rspack)),
    })
  }

  #[napi(ts_return_type = "Promise<StatsCompilation>")]
  pub fn build(&self, env: Env) -> Result<JsObject> {
    let inner = self.inner.clone();
    env.execute_tokio_future(
      async move {
        let mut compiler = inner.write().await;

        let rspack_stats = compiler
          .build()
          .await
          .map_err(|e| Error::new(napi::Status::GenericFailure, format!("{:?}", e)))?;

        let stats: StatsCompilation = rspack_stats.to_description().into();
        if stats.errors.is_empty() {
          tracing::info!("build success");
        } else {
          tracing::info!("build failed");
        }
        Ok(stats)
      },
      |_env, ret| Ok(ret),
    )
  }

  #[napi(ts_return_type = "Promise<Record<string, {content: string, kind: number}>>")]
  pub fn rebuild(
    &self,
    env: Env,
    changed_files: Vec<String>,
    removed_files: Vec<String>,
  ) -> Result<JsObject> {
    let inner = self.inner.clone();

    env.execute_tokio_future(
      async move {
        let mut compiler = inner.write().await;

        let diff = compiler
          .rebuild(
            HashSet::from_iter(changed_files.into_iter()),
            HashSet::from_iter(removed_files.into_iter()),
          )
          .await
          .map_err(|e| Error::new(napi::Status::GenericFailure, format!("{:?}", e)))?;
        let stats: HashMap<String, DiffStat> = diff
          .into_iter()
          .map(|(uri, stats)| {
            (
              uri,
              DiffStat {
                kind: DiffStatKind::from(stats.0),
                content: stats.1,
              },
            )
          })
          .collect();
        // let stats: Stats = _rspack_stats.into();

        tracing::info!("rebuild success");
        Ok(stats)
      },
      |_env, ret| Ok(ret),
    )
  }
}

impl Rspack {
  fn prepare_environment(env: &Env, options: &mut RawOptions) {
    NAPI_ENV.with(|napi_env| *napi_env.borrow_mut() = Some(env.raw()));

    #[cfg(debug_assertions)]
    {
      if let Some(module) = options.module.as_mut() {
        for rule in &mut module.rules {
          if let Some(uses) = rule.uses.as_mut() {
            for item in uses {
              if let Some(loader) = item.loader.as_ref() {
                // let (env_ptr, loader_ptr) = unsafe { (env.raw(), loader.raw()) };
                if let Ok(display_name) =
                  get_named_property_value_string(*env, loader, "displayName")
                {
                  item.__loader_name = Some(display_name);
                } else if let Ok(name) = get_named_property_value_string(*env, loader, "name") {
                  item.__loader_name = Some(name);
                }
              }
            }
          }
        }
      }
    }
  }
}

#[napi::module_init]
fn init() {
  use backtrace::Backtrace;
  use std::panic::set_hook;

  // set_hook(Box::new(|panic_info| {
  //   let backtrace = Backtrace::new();
  //   println!("Panic: {:?}\nBacktrace: {:?}", panic_info, backtrace);
  //   std::process::exit(1)
  // }));
}
