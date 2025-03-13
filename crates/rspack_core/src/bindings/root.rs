use std::{
  ops::{Deref, DerefMut},
  sync::{Arc, Mutex},
  thread::{self, ThreadId},
};

use derive_more::Debug;
use napi::{
  bindgen_prelude::{Reference, ToNapiValue},
  sys::{napi_env, napi_value},
};
use rspack_napi::next_tick;

use crate::bindings;
use crate::Compilation;

// ThreadSafeReference 是对 napi::Reference<()> 的封装
// 它只能在 JS 线程中被创建，但可以在任何线程中被使用
// 当它在 JS 线程中被 drop 时，会直接释放
// 当它在非 JS 线程中被 drop 时，会被移动到JS 线程，并等待释放它
#[derive(Debug)]
struct ThreadSafeReference {
  thread_id: ThreadId,
  #[debug(skip)]
  i: Option<Reference<()>>,
}

impl ThreadSafeReference {
  pub fn new(_env: napi_env, i: Reference<()>) -> Self {
    Self {
      thread_id: thread::current().id(),
      i: Some(i),
    }
  }

  // pub unsafe fn from_value_ptr(t: *mut c_void, env: napi_env) -> napi::Result<Self> {
  //   let i = unsafe { Reference::from_value_ptr(t, env)? };
  //   Ok(Self {
  //     thread_id: thread::current().id(),
  //     i: Some(i),
  //   })
  // }
}

impl ToNapiValue for ThreadSafeReference {
  unsafe fn to_napi_value(env: napi_env, val: Self) -> napi::Result<napi_value> {
    let reference = val.i.as_ref().unwrap();
    ToNapiValue::to_napi_value(env, reference.downgrade())
  }
}

impl ToNapiValue for &ThreadSafeReference {
  unsafe fn to_napi_value(env: napi_env, val: Self) -> napi::Result<napi_value> {
    let reference = val.i.as_ref().unwrap();
    ToNapiValue::to_napi_value(env, reference.downgrade())
  }
}

impl ToNapiValue for &mut ThreadSafeReference {
  unsafe fn to_napi_value(env: napi_env, val: Self) -> napi::Result<napi_value> {
    let reference = val.i.as_ref().unwrap();
    ToNapiValue::to_napi_value(env, reference.downgrade())
  }
}

impl Drop for ThreadSafeReference {
  fn drop(&mut self) {
    if self.thread_id == thread::current().id() {
      self.i = None;
    } else {
      let i = self.i.take();
      next_tick(move || {
        drop(i);
      })
    }
  }
}

// state 表示状态，不应当能够被 clone
#[derive(Debug)]
enum Heap<T> {
  Untracked(Option<Box<T>>),
  Tracked(ThreadSafeReference),
}

#[derive(Debug)]
pub struct Root<T> {
  raw: *mut T,
  // Arc 只在 rust 线程发送给 JS 线程时使用
  state: Arc<Mutex<Heap<T>>>,
}

unsafe impl<T: Send> Send for Root<T> {}
unsafe impl<T: Sync> Sync for Root<T> {}

impl<T> Root<T> {
  pub fn new(value: T) -> Self {
    // 这里 Pin<Box<T>> 会更好
    let mut value = Box::new(value);
    Self {
      raw: &mut *value.as_mut() as *mut T,
      state: Arc::new(Mutex::new(Heap::Untracked(Some(value)))),
    }
  }
}

impl<T> Deref for Root<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.raw }
  }
}

impl<T> DerefMut for Root<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.raw }
  }
}

// 仅在 JS binding feature 下是可以 clone 的
impl<T> Clone for Root<T> {
  fn clone(&self) -> Self {
    Self {
      raw: self.raw.clone(),
      state: self.state.clone(),
    }
  }
}

impl ToNapiValue for Root<Compilation> {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    bindings::with_thread_local_allocator(|allocator| {
      let heap = &mut *val.state.lock().unwrap();
      match heap {
        Heap::Untracked(val) => {
          let reference = allocator.allocate_compilation(val.take().unwrap())?;
          let threadsafe_reference = ThreadSafeReference::new(env, reference);
          let napi_value = ToNapiValue::to_napi_value(env, &threadsafe_reference)?;
          *heap = Heap::Tracked(threadsafe_reference);
          Ok(napi_value)
        }
        Heap::Tracked(threadsafe_reference) => {
          ToNapiValue::to_napi_value(env, threadsafe_reference)
        }
      }
    })
  }
}
