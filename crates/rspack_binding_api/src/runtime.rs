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

  macro_rules! declare_runtime_global {
    ($name:ident) => {
      to_js_map.insert(
        RuntimeGlobals::$name,
        stringify!($name).to_lower_camel_case().into(),
      );
      from_js_map.insert(stringify!($name), RuntimeGlobals::$name);
    };
  }

  declare_runtime_global!(REQUIRE_SCOPE);
  declare_runtime_global!(MODULE);
  declare_runtime_global!(MODULE_ID);
  declare_runtime_global!(REQUIRE);
  declare_runtime_global!(MODULE_CACHE);
  declare_runtime_global!(ENSURE_CHUNK);
  declare_runtime_global!(ENSURE_CHUNK_HANDLERS);
  declare_runtime_global!(PUBLIC_PATH);
  declare_runtime_global!(GET_CHUNK_SCRIPT_FILENAME);
  declare_runtime_global!(GET_CHUNK_CSS_FILENAME);
  declare_runtime_global!(LOAD_SCRIPT);
  declare_runtime_global!(HAS_OWN_PROPERTY);
  declare_runtime_global!(MODULE_FACTORIES_ADD_ONLY);
  declare_runtime_global!(ON_CHUNKS_LOADED);
  declare_runtime_global!(CHUNK_CALLBACK);
  declare_runtime_global!(MODULE_FACTORIES);
  declare_runtime_global!(INTERCEPT_MODULE_EXECUTION);
  declare_runtime_global!(HMR_DOWNLOAD_MANIFEST);
  declare_runtime_global!(HMR_DOWNLOAD_UPDATE_HANDLERS);
  declare_runtime_global!(GET_UPDATE_MANIFEST_FILENAME);
  declare_runtime_global!(GET_CHUNK_UPDATE_SCRIPT_FILENAME);
  declare_runtime_global!(GET_CHUNK_UPDATE_CSS_FILENAME);
  declare_runtime_global!(HMR_MODULE_DATA);
  declare_runtime_global!(HMR_RUNTIME_STATE_PREFIX);
  declare_runtime_global!(EXTERNAL_INSTALL_CHUNK);
  declare_runtime_global!(GET_FULL_HASH);
  declare_runtime_global!(GLOBAL);
  declare_runtime_global!(RETURN_EXPORTS_FROM_RUNTIME);
  declare_runtime_global!(INSTANTIATE_WASM);
  declare_runtime_global!(ASYNC_MODULE);
  declare_runtime_global!(BASE_URI);
  declare_runtime_global!(MODULE_LOADED);
  declare_runtime_global!(STARTUP_ENTRYPOINT);
  declare_runtime_global!(STARTUP_CHUNK_DEPENDENCIES);
  declare_runtime_global!(CREATE_SCRIPT_URL);
  declare_runtime_global!(CREATE_SCRIPT);
  declare_runtime_global!(GET_TRUSTED_TYPES_POLICY);
  declare_runtime_global!(DEFINE_PROPERTY_GETTERS);
  declare_runtime_global!(ENTRY_MODULE_ID);
  declare_runtime_global!(STARTUP_NO_DEFAULT);
  declare_runtime_global!(ENSURE_CHUNK_INCLUDE_ENTRIES);
  declare_runtime_global!(STARTUP);
  declare_runtime_global!(MAKE_NAMESPACE_OBJECT);
  declare_runtime_global!(EXPORTS);
  declare_runtime_global!(COMPAT_GET_DEFAULT_EXPORT);
  declare_runtime_global!(CREATE_FAKE_NAMESPACE_OBJECT);
  declare_runtime_global!(NODE_MODULE_DECORATOR);
  declare_runtime_global!(ESM_MODULE_DECORATOR);
  declare_runtime_global!(SYSTEM_CONTEXT);
  declare_runtime_global!(THIS_AS_EXPORTS);
  declare_runtime_global!(CURRENT_REMOTE_GET_SCOPE);
  declare_runtime_global!(SHARE_SCOPE_MAP);
  declare_runtime_global!(INITIALIZE_SHARING);
  declare_runtime_global!(SCRIPT_NONCE);
  declare_runtime_global!(RELATIVE_URL);
  declare_runtime_global!(CHUNK_NAME);
  declare_runtime_global!(RUNTIME_ID);
  declare_runtime_global!(PREFETCH_CHUNK);
  declare_runtime_global!(PREFETCH_CHUNK_HANDLERS);
  declare_runtime_global!(PRELOAD_CHUNK);
  declare_runtime_global!(PRELOAD_CHUNK_HANDLERS);
  declare_runtime_global!(UNCAUGHT_ERROR_HANDLER);
  declare_runtime_global!(RSPACK_VERSION);
  declare_runtime_global!(HAS_CSS_MODULES);
  declare_runtime_global!(RSPACK_UNIQUE_ID);
  declare_runtime_global!(HAS_FETCH_PRIORITY);
  declare_runtime_global!(AMD_DEFINE);
  declare_runtime_global!(AMD_OPTIONS);
  declare_runtime_global!(ASYNC_MODULE_EXPORT_SYMBOL);
  declare_runtime_global!(MAKE_DEFERRED_NAMESPACE_OBJECT);
  declare_runtime_global!(MAKE_DEFERRED_NAMESPACE_OBJECT_SYMBOL);

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
