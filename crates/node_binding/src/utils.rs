use std::cell::Cell;
use std::ffi::CStr;
use std::io::Write;
use std::ptr;

use dashmap::DashMap;
use futures::Future;
use napi::bindgen_prelude::*;
use napi::{check_status, Env, Error, JsFunction, JsUnknown, NapiRaw, Result};
use napi_derive::napi;
use once_cell::sync::OnceCell;
use rspack_error::CatchUnwindFuture;
use rspack_napi_shared::threadsafe_function::{
  ThreadSafeContext, ThreadsafeFunction, ThreadsafeFunctionCallMode,
};
use rspack_tracing::chrome::FlushGuard;

static CUSTOM_TRACE_SUBSCRIBER: OnceCell<bool> = OnceCell::new();

thread_local! {
  static CUSTOM_TRACE_CHROME_FLUSH_GUARD: Cell<Option<FlushGuard>> = Cell::new(None);
}

/// Try to resolve the string value of a given named property
#[allow(unused)]
pub(crate) fn get_named_property_value_string<T: NapiRaw>(
  env: Env,
  object: T,
  property_name: &str,
) -> Result<String> {
  let mut bytes_with_nul: Vec<u8> = Vec::with_capacity(property_name.len() + 1);

  write!(&mut bytes_with_nul, "{property_name}")?;
  write!(&mut bytes_with_nul, "\0")?;

  let mut value_ptr = ptr::null_mut();

  check_status!(
    unsafe {
      napi_sys::napi_get_named_property(
        env.raw(),
        object.raw(),
        CStr::from_bytes_with_nul_unchecked(&bytes_with_nul).as_ptr(),
        &mut value_ptr,
      )
    },
    "failed to get the value"
  )?;

  let mut str_len = 0;
  check_status!(
    unsafe {
      napi_sys::napi_get_value_string_utf8(env.raw(), value_ptr, ptr::null_mut(), 0, &mut str_len)
    },
    "failed to get the value"
  )?;

  str_len += 1;
  let mut buf = Vec::with_capacity(str_len);
  let mut copied_len = 0;

  check_status!(
    unsafe {
      napi_sys::napi_get_value_string_utf8(
        env.raw(),
        value_ptr,
        buf.as_mut_ptr(),
        str_len,
        &mut copied_len,
      )
    },
    "failed to get the value"
  )?;

  // Vec<i8> -> Vec<u8> See: https://stackoverflow.com/questions/59707349/cast-vector-of-i8-to-vector-of-u8-in-rust
  let mut buf = std::mem::ManuallyDrop::new(buf);

  let buf = unsafe { Vec::from_raw_parts(buf.as_mut_ptr() as *mut u8, copied_len, copied_len) };

  String::from_utf8(buf).map_err(|_| Error::from_reason("failed to get property"))
}

/**
 * Some code is modified based on
 * https://github.com/swc-project/swc/blob/d1d0607158ab40463d1b123fed52cc526eba8385/bindings/binding_core_node/src/util.rs#L29-L58
 * Apache-2.0 licensed
 * Author Donny/강동윤
 * Copyright (c)
 */
#[napi(catch_unwind)]
pub fn init_custom_trace_subscriber(
  mut env: Env,
  // trace_out_file_path: Option<String>,
) {
  CUSTOM_TRACE_SUBSCRIBER.get_or_init(|| {
    let layer = std::env::var("layer").unwrap_or("logger".to_string());

    CUSTOM_TRACE_CHROME_FLUSH_GUARD.with(|v| {
      let mut guard = v.take();
      if guard.is_none() {
        guard = match layer.as_str() {
          "chrome" => rspack_tracing::enable_tracing_by_env_with_chrome_layer(),
          "logger" => {
            rspack_tracing::enable_tracing_by_env();
            None
          }
          _ => panic!("not supported layer type:{layer}"),
        }
      }
      v.set(guard);
    });

    env
      .add_env_cleanup_hook((), |_| {
        cleanup_custom_trace_subscriber();
      })
      .expect("Should able to initialize cleanup for custom trace subscriber");
    true
  });
}

#[napi]
pub fn cleanup_custom_trace_subscriber() {
  CUSTOM_TRACE_CHROME_FLUSH_GUARD.with(|v| {
    if let Some(g) = v.take() {
      g.flush();
      // drop
    }
  })
}

pub fn callbackify<R, F>(env: Env, f: JsFunction, fut: F) -> Result<()>
where
  R: 'static + ToNapiValue,
  F: 'static + Send + Future<Output = Result<R>>,
{
  let ptr = unsafe { f.raw() };

  let tsfn = ThreadsafeFunction::<Result<R>, ()>::create(env.raw(), ptr, 0, |ctx| {
    let ThreadSafeContext {
      value,
      env,
      callback,
      ..
    } = ctx;

    let argv = match value {
      Ok(value) => {
        let val = unsafe { R::to_napi_value(env.raw(), value)? };
        let js_value = unsafe { JsUnknown::from_napi_value(env.raw(), val)? };
        vec![env.get_null()?.into_unknown(), js_value]
      }
      Err(err) => {
        vec![JsError::from(err).into_unknown(env)]
      }
    };

    callback.call(None, &argv)?;

    Ok(())
  })?;

  napi::bindgen_prelude::spawn(async move {
    let fut = CatchUnwindFuture::create(fut);
    let res = fut.await;
    match res {
      Ok(result) => {
        tsfn
          .call(result, ThreadsafeFunctionCallMode::NonBlocking)
          .expect("Failed to call JS callback");
      }
      Err(e) => {
        tsfn
          .call(
            Err(Error::from_reason(format!("{e}"))),
            ThreadsafeFunctionCallMode::NonBlocking,
          )
          .expect("Failed to send panic info");
      }
    }
  });

  Ok(())
}

// **Note** that Node's main thread and the worker thread share the same binding context. Using `Mutex<HashMap>` would cause deadlocks if multiple compilers exist.
pub(crate) struct SingleThreadedHashMap<K, V>(DashMap<K, V>);

impl<K, V> SingleThreadedHashMap<K, V>
where
  K: Eq + std::hash::Hash + std::fmt::Display,
{
  /// Acquire a mutable reference to the inner hashmap.
  ///
  /// Safety: Mutable reference can almost let you do anything you want, this is intended to be used from the thread where the map was created.
  pub(crate) unsafe fn borrow_mut<F, R>(&self, key: &K, f: F) -> Result<R>
  where
    F: FnOnce(&mut V) -> Result<R>,
  {
    let mut inner = self.0.get_mut(key).ok_or_else(|| {
      napi::Error::from_reason(format!(
        "Failed to find key {key} for single-threaded hashmap",
      ))
    })?;

    f(&mut inner)
  }

  /// Acquire a shared reference to the inner hashmap.
  ///
  /// Safety: It's not thread-safe if a value is not safe to modify cross thread boundary, so this is intended to be used from the thread where the map was created.
  #[allow(unused)]
  pub(crate) unsafe fn borrow<F, R>(&self, key: &K, f: F) -> Result<R>
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
  pub(crate) unsafe fn insert_if_vacant(&self, key: K, value: V) -> Result<()> {
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
  pub(crate) unsafe fn remove(&self, key: &K) -> Option<V> {
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
