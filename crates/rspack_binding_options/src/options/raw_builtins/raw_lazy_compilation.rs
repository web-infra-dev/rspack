use napi_derive::napi;
use rspack_core::ModuleIdentifier;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_lazy_compilation::backend::{Backend, ModuleInfo};

use crate::RawRegexMatcher;

#[napi(object)]
pub struct RawModuleInfo {
  pub active: bool,
  pub client: String,
  pub data: String,
}

#[napi(object, object_to_js = false)]
pub struct RawLazyCompilationOption {
  pub module: ThreadsafeFunction<RawModuleArg, RawModuleInfo>,
  pub test: Option<RawRegexMatcher>,
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
