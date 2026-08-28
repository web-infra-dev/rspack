use std::{
  cell::RefCell,
  fmt::Debug,
  rc::{Rc, Weak as RcWeak},
  sync::{Arc, Weak as SyncWeak},
};

use futures::future::BoxFuture;
use napi::{
  Either,
  bindgen_prelude::{FnArgs, FromNapiValue, Function, FunctionRef, TypeName},
};
use rspack_core::{Filename, FilenameFn, LocalFilenameFn, PathData, PublicPath};

use crate::{
  asset::AssetInfo, compiler_scoped_tsfn::CompilerScopedTsFnHandle as ThreadsafeFunction,
  path_data::JsPathData,
};

thread_local! {
  static CURRENT_FILENAME_TSFN_MANAGER: RefCell<Option<RcWeak<CompilerScopedFilenameTsFnManagerInner>>> = Default::default();
}

type RawFilenameValue =
  Either<String, ThreadsafeFunction<FnArgs<(JsPathData, Option<AssetInfo>)>, String>>;
type JsFilenameFnArgs = FnArgs<(JsPathData, Option<AssetInfo>)>;
pub(crate) type JsFilenameFunctionRef = FunctionRef<JsFilenameFnArgs, String>;

pub(crate) type JsFilenameFunction<'env> = Function<'env, JsFilenameFnArgs, String>;

#[derive(Debug)]
enum FilenameValue {
  Template(String),
  Function(Filename),
}

struct CompilerScopedFilenameTsFnReference {
  filename_fn: SyncWeak<dyn FilenameFn>,
  function_ref: JsFilenameFunctionRef,
}

#[derive(Default)]
struct CompilerScopedFilenameTsFnManagerInner {
  references: RefCell<Vec<CompilerScopedFilenameTsFnReference>>,
}

/// Maintains the compiler-scoped identity mapping between JavaScript filename
/// functions and their Rust `FilenameFn` counterparts.
///
/// # Why the mapping exists
///
/// Converting a JavaScript filename function creates two representations of the
/// same callback:
///
/// - a `CompilerScopedTsFnHandle`, wrapped by `ThreadSafeFilenameFn` and stored
///   in Rust as an `Arc<dyn FilenameFn>`, so the compiler can render filenames;
/// - a `FunctionRef` to the original JavaScript function, so binding APIs such
///   as `Chunk::filename_template` can return that exact function object.
///
/// A Rust `Filename` cannot reconstruct the original JavaScript function by
/// itself. Creating a new JavaScript wrapper around the Rust `FilenameFn` would
/// introduce a nested boundary crossing:
///
/// ```text
/// JavaScript wrapper -> Rust FilenameFn -> TSFN -> original JavaScript function
/// ```
///
/// Besides adding calls and error/lifetime handling, that wrapper would have a
/// different JavaScript identity. Keeping this mapping lets the binding borrow
/// back the original function directly, preserving identity and avoiding the
/// nested JavaScript -> Rust -> JavaScript call path.
///
/// # Filename sources
///
/// Filename functions are not limited to initial compiler options. They can be
/// introduced through:
///
/// - compiler options such as entry and output filename settings;
/// - built-in plugin options, including entry and split-chunks configuration;
/// - compilation-time APIs such as `addEntry` and `addInclude`.
///
/// Consequently, every conversion path that can create a compiler-owned
/// `JsFilename` must run inside `CompilerScopedFilenameTsFnManager::scope`.
/// The thread-local scope supplies the owning manager because `FromNapiValue`
/// cannot receive the compiler as explicit conversion context.
///
/// # Ownership and cleanup
///
/// Each mapping owns the JavaScript `FunctionRef`, but holds only a `Weak` to
/// the Rust `FilenameFn`. A strong Rust reference here would keep filenames
/// alive after their compilation data had been discarded and could retain the
/// TSFN and its JavaScript closure indefinitely.
///
/// Once the last real Rust owner drops a filename function, `sweep` observes
/// that its `Weak` can no longer be upgraded and removes the mapping, which also
/// drops the corresponding `FunctionRef`. Sweeping occurs after each
/// compilation and lazily before lookup. `release` removes every remaining
/// mapping when the compiler is closed.
#[derive(Clone, Default)]
pub(crate) struct CompilerScopedFilenameTsFnManager {
  inner: Rc<CompilerScopedFilenameTsFnManagerInner>,
}

impl CompilerScopedFilenameTsFnManager {
  pub fn scope<R>(&self, f: impl FnOnce() -> R) -> R {
    struct ManagerGuard(Option<RcWeak<CompilerScopedFilenameTsFnManagerInner>>);

    impl Drop for ManagerGuard {
      fn drop(&mut self) {
        CURRENT_FILENAME_TSFN_MANAGER.with(|current| {
          current.replace(self.0.take());
        });
      }
    }

    let previous = CURRENT_FILENAME_TSFN_MANAGER
      .with(|current| current.replace(Some(Rc::downgrade(&self.inner))));
    let _guard = ManagerGuard(previous);
    f()
  }

  fn current_context() -> Option<Self> {
    CURRENT_FILENAME_TSFN_MANAGER
      .with(|current| current.borrow().clone())
      .and_then(|manager| manager.upgrade().map(|inner| Self { inner }))
  }

  fn register(&self, filename_fn: SyncWeak<dyn FilenameFn>, function_ref: JsFilenameFunctionRef) {
    self
      .inner
      .references
      .borrow_mut()
      .push(CompilerScopedFilenameTsFnReference {
        filename_fn,
        function_ref,
      });
  }

  pub fn get<'env>(
    &self,
    filename: &Filename,
    env: &'env napi::Env,
  ) -> napi::Result<Option<JsFilenameFunction<'env>>> {
    self.sweep();
    let references = self.inner.references.borrow();

    for reference in references.iter() {
      let Some(filename_fn) = reference.filename_fn.upgrade() else {
        continue;
      };
      if Filename::from(filename_fn).eq(filename) {
        return reference.function_ref.borrow_back(env).map(Some);
      }
    }

    Ok(None)
  }

  pub fn sweep(&self) {
    self
      .inner
      .references
      .borrow_mut()
      .retain(|reference| reference.filename_fn.strong_count() > 0);
  }

  pub fn release(&self) {
    self.inner.references.borrow_mut().clear();
  }
}

/// A js filename value. Either a string or a function
#[derive(Debug)]
pub struct JsFilename {
  filename: FilenameValue,
}

impl FromNapiValue for JsFilename {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    unsafe {
      let filename = match RawFilenameValue::from_napi_value(env, napi_val)? {
        Either::A(template) => FilenameValue::Template(template),
        Either::B(f) => {
          let filename_fn = Arc::new(ThreadSafeFilenameFn(Arc::new(
            move |path_data, asset_info| {
              let f = f.clone();
              Box::pin(async move { f.call_with_sync((path_data, asset_info).into()).await })
            },
          ))) as Arc<dyn FilenameFn>;

          if let Some(manager) = CompilerScopedFilenameTsFnManager::current_context() {
            let function_ref = JsFilenameFunctionRef::from_napi_value(env, napi_val)?;
            manager.register(Arc::downgrade(&filename_fn), function_ref);
          }

          FilenameValue::Function(Filename::from(filename_fn))
        }
      };
      Ok(Self { filename })
    }
  }
}

impl TypeName for JsFilename {
  fn type_name() -> &'static str {
    "JsFilename"
  }

  fn value_type() -> napi::ValueType {
    napi::ValueType::Unknown
  }
}

impl From<JsFilename> for Filename {
  fn from(value: JsFilename) -> Self {
    match value.filename {
      FilenameValue::Template(template) => Filename::from(template),
      FilenameValue::Function(filename) => filename,
    }
  }
}

impl From<JsFilename> for PublicPath {
  fn from(value: JsFilename) -> Self {
    match value.filename {
      FilenameValue::Template(template) => template.into(),
      FilenameValue::Function(filename) => PublicPath::Filename(filename),
    }
  }
}

pub type FilenameTsfn = Arc<
  dyn Fn(JsPathData, Option<AssetInfo>) -> BoxFuture<'static, rspack_error::Result<String>>
    + Sync
    + Send,
>;

/// Wrapper of a thread-safe filename js function. Implements `FilenameFn`
struct ThreadSafeFilenameFn(FilenameTsfn);

impl Debug for ThreadSafeFilenameFn {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ThreadSafeFilenameFn").finish()
  }
}

#[async_trait::async_trait]
impl LocalFilenameFn for ThreadSafeFilenameFn {
  async fn call(
    &self,
    path_data: &PathData,
    asset_info: Option<&rspack_core::AssetInfo>,
  ) -> rspack_error::Result<String> {
    (self.0)(
      JsPathData::from_path_data(*path_data),
      asset_info.cloned().map(AssetInfo::from),
    )
    .await
  }
}
impl FilenameFn for ThreadSafeFilenameFn {}
