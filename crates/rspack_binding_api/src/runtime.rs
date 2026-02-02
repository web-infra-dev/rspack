use std::sync::LazyLock;

use cow_utils::CowUtils;
use heck::{ToLowerCamelCase, ToSnakeCase};
use napi::Either;
use napi_derive::napi;
use rspack_core::RuntimeGlobals;
use rspack_plugin_runtime::{
  CreateLinkData, CreateScriptData, LinkPrefetchData, LinkPreloadData, RuntimeModuleChunkWrapper,
};
use rustc_hash::FxHashMap;

use crate::chunk::ChunkWrapper;

#[napi(object, object_from_js = false)]
pub struct JsAdditionalTreeRuntimeRequirementsArg {
  #[napi(ts_type = "Chunk")]
  pub chunk: ChunkWrapper,
  pub runtime_requirements: JsRuntimeGlobals,
}

#[derive(Debug)]
#[napi(object)]
pub struct JsRuntimeGlobals {
  pub value: Vec<String>,
}

impl From<RuntimeGlobals> for JsRuntimeGlobals {
  fn from(value: RuntimeGlobals) -> Self {
    Self {
      value: value
        .to_names()
        .into_iter()
        .map(|name| name.to_lower_camel_case())
        .collect::<Vec<_>>(),
    }
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct JsAdditionalTreeRuntimeRequirementsResult {
  pub runtime_requirements: JsRuntimeGlobals,
}

impl JsAdditionalTreeRuntimeRequirementsResult {
  pub fn as_runtime_globals(&self) -> RuntimeGlobals {
    let names = self
      .runtime_requirements
      .value
      .iter()
      .map(|name| name.to_snake_case().cow_to_ascii_uppercase().to_string())
      .collect::<Vec<_>>();
    RuntimeGlobals::from_names(&names)
  }
}

#[napi(object, object_from_js = false)]
pub struct JsRuntimeRequirementInTreeArg {
  #[napi(ts_type = "Chunk")]
  pub chunk: ChunkWrapper,
  pub all_runtime_requirements: JsRuntimeGlobals,
  pub runtime_requirements: JsRuntimeGlobals,
}

#[derive(Debug)]
#[napi(object)]
pub struct JsRuntimeRequirementInTreeResult {
  pub all_runtime_requirements: JsRuntimeGlobals,
}

impl JsRuntimeRequirementInTreeResult {
  pub fn as_runtime_globals(&self) -> RuntimeGlobals {
    let names = self
      .all_runtime_requirements
      .value
      .iter()
      .map(|name| name.to_snake_case().cow_to_ascii_uppercase().to_string())
      .collect::<Vec<_>>();
    RuntimeGlobals::from_names(&names)
  }
}

#[napi(object, object_from_js = false)]
pub struct JsCreateScriptData {
  pub code: String,
  #[napi(ts_type = "Chunk")]
  pub chunk: ChunkWrapper,
}

impl From<CreateScriptData> for JsCreateScriptData {
  fn from(value: CreateScriptData) -> Self {
    Self {
      code: value.code,
      chunk: value.chunk.into(),
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsCreateLinkData {
  pub code: String,
  #[napi(ts_type = "Chunk")]
  pub chunk: ChunkWrapper,
}

impl From<CreateLinkData> for JsCreateLinkData {
  fn from(value: CreateLinkData) -> Self {
    Self {
      code: value.code,
      chunk: value.chunk.into(),
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsLinkPreloadData {
  pub code: String,
  #[napi(ts_type = "Chunk")]
  pub chunk: ChunkWrapper,
}

impl From<LinkPreloadData> for JsLinkPreloadData {
  fn from(value: LinkPreloadData) -> Self {
    Self {
      code: value.code,
      chunk: value.chunk.into(),
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsLinkPrefetchData {
  pub code: String,
  #[napi(ts_type = "Chunk")]
  pub chunk: ChunkWrapper,
}

impl From<LinkPrefetchData> for JsLinkPrefetchData {
  fn from(value: LinkPrefetchData) -> Self {
    Self {
      code: value.code,
      chunk: value.chunk.into(),
    }
  }
}

impl From<RuntimeModuleChunkWrapper> for ChunkWrapper {
  fn from(value: RuntimeModuleChunkWrapper) -> Self {
    Self {
      chunk_ukey: value.chunk_ukey,
      compilation_id: value.compilation_id,
      compilation: value.compilation,
    }
  }
}

pub type JsRuntimeSpec = Option<Either<String, Vec<String>>>;

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_has_all_runtime_globals() {
    let mut runtime_globals = RuntimeGlobals::default();

    for item in RUNTIME_GLOBAL_MAP.0.keys() {
      runtime_globals.extend(*item);
    }

    for (name, item) in RuntimeGlobals::all().iter_names() {
      assert!(
        runtime_globals.contains(item),
        "missing runtime global in RUNTIME_GLOBAL_MAP.\nname: {name}"
      );
    }
  }
}
