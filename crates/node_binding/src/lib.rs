#![recursion_limit = "256"]
#![feature(try_blocks)]
#[macro_use]
extern crate napi_derive;

#[macro_use]
extern crate rspack_binding_macros;

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::atomic::{AtomicU32, Ordering};

use dashmap::DashMap;
use napi::bindgen_prelude::*;
use once_cell::sync::Lazy;

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
    let mut inner = self.0.get_mut(key).ok_or_else(|| {
      napi::Error::from_reason(format!(
        "Failed to find key {key} for single-threaded hashmap",
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
    let inner = self.0.get(key).ok_or_else(|| {
      napi::Error::from_reason(format!(
        "Failed to find key {key} for single-threaded hashmap",
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
  pub fn new(env: Env, mut options: RawOptions, js_hooks: Option<JsHooks>) -> Result<Self> {
    init_custom_trace_subscriber(env)?;
    // rspack_tracing::enable_tracing_by_env();
    Self::prepare_environment(&env, &mut options);
    tracing::info!("raw_options: {:#?}", &options);

    let compiler_options = {
      let mut options =
        normalize_bundle_options(options).map_err(|e| Error::from_reason(format!("{e}")))?;

      if let Some(hooks_adapter) = js_hooks
        .map(|js_hooks| JsHooksAdapter::from_js_hooks(env, js_hooks))
        .transpose()?
      {
        options
          .plugins
          .push(Box::new(hooks_adapter) as Box<dyn rspack_core::Plugin>);
      };

      options
        .module
        .rules
        .iter_mut()
        .try_for_each(|rule| {
          rule.r#use.iter_mut().try_for_each(|loader| {
            let casted = loader.as_any_mut();
            if let Some(adapter) = casted.downcast_mut::<JsLoaderAdapter>() {
              adapter.unref(&env)
            } else {
              Ok(())
            }
          })
        })
        .map_err(|e| Error::from_reason(format!("failed to unref tsfn {e:?}")))?;

      options
    };

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
    catch_unwind,
    js_name = "unsafe_build",
    ts_args_type = "callback: (err: null | Error) => void"
  )]
  pub fn build(&self, env: Env, f: JsFunction) -> Result<()> {
    let handle_build = |compiler: &mut _| {
      // Safety: compiler is stored in a global hashmap, so it's guaranteed to be alive.
      let compiler = unsafe {
        std::mem::transmute::<&'_ mut rspack::Compiler, &'static mut rspack::Compiler>(compiler)
      };

      callbackify(env, f, async move {
        compiler
          .build()
          .await
          .map_err(|e| Error::new(napi::Status::GenericFailure, format!("{e}")))?;
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
    catch_unwind,
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
    let handle_rebuild = |compiler: &mut _| {
      // Safety: compiler is stored in a global hashmap, so it's guaranteed to be alive.
      let compiler = unsafe {
        std::mem::transmute::<&'_ mut rspack::Compiler, &'static mut rspack::Compiler>(compiler)
      };

      callbackify(env, f, async move {
        compiler
          .rebuild(
            HashSet::from_iter(changed_files.into_iter()),
            HashSet::from_iter(removed_files.into_iter()),
          )
          .await
          .map_err(|e| Error::new(napi::Status::GenericFailure, format!("{e:?}")))?;
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
  #[napi(catch_unwind, js_name = "unsafe_last_compilation")]
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
  #[napi(catch_unwind, js_name = "unsafe_drop")]
  pub fn drop(&self) -> Result<()> {
    unsafe { COMPILERS.remove(&self.id) };

    Ok(())
  }
}

impl Rspack {
  fn prepare_environment(env: &Env, options: &mut RawOptions) {
    NAPI_ENV.with(|napi_env| *napi_env.borrow_mut() = Some(env.raw()));

    if let Some(module) = options.module.as_mut() {
      for rule in &mut module.rules {
        if let Some(uses) = rule.r#use.as_mut() {
          for item in uses {
            if let Some(loader) = item.loader.as_ref() {
              // let (env_ptr, loader_ptr) = unsafe { (env.raw(), loader.raw()) };
              if let Ok(display_name) = get_named_property_value_string(*env, loader, "displayName")
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

#[napi::module_init]
fn init() {
  use std::panic::set_hook;

  use backtrace::Backtrace;

  set_hook(Box::new(|panic_info| {
    let backtrace = Backtrace::new();
    println!("Panic: {panic_info:?}\nBacktrace: \n{backtrace:?}");
  }));
}
