use std::{
  cell::{Cell, RefCell},
  fmt,
  rc::{Rc, Weak},
  sync::Arc,
};

use atomic_refcell::AtomicRefCell;
use napi::{
  Env, JsValue, Status, ValueType,
  bindgen_prelude::{
    FromNapiValue, JsObjectValue, JsValuesTupleIntoVec, Promise, TypeName, ValidateNapiValue,
  },
  sys,
};
use rspack_error::error;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use slotmap::{DefaultKey, SlotMap};

const COMPILER_CLOSED_MESSAGE: &str = "Rspack compiler has already been closed by `compiler.close()`. Do not call Rspack compiler APIs after close; create a new compiler instead.";

thread_local! {
  // `FromNapiValue` does not accept extra context, so constructor-time parsing uses a
  // thread-local scope to tell nested callback conversions which compiler-local manager
  // should own their TSFN cleanup.
  static CURRENT_COMPILER_SCOPED_TS_FN_CONTEXT: RefCell<Option<CompilerScopedTsFnContext>> = Default::default();
}

#[derive(Clone)]
struct CompilerScopedTsFnContext {
  manager: Weak<CompilerScopedTsFnManagerInner>,
}

type Releaser = Box<dyn FnOnce()>;

#[derive(Default)]
struct CompilerScopedTsFnManagerInner {
  releasers: RefCell<SlotMap<DefaultKey, Releaser>>,
  closed: Cell<bool>,
}

impl CompilerScopedTsFnManagerInner {
  fn release(&self) {
    if self.closed.replace(true) {
      return;
    }
    // Finalizers may unregister callbacks. Finish borrowing the registry before
    // releasing TSFNs, and make repeated close/drop calls harmless.
    let releasers = std::mem::take(&mut *self.releasers.borrow_mut());
    for (_, releaser) in releasers {
      releaser();
    }
  }
}

impl Drop for CompilerScopedTsFnManagerInner {
  fn drop(&mut self) {
    self.release();
  }
}

pub(crate) struct CompilerScopedTsFnManager {
  inner: Rc<CompilerScopedTsFnManagerInner>,
}

impl CompilerScopedTsFnManager {
  pub fn new() -> Self {
    Self {
      inner: Rc::new(Default::default()),
    }
  }

  pub fn scope<R>(&self, f: impl FnOnce() -> R) -> R {
    struct ManagerGuard(Option<CompilerScopedTsFnContext>);

    impl Drop for ManagerGuard {
      fn drop(&mut self) {
        CURRENT_COMPILER_SCOPED_TS_FN_CONTEXT.with(|current| {
          current.replace(self.0.take());
        });
      }
    }

    let previous = CURRENT_COMPILER_SCOPED_TS_FN_CONTEXT.with(|current| {
      current.replace(Some(CompilerScopedTsFnContext {
        manager: Rc::downgrade(&self.inner),
      }))
    });
    let _guard = ManagerGuard(previous);
    f()
  }

  pub fn release(&self) {
    self.inner.release();
  }

  fn current_context() -> Option<Self> {
    CURRENT_COMPILER_SCOPED_TS_FN_CONTEXT
      .with(|current| current.borrow().clone())
      .and_then(|context| context.manager.upgrade().map(|inner| Self { inner }))
  }

  fn register_releaser(&self, releaser: Releaser) -> napi::Result<impl FnOnce() + 'static> {
    if self.inner.closed.get() {
      return Err(napi::Error::new(
        Status::GenericFailure,
        COMPILER_CLOSED_MESSAGE,
      ));
    }
    let key = self.inner.releasers.borrow_mut().insert(releaser);

    let manager = Rc::downgrade(&self.inner);
    Ok(move || {
      // TSFN finalization (including destruction of this Rc::Weak) runs on the
      // owning JS thread, so the registry needs no cross-thread synchronization.
      if let Some(manager) = manager.upgrade() {
        let releaser = manager.releasers.borrow_mut().remove(key);
        drop(releaser);
      }
    })
  }
}

type SharedThreadsafeFunction<T, R> = Arc<AtomicRefCell<Option<ThreadsafeFunction<T, R>>>>;

#[cfg(debug_assertions)]
const CALLBACK_SOURCE_PREVIEW_LIMIT: usize = 160;

#[cfg(debug_assertions)]
fn truncate_source_preview(source: &str) -> String {
  let compact = source.split_whitespace().collect::<Vec<_>>().join(" ");
  let mut preview = compact
    .chars()
    .take(CALLBACK_SOURCE_PREVIEW_LIMIT)
    .collect::<String>();
  if compact.chars().count() > CALLBACK_SOURCE_PREVIEW_LIMIT {
    preview.push_str("...");
  }
  preview
}

#[cfg(debug_assertions)]
fn format_js_function(env: sys::napi_env, napi_val: sys::napi_value) -> String {
  let env = Env::from_raw(env);
  let function = napi::bindgen_prelude::Object::from_raw(env.raw(), napi_val);
  if let Some(name) = function
    .get_named_property::<String>("name")
    .ok()
    .filter(|name| !name.is_empty())
  {
    return format!("function `{name}`");
  }

  function
    .coerce_to_string()
    .ok()
    .and_then(|source| source.into_utf8().ok())
    .and_then(|source| source.into_owned().ok())
    .filter(|source| !source.is_empty())
    .map_or_else(
      || "anonymous function".to_string(),
      |source| {
        format!(
          "anonymous function with source preview: {}",
          truncate_source_preview(&source)
        )
      },
    )
}

// Consumers own the TSFN slot; the compiler only weakly registers it for close/drop
// cleanup. The last consumer can release its callback before the compiler closes.
pub struct CompilerScopedTsFnHandle<T: 'static + JsValuesTupleIntoVec, R> {
  active_tsfn: SharedThreadsafeFunction<T, R>,
}

impl<T: 'static + JsValuesTupleIntoVec, R> Clone for CompilerScopedTsFnHandle<T, R> {
  fn clone(&self) -> Self {
    Self {
      active_tsfn: self.active_tsfn.clone(),
    }
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R> fmt::Debug for CompilerScopedTsFnHandle<T, R> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("CompilerScopedTsFnHandle")
      .finish_non_exhaustive()
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R> CompilerScopedTsFnHandle<T, R> {
  fn expect_active_tsfn(&self) -> rspack_error::Result<ThreadsafeFunction<T, R>> {
    self
      .active_tsfn
      .borrow()
      .clone()
      .ok_or_else(|| error!("{COMPILER_CLOSED_MESSAGE}"))
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R: 'static + FromNapiValue> CompilerScopedTsFnHandle<T, R> {
  pub async fn call_with_sync(&self, value: T) -> rspack_error::Result<R> {
    self.expect_active_tsfn()?.call_with_sync(value).await
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R: 'static + FromNapiValue>
  CompilerScopedTsFnHandle<T, Promise<R>>
{
  pub async fn call_with_promise(&self, value: T) -> rspack_error::Result<R> {
    self.expect_active_tsfn()?.call_with_promise(value).await
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R: 'static> FromNapiValue
  for CompilerScopedTsFnHandle<T, R>
{
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> napi::Result<Self> {
    let active_tsfn = Arc::new(AtomicRefCell::new(None));

    if let Some(manager) = CompilerScopedTsFnManager::current_context() {
      let active_tsfn_for_releaser = Arc::downgrade(&active_tsfn);
      let finalize = manager.register_releaser(Box::new(move || {
        if let Some(active_tsfn) = active_tsfn_for_releaser.upgrade() {
          let tsfn = active_tsfn.borrow_mut().take();
          drop(tsfn);
        }
      }))?;
      let tsfn =
        unsafe { ThreadsafeFunction::from_napi_value_with_finalize(env, napi_val, finalize) }?;
      *active_tsfn.borrow_mut() = Some(tsfn);
    } else {
      // Callbacks parsed outside a compiler construction scope fall back to eager TSFN behavior.
      // This should not happen in normal usage - report an error in debug builds to catch issues early.
      #[cfg(debug_assertions)]
      {
        let function_desc = format_js_function(env, napi_val);
        return Err(napi::Error::new(
          Status::GenericFailure,
          format!(
            "Compiler-scoped callback {function_desc} was parsed outside a compiler construction scope",
          ),
        ));
      }
      #[cfg(not(debug_assertions))]
      {
        let tsfn = unsafe { ThreadsafeFunction::from_napi_value(env, napi_val) }?;
        *active_tsfn.borrow_mut() = Some(tsfn);
      }
    }

    Ok(Self { active_tsfn })
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R> ValidateNapiValue for CompilerScopedTsFnHandle<T, R> {}

impl<T: 'static + JsValuesTupleIntoVec, R> TypeName for CompilerScopedTsFnHandle<T, R> {
  fn type_name() -> &'static str {
    "ThreadsafeFunction"
  }

  fn value_type() -> ValueType {
    ValueType::Function
  }
}
