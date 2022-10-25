#[macro_use]
extern crate napi_derive;

use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

use napi::bindgen_prelude::*;
use napi::JsObject;

use once_cell::sync::Lazy;
use rspack_tracing::enable_tracing_by_env;

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

struct SingleThreadedHashMap<K, V>(Mutex<HashMap<K, V>>);

impl<K, V> SingleThreadedHashMap<K, V>
where
  K: Eq + std::hash::Hash,
{
  /// Acquire a mutable reference to the inner hashmap.
  ///
  /// Safety: Mutable reference can almost let you do anything you want, this is intended to be used from the thread where the map was created.
  #[allow(unused)]
  unsafe fn borrow_mut<F, R>(&self, f: F) -> Result<R>
  where
    F: FnOnce(&mut HashMap<K, V>) -> Result<R>,
  {
    let mut inner = self
      .0
      .lock()
      .map_err(|_| napi::Error::from_reason("Failed to acquire lock on single-threaded hashmap"))?;

    f(&mut *inner)
  }

  /// Acquire a shared reference to the inner hashmap.
  ///
  /// Safety: It's not thread-safe, so this is intended to be used from the thread where the map was created.
  #[allow(unused)]
  unsafe fn borrow<F, R>(&self, f: F) -> Result<R>
  where
    F: FnOnce(&HashMap<K, V>) -> Result<R>,
  {
    let inner = self
      .0
      .lock()
      .map_err(|_| napi::Error::from_reason("Failed to acquire lock on single-threaded hashmap"))?;

    f(&*inner)
  }
}

impl<K, V> Default for SingleThreadedHashMap<K, V> {
  fn default() -> Self {
    Self(Mutex::new(HashMap::new()))
  }
}

// Safety: Methods are already marked as unsafe.
unsafe impl<K, V> Send for SingleThreadedHashMap<K, V> {}
unsafe impl<K, V> Sync for SingleThreadedHashMap<K, V> {}

static COMPILERS: Lazy<SingleThreadedHashMap<CompilerId, rspack::Compiler>> =
  Lazy::new(SingleThreadedHashMap::default);

static COMPILER_ID: AtomicU32 = AtomicU32::new(1);

type CompilerId = u32;

#[napi]
pub struct Rspack {
  id: CompilerId,
}

#[napi]
impl Rspack {
  #[napi(constructor)]
  pub fn new(
    env: Env,
    mut options: RawOptions,
    plugin_callbacks: Option<PluginCallbacks>,
  ) -> Result<Self> {
    enable_tracing_by_env();
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

    let handle_compiler_creation = move |map: &mut HashMap<_, _>| {
      let id = COMPILER_ID.fetch_add(1, Ordering::SeqCst);
      map.insert(id, rspack);
      Ok(id)
    };

    let id = unsafe { COMPILERS.borrow_mut(handle_compiler_creation) }?;

    Ok(Self { id })
  }

  /// Build with the given option passed to the constructor
  ///
  /// Warning:
  /// Calling this method recursively will cause a panic.
  #[napi(js_name = "unsafe_build", ts_return_type = "Promise<StatsCompilation>")]
  pub fn build(&self, env: Env) -> Result<JsObject> {
    let handle_build = |map: &mut HashMap<_, _>| {
      // Safety: compiler is stored in a global hashmap, so it's guaranteed to be alive.
      let compiler = unsafe {
        std::mem::transmute::<&'_ mut rspack::Compiler, &'static mut rspack::Compiler>(
          map.get_mut(&self.id).unwrap(),
        )
      };

      env.execute_tokio_future(
        async move {
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
    };
    unsafe { COMPILERS.borrow_mut(handle_build) }
  }

  /// Rebuild with the given option passed to the constructor
  ///
  /// Warning:
  /// Calling this method recursively will cause a panic.
  #[napi(
    js_name = "unsafe_rebuild",
    ts_return_type = "Promise<Record<string, {content: string, kind: number}>>"
  )]
  pub fn rebuild(
    &self,
    env: Env,
    changed_files: Vec<String>,
    removed_files: Vec<String>,
  ) -> Result<JsObject> {
    let handle_rebuild = |map: &mut HashMap<_, _>| {
      // Safety: compiler is stored in a global hashmap, so it's guaranteed to be alive.
      let compiler = unsafe {
        std::mem::transmute::<&'_ mut rspack::Compiler, &'static mut rspack::Compiler>(
          map.get_mut(&self.id).unwrap(),
        )
      };

      env.execute_tokio_future(
        async move {
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
    };

    unsafe { COMPILERS.borrow_mut(handle_rebuild) }
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

  set_hook(Box::new(|panic_info| {
    let backtrace = Backtrace::new();
    println!("Panic: {:?}\nBacktrace: {:?}", panic_info, backtrace);
    std::process::exit(1)
  }));
}
