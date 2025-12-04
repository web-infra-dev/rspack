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

type RuntimeGlobalMap = (
  FxHashMap<RuntimeGlobals, String>,
  FxHashMap<&'static str, RuntimeGlobals>,
);

static RUNTIME_GLOBAL_MAP: LazyLock<RuntimeGlobalMap> = LazyLock::new(|| {
  let mut to_js_map = FxHashMap::default();
  let mut from_js_map = FxHashMap::default();

  for (name, value) in RuntimeGlobals::all().iter_names() {
    to_js_map.insert(value, name.to_lower_camel_case());
    from_js_map.insert(name, value);
  }

  to_js_map.shrink_to_fit();
  from_js_map.shrink_to_fit();
  (to_js_map, from_js_map)
});

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
    let mut runtime_globals = vec![];

    for (item, js_name) in RUNTIME_GLOBAL_MAP.0.iter() {
      if value.contains(*item) {
        runtime_globals.push(js_name.into());
      }
    }

    Self {
      value: runtime_globals,
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
    let mut runtime_requirements = RuntimeGlobals::default();

    for item in self.runtime_requirements.value.iter() {
      let snake_case = item.to_snake_case();
      let name = snake_case.cow_to_ascii_uppercase();

      if let Some(item) = RUNTIME_GLOBAL_MAP.1.get(name.as_ref()) {
        runtime_requirements.extend(*item);
      }
    }

    runtime_requirements
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
    let mut runtime_requirements = RuntimeGlobals::default();

    for item in self.all_runtime_requirements.value.iter() {
      let snake_name = item.to_snake_case();

      if let Some(item) = RUNTIME_GLOBAL_MAP
        .1
        .get(snake_name.cow_to_ascii_uppercase().as_ref())
      {
        runtime_requirements.extend(*item);
      }
    }

    runtime_requirements
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
