use std::{
  cell::RefCell,
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
use rspack_napi::{WeakRef, threadsafe_function::ThreadsafeFunction};

thread_local! {
  // `FromNapiValue` does not accept extra context, so constructor-time parsing uses a
  // thread-local scope to tell nested callback conversions which compiler-local manager
  // should own their registration.
  static CURRENT_COMPILATION_SCOPED_TS_FN_CONTEXT: RefCell<Option<CompilationScopedTsFnContext>> = Default::default();
}

#[derive(Clone)]
struct CompilationScopedTsFnContext {
  manager: Weak<CompilationScopedTsFnManagerInner>,
}

trait CompilationScopedTsFnRegistrationOps {
  fn activate(&self, env: Env) -> napi::Result<()>;
  fn release(&self);
}

#[derive(Default)]
struct CompilationScopedTsFnManagerInner {
  registrations: RefCell<Vec<Rc<dyn CompilationScopedTsFnRegistrationOps>>>,
}

#[derive(Clone)]
pub(crate) struct CompilationScopedTsFnManager {
  inner: Rc<CompilationScopedTsFnManagerInner>,
}

impl CompilationScopedTsFnManager {
  pub fn new() -> Self {
    Self {
      inner: Rc::new(Default::default()),
    }
  }

  pub fn scope<R>(&self, f: impl FnOnce() -> R) -> R {
    struct ManagerGuard(Option<CompilationScopedTsFnContext>);

    impl Drop for ManagerGuard {
      fn drop(&mut self) {
        CURRENT_COMPILATION_SCOPED_TS_FN_CONTEXT.with(|current| {
          current.replace(self.0.take());
        });
      }
    }

    let previous = CURRENT_COMPILATION_SCOPED_TS_FN_CONTEXT.with(|current| {
      current.replace(Some(CompilationScopedTsFnContext {
        manager: Rc::downgrade(&self.inner),
      }))
    });
    let _guard = ManagerGuard(previous);
    f()
  }

  pub fn activate(&self, env: Env) -> napi::Result<()> {
    let raw_env = env.raw();
    let registrations = self.registrations();
    for registration in &registrations {
      if let Err(err) = registration.activate(Env::from_raw(raw_env)) {
        for registration in registrations {
          registration.release();
        }
        return Err(err);
      }
    }

    Ok(())
  }

  pub fn release(&self) {
    for registration in self.registrations() {
      registration.release();
    }
  }

  fn current_context() -> Option<Self> {
    CURRENT_COMPILATION_SCOPED_TS_FN_CONTEXT
      .with(|current| current.borrow().clone())
      .and_then(|context| context.manager.upgrade().map(|inner| Self { inner }))
  }

  fn register<T>(&self, registration: Rc<T>)
  where
    T: CompilationScopedTsFnRegistrationOps + 'static,
  {
    let registration: Rc<dyn CompilationScopedTsFnRegistrationOps> = registration;
    self.inner.registrations.borrow_mut().push(registration);
  }

  fn registrations(&self) -> Vec<Rc<dyn CompilationScopedTsFnRegistrationOps>> {
    self.inner.registrations.borrow().clone()
  }
}

type ActiveThreadsafeFunction<T, R> = Arc<AtomicRefCell<Option<ThreadsafeFunction<T, R>>>>;

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

// A registration keeps the original JS callback registered fn alive only as a weak reference.
// Each build/rebuild activates a fresh TSFN into the shared slot and the build callback
// finalizer clears that slot again.
struct CompilationScopedTsFnRegistration<T: 'static + JsValuesTupleIntoVec, R> {
  // A weak reference to the JS callback function. The actual function is kept alive on the JS side:
  // - For options callbacks: stored in `#rawOptions` in `packages/rspack/src/Compiler.ts`
  // - For hook registration callbacks: stored in `#registers` in `packages/rspack/src/Compiler.ts`
  // This ensures the function's lifetime matches the Compiler's lifetime.
  registered_fn: WeakRef,
  active_tsfn: ActiveThreadsafeFunction<T, R>,
  #[cfg(debug_assertions)]
  function_desc: String,
}

impl<T: 'static + JsValuesTupleIntoVec, R: 'static> CompilationScopedTsFnRegistrationOps
  for CompilationScopedTsFnRegistration<T, R>
{
  fn activate(&self, env: Env) -> napi::Result<()> {
    let function = self.registered_fn.as_object(&env).map_err(|_| {
      #[cfg(debug_assertions)]
      let message = format!(
        "Compilation-scoped JS callback {} has been garbage collected before activation",
        self.function_desc
      );
      #[cfg(not(debug_assertions))]
      let message = "Compilation-scoped JS callback has been garbage collected before activation";
      napi::Error::new(Status::GenericFailure, message)
    })?;
    let tsfn = unsafe { ThreadsafeFunction::from_napi_value(env.raw(), function.raw()) }?;
    *self.active_tsfn.borrow_mut() = Some(tsfn);
    Ok(())
  }

  fn release(&self) {
    *self.active_tsfn.borrow_mut() = None;
  }
}

// Call sites only keep this handle. It never owns the JS function directly and can only
// execute while the manager has installed an active TSFN for the current compilation run.
pub struct CompilationScopedTsFnHandle<T: 'static + JsValuesTupleIntoVec, R> {
  active_tsfn: ActiveThreadsafeFunction<T, R>,
}

impl<T: 'static + JsValuesTupleIntoVec, R> Clone for CompilationScopedTsFnHandle<T, R> {
  fn clone(&self) -> Self {
    Self {
      active_tsfn: self.active_tsfn.clone(),
    }
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R> fmt::Debug for CompilationScopedTsFnHandle<T, R> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("CompilationScopedTsFnHandle")
      .finish_non_exhaustive()
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R> CompilationScopedTsFnHandle<T, R> {
  fn expect_active_tsfn(&self) -> rspack_error::Result<ThreadsafeFunction<T, R>> {
    self.active_tsfn.borrow().clone().ok_or_else(|| {
      error!("Compilation-scoped JS callback was invoked outside an active compilation session")
    })
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R: 'static + FromNapiValue>
  CompilationScopedTsFnHandle<T, R>
{
  pub async fn call_with_sync(&self, value: T) -> rspack_error::Result<R> {
    self.expect_active_tsfn()?.call_with_sync(value).await
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R: 'static + FromNapiValue>
  CompilationScopedTsFnHandle<T, Promise<R>>
{
  pub async fn call_with_promise(&self, value: T) -> rspack_error::Result<R> {
    self.expect_active_tsfn()?.call_with_promise(value).await
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R: 'static> FromNapiValue
  for CompilationScopedTsFnHandle<T, R>
{
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> napi::Result<Self> {
    let active_tsfn = Arc::new(AtomicRefCell::new(None));

    if let Some(manager) = CompilationScopedTsFnManager::current_context() {
      let mut function = napi::bindgen_prelude::Object::from_raw(env, napi_val);
      let registration = Rc::new(CompilationScopedTsFnRegistration {
        registered_fn: WeakRef::new(env, &mut function)?,
        active_tsfn: active_tsfn.clone(),
        #[cfg(debug_assertions)]
        function_desc: format_js_function(env, napi_val),
      });
      manager.register(registration);
    } else {
      // Callbacks parsed outside a compiler construction scope fall back to eager TSFN behavior.
      // This should not happen in normal usage - report an error in debug builds to catch issues early.
      #[cfg(debug_assertions)]
      {
        let function_desc = format_js_function(env, napi_val);
        return Err(napi::Error::new(
          Status::GenericFailure,
          format!(
            "Compilation-scoped callback {function_desc} was parsed outside a compiler construction scope",
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

impl<T: 'static + JsValuesTupleIntoVec, R> ValidateNapiValue for CompilationScopedTsFnHandle<T, R> {}

impl<T: 'static + JsValuesTupleIntoVec, R> TypeName for CompilationScopedTsFnHandle<T, R> {
  fn type_name() -> &'static str {
    "ThreadsafeFunction"
  }

  fn value_type() -> ValueType {
    ValueType::Function
  }
}
