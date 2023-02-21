#![recursion_limit = "256"]
#![feature(try_blocks)]
#[macro_use]
extern crate napi_derive;

#[macro_use]
extern crate rspack_binding_macros;

use std::collections::HashSet;
use std::sync::atomic::{AtomicU32, Ordering};

use dashmap::DashMap;
use napi::bindgen_prelude::*;
use once_cell::sync::Lazy;
use rspack_core::PluginExt;
use rspack_fs_node::{AsyncNodeWritableFileSystem, ThreadsafeNodeFS};

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

static COMPILERS: Lazy<
  SingleThreadedHashMap<CompilerId, rspack_core::Compiler<AsyncNodeWritableFileSystem>>,
> = Lazy::new(Default::default);

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
    options: RawOptions,
    js_hooks: Option<JsHooks>,
    output_filesystem: ThreadsafeNodeFS,
  ) -> Result<Self> {
    init_custom_trace_subscriber(env)?;
    // rspack_tracing::enable_tracing_by_env();
    Self::prepare_environment(&env);
    tracing::info!("raw_options: {:#?}", &options);

    let mut plugins = Vec::new();
    if let Some(js_hooks) = js_hooks {
      plugins.push(JsHooksAdapter::from_js_hooks(env, js_hooks)?.boxed());
    }

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
      let compiler: &'static mut rspack_core::Compiler<AsyncNodeWritableFileSystem> =
        unsafe { std::mem::transmute::<&'_ mut _, &'static mut _>(compiler) };

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
      let compiler: &'static mut rspack_core::Compiler<AsyncNodeWritableFileSystem> =
        unsafe { std::mem::transmute::<&'_ mut _, &'static mut _>(compiler) };

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
      let compiler: &'static mut rspack_core::Compiler<AsyncNodeWritableFileSystem> =
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
  #[napi(catch_unwind, js_name = "unsafe_drop")]
  pub fn drop(&self) -> Result<()> {
    unsafe { COMPILERS.remove(&self.id) };

    Ok(())
  }
}

impl Rspack {
  fn prepare_environment(env: &Env) {
    NAPI_ENV.with(|napi_env| *napi_env.borrow_mut() = Some(env.raw()));
  }
}
