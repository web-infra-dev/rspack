#[macro_use]
extern crate napi_derive;

#[macro_use]
mod macros;

use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::atomic::{AtomicU32, Ordering};

use napi::bindgen_prelude::*;

use dashmap::DashMap;
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

// **Note** that Node's main thread and the worker thread share the same binding context. Using `Mutex<HashMap>` would cause deadlocks if multiple compilers exist.
struct SingleThreadedHashMap<K, V>(DashMap<K, V>);

impl<K, V> SingleThreadedHashMap<K, V>
where
  K: Eq + std::hash::Hash + std::fmt::Display,
{
  /// Acquire a mutable reference to the inner hashmap.
  ///
  /// Safety: Mutable reference can almost let you do anything you want, this is intended to be used from the thread where the map was created.
  #[allow(unused)]
  unsafe fn borrow_mut<F, R>(&self, key: &K, f: F) -> Result<R>
  where
    F: FnOnce(&mut V) -> Result<R>,
  {
    let mut inner = self.0.get_mut(&key).ok_or_else(|| {
      napi::Error::from_reason(format!(
        "Failed to find key {} for single-threaded hashmap",
        key
      ))
    })?;

    f(&mut *inner)
  }

  /// Acquire a shared reference to the inner hashmap.
  ///
  /// Safety: It's not thread-safe if a value is not safe to modify cross thread boundary, so this is intended to be used from the thread where the map was created.
  #[allow(unused)]
  unsafe fn borrow<F, R>(&self, key: &K, f: F) -> Result<R>
  where
    F: FnOnce(&V) -> Result<R>,
  {
    let inner = self.0.get(&key).ok_or_else(|| {
      napi::Error::from_reason(format!(
        "Failed to find key {} for single-threaded hashmap",
        key
      ))
    })?;

    f(&*inner)
  }

  /// Insert a value into the map.
  ///
  /// Safety: It's not thread-safe if a value has thread affinity, so this is intended to be used from the thread where the map was created.
  #[allow(unused)]
  unsafe fn insert_if_vacant(&self, key: K, value: V) -> Result<()> {
    if let dashmap::mapref::entry::Entry::Vacant(vacant) = self.0.entry(key) {
      vacant.insert(value);
      Ok(())
    } else {
      Err(napi::Error::from_reason(
        "Failed to insert on single-threaded hashmap as it's not vacant",
      ))
    }
  }

  /// Remove a value from the map.
  ///
  /// See: [DashMap::remove] for more details. https://docs.rs/dashmap/latest/dashmap/struct.DashMap.html#method.remove
  ///
  /// Safety: It's not thread-safe if a value has thread affinity, so this is intended to be used from the thread where the map was created.
  #[allow(unused)]
  unsafe fn remove(&self, key: &K) -> Option<V> {
    self.0.remove(key).map(|(_, v)| v)
  }
}

impl<K, V> Default for SingleThreadedHashMap<K, V>
where
  K: Eq + std::hash::Hash,
{
  fn default() -> Self {
    Self(Default::default())
  }
}

// Safety: Methods are already marked as unsafe.
unsafe impl<K, V> Send for SingleThreadedHashMap<K, V> {}
unsafe impl<K, V> Sync for SingleThreadedHashMap<K, V> {}

static COMPILERS: Lazy<SingleThreadedHashMap<CompilerId, rspack::Compiler>> =
  Lazy::new(Default::default);

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
    tracing::info!("raw_options: {:#?}", &options);
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
    tracing::info!("normalized_options: {:#?}", &compiler_options);

    let rspack = rspack::rspack(compiler_options, vec![]);

    let id = COMPILER_ID.fetch_add(1, Ordering::SeqCst);
    unsafe { COMPILERS.insert_if_vacant(id, rspack) }?;

    Ok(Self { id })
  }

  /// Build with the given option passed to the constructor
  ///
  /// Warning:
  /// Calling this method recursively might cause a deadlock.
  #[napi(
    js_name = "unsafe_build",
    ts_args_type = "callback: (err: null | Error, result: StatsCompilation) => void"
  )]
  pub fn build(&self, env: Env, f: JsFunction) -> Result<()> {
    let handle_build = |compiler: &mut _| {
      // Safety: compiler is stored in a global hashmap, so it's guaranteed to be alive.
      let compiler = unsafe {
        std::mem::transmute::<&'_ mut rspack::Compiler, &'static mut rspack::Compiler>(compiler)
      };

      callbackify(env, f, async move {
        let rspack_stats = compiler
          .build()
          .await
          .map_err(|e| Error::new(napi::Status::GenericFailure, format!("{}", e)))?;

        let stats: StatsCompilation = rspack_stats.to_description().into();
        if stats.errors.is_empty() {
          tracing::info!("build success");
        } else {
          tracing::info!("build failed");
        }

        Ok(stats)
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
    ts_args_type = "callback: (err: null | Error, result: Record<string, {content: string, kind: number}>) => void"
  )]
  pub fn rebuild(
    &self,
    env: Env,
    changed_files: Vec<String>,
    removed_files: Vec<String>,
    f: JsFunction,
  ) -> Result<()> {
    let handle_rebuild = |compiler: &mut _| {
      // Safety: compiler is stored in a global hashmap, so it's guaranteed to be alive.
      let compiler = unsafe {
        std::mem::transmute::<&'_ mut rspack::Compiler, &'static mut rspack::Compiler>(compiler)
      };

      callbackify(env, f, async move {
        let (diff, stats) = compiler
          .rebuild(
            HashSet::from_iter(changed_files.into_iter()),
            HashSet::from_iter(removed_files.into_iter()),
          )
          .await
          .map_err(|e| Error::new(napi::Status::GenericFailure, format!("{}", e)))?;

        let diff_stats: HashMap<String, DiffStat> = diff
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
        let stats: StatsCompilation = stats.to_description().into();
        // let stats: Stats = _rspack_stats.into();
        let rebuild_result = RebuildResult {
          diff: diff_stats,
          stats: stats,
        };
        tracing::info!("rebuild success");
        Ok(rebuild_result)
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
    let handle_last_compilation = |compiler: &mut _| {
      // Safety: compiler is stored in a global hashmap, and compilation is only available in the callback of this function, so it is safe to cast to a static lifetime. See more in the warning part of this method.
      let compiler = unsafe {
        std::mem::transmute::<&'_ mut rspack::Compiler, &'static mut rspack::Compiler>(compiler)
      };
      f(JsCompilation::from_compilation(unsafe {
        Pin::new_unchecked(&mut compiler.compilation)
      }))
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
