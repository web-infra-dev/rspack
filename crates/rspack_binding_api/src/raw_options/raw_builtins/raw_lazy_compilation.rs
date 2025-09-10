use std::ptr::NonNull;

use napi::{
  Either,
  bindgen_prelude::{FromNapiValue, ToNapiValue, ValidateNapiValue},
};
use napi_derive::napi;
use rspack_collections::IdentifierSet;
use rspack_core::{CompilationId, CompilerId, Module, ModuleIdentifier};
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_lazy_compilation::{Backend, LazyCompilationTest, LazyCompilationTestCheck};
use rspack_regex::RspackRegex;

use crate::module::ModuleObject;

#[derive(Debug)]
pub struct RawLazyCompilationTest<F = ThreadsafeFunction<ModuleObject, Option<bool>>>(
  pub Either<RspackRegex, F>,
);

impl<F: FromNapiValue + ValidateNapiValue> FromNapiValue for RawLazyCompilationTest<F> {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    unsafe { Ok(Self(Either::from_napi_value(env, napi_val)?)) }
  }
}

impl<F: ToNapiValue> ToNapiValue for RawLazyCompilationTest<F> {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    unsafe { Either::to_napi_value(env, val.0) }
  }
}

#[derive(Debug)]
pub struct LazyCompilationTestFn {
  tsfn: ThreadsafeFunction<ModuleObject, Option<bool>>,
}

#[async_trait::async_trait]
impl LazyCompilationTestCheck for LazyCompilationTestFn {
  async fn test(
    &self,
    compiler_id: CompilerId,
    _compilation_id: CompilationId,
    m: &dyn rspack_core::Module,
  ) -> bool {
    #[allow(clippy::unwrap_used)]
    let res = self
      .tsfn
      .call_with_sync(ModuleObject::with_ptr(
        NonNull::new(m as *const dyn Module as *mut dyn Module).unwrap(),
        compiler_id,
      ))
      .await
      .expect("failed to invoke lazyCompilation.test");

    res.unwrap_or(false)
  }
}

impl From<RawLazyCompilationTest> for LazyCompilationTest<LazyCompilationTestFn> {
  fn from(value: RawLazyCompilationTest) -> Self {
    match value.0 {
      Either::A(regex) => Self::Regex(
        RspackRegex::with_flags(&regex.source, &regex.flags).unwrap_or_else(|_| {
          let msg = format!("[lazyCompilation]incorrect regex {regex:?}");
          panic!("{msg}");
        }),
      ),
      Either::B(tsfn) => Self::Fn(LazyCompilationTestFn { tsfn }),
    }
  }
}

#[napi(object)]
pub struct RawModuleInfo {
  pub active: bool,
  pub client: String,
  pub data: String,
}

#[napi(object, object_to_js = false)]
pub struct RawLazyCompilationOption {
  pub current_active_modules: ThreadsafeFunction<(), std::collections::HashSet<String>>,
  pub test: Option<RawLazyCompilationTest>,
  pub entries: bool,
  pub imports: bool,
  pub client: String,
}

pub(crate) struct JsBackend {
  current_active_modules: ThreadsafeFunction<(), std::collections::HashSet<String>>,
}

impl std::fmt::Debug for JsBackend {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("JsBackend").finish()
  }
}

impl From<&RawLazyCompilationOption> for JsBackend {
  fn from(value: &RawLazyCompilationOption) -> Self {
    Self {
      current_active_modules: value.current_active_modules.clone(),
    }
  }
}

#[async_trait::async_trait]
impl Backend for JsBackend {
  async fn current_active_modules(&mut self) -> rspack_error::Result<IdentifierSet> {
    let active_modules = self
      .current_active_modules
      .call_with_sync(())
      .await
      .expect("channel should have result");

    Ok(active_modules.into_iter().map(Into::into).collect())
  }
}
