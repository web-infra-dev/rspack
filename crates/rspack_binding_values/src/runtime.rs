use heck::{ToLowerCamelCase, ToSnakeCase};
use napi_derive::napi;
use once_cell::sync::Lazy;
use rspack_core::RuntimeGlobals;
use rustc_hash::FxHashMap;

use crate::JsChunk;

static RUNTIME_GLOBAL_MAP: Lazy<(
  FxHashMap<RuntimeGlobals, String>,
  FxHashMap<String, RuntimeGlobals>,
)> = Lazy::new(|| {
  let mut to_js_map = FxHashMap::default();
  let mut from_js_map = FxHashMap::default();

  macro_rules! declare_runtime_global {
    ($name:ident) => {
      to_js_map.insert(
        RuntimeGlobals::$name,
        stringify!($name).to_lower_camel_case().into(),
      );
      from_js_map.insert(stringify!($name).into(), RuntimeGlobals::$name);
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
  declare_runtime_global!(HARMONY_MODULE_DECORATOR);
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
  declare_runtime_global!(RSPACK_VERSION);
  declare_runtime_global!(HAS_CSS_MODULES);

  (to_js_map, from_js_map)
});

#[napi(object)]
pub struct JsAdditionalTreeRuntimeRequirementsArg {
  pub chunk: JsChunk,
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
      let name = item.to_snake_case().to_uppercase();

      if let Some(item) = RUNTIME_GLOBAL_MAP.1.get(&name) {
        runtime_requirements.extend(*item);
      }
    }

    runtime_requirements
  }
}
