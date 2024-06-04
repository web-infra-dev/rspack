use napi::{
  bindgen_prelude::{FromNapiValue, ToNapiValue, ValidateNapiValue},
  Either,
};
use napi_derive::napi;
use rspack_binding_values::{JsModule, RawRegex, ToJsModule};
use rspack_core::ModuleIdentifier;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_lazy_compilation::{
  backend::{Backend, ModuleInfo},
  plugin::{LazyCompilationTest, LazyCompilationTestCheck},
};
use rspack_regex::RspackRegex;

#[derive(Debug)]
pub struct RawLazyCompilationTest<F = ThreadsafeFunction<JsModule, Option<bool>>>(
  pub Either<RawRegex, F>,
);

impl<F: FromNapiValue + ValidateNapiValue> FromNapiValue for RawLazyCompilationTest<F> {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    Ok(Self(Either::from_napi_value(env, napi_val)?))
  }
}

impl<F: ToNapiValue> ToNapiValue for RawLazyCompilationTest<F> {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    Either::to_napi_value(env, val.0)
  }
}

#[derive(Debug)]
pub struct LazyCompilationTestFn {
  tsfn: ThreadsafeFunction<JsModule, Option<bool>>,
}

impl LazyCompilationTestCheck for LazyCompilationTestFn {
  fn test(&self, m: &dyn rspack_core::Module) -> bool {
    let res = self
      .tsfn
      .blocking_call_with_sync(
        m.to_js_module()
          .expect("failed to convert module to js module"),
      )
      .expect("failed to invoke lazyCompilation.test");

    res.unwrap_or(false)
  }
}

impl From<RawLazyCompilationTest> for LazyCompilationTest<LazyCompilationTestFn> {
  fn from(value: RawLazyCompilationTest) -> Self {
    match value.0 {
      Either::A(regex) => Self::Regex(
        RspackRegex::with_flags(&regex.source, &regex.flags).unwrap_or_else(|_| {
          let msg = format!("[lazyCompilation]incorrect regex {:?}", regex);
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
  pub module: ThreadsafeFunction<RawModuleArg, RawModuleInfo>,
  pub test: Option<RawLazyCompilationTest>,
  pub entries: bool,
  pub imports: bool,
  pub cacheable: bool,
}

#[napi(object)]
pub struct RawModuleArg {
  pub module: String,
  pub path: String,
}

pub(crate) struct JsBackend {
  module: ThreadsafeFunction<RawModuleArg, RawModuleInfo>,
}

impl std::fmt::Debug for JsBackend {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("JsBackend").finish()
  }
}

impl From<&RawLazyCompilationOption> for JsBackend {
  fn from(value: &RawLazyCompilationOption) -> Self {
    Self {
      module: value.module.clone(),
    }
  }
}

#[async_trait::async_trait]
impl Backend for JsBackend {
  async fn module(
    &mut self,
    identifier: ModuleIdentifier,
    path: String,
  ) -> rspack_error::Result<ModuleInfo> {
    let module_info = self
      .module
      .call(RawModuleArg {
        module: identifier.to_string(),
        path,
      })
      .await
      .expect("channel should have result");

    Ok(ModuleInfo {
      active: module_info.active,
      client: module_info.client,
      data: module_info.data,
    })
  }
}
