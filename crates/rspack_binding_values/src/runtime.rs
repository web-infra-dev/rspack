use heck::{ToLowerCamelCase, ToSnakeCase};
use napi_derive::napi;
use rspack_core::RuntimeGlobals;

use crate::JsChunk;

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

    macro_rules! declare_from_runtime_global {
      ($name:ident) => {
        if value.contains(RuntimeGlobals::$name) {
          runtime_globals.push(stringify!($name).to_lower_camel_case().into());
        }
      };
    }

    declare_from_runtime_global!(REQUIRE_SCOPE);
    declare_from_runtime_global!(MODULE);
    declare_from_runtime_global!(MODULE_ID);
    declare_from_runtime_global!(REQUIRE);
    declare_from_runtime_global!(MODULE_CACHE);
    declare_from_runtime_global!(ENSURE_CHUNK);
    declare_from_runtime_global!(ENSURE_CHUNK_HANDLERS);
    declare_from_runtime_global!(PUBLIC_PATH);
    declare_from_runtime_global!(GET_CHUNK_SCRIPT_FILENAME);
    declare_from_runtime_global!(GET_CHUNK_CSS_FILENAME);
    declare_from_runtime_global!(LOAD_SCRIPT);
    declare_from_runtime_global!(HAS_OWN_PROPERTY);
    declare_from_runtime_global!(MODULE_FACTORIES_ADD_ONLY);
    declare_from_runtime_global!(ON_CHUNKS_LOADED);
    declare_from_runtime_global!(CHUNK_CALLBACK);
    declare_from_runtime_global!(MODULE_FACTORIES);
    declare_from_runtime_global!(INTERCEPT_MODULE_EXECUTION);
    declare_from_runtime_global!(HMR_DOWNLOAD_MANIFEST);
    declare_from_runtime_global!(HMR_DOWNLOAD_UPDATE_HANDLERS);
    declare_from_runtime_global!(GET_UPDATE_MANIFEST_FILENAME);
    declare_from_runtime_global!(GET_CHUNK_UPDATE_SCRIPT_FILENAME);
    declare_from_runtime_global!(GET_CHUNK_UPDATE_CSS_FILENAME);
    declare_from_runtime_global!(HMR_MODULE_DATA);
    declare_from_runtime_global!(HMR_RUNTIME_STATE_PREFIX);
    declare_from_runtime_global!(EXTERNAL_INSTALL_CHUNK);
    declare_from_runtime_global!(GET_FULL_HASH);
    declare_from_runtime_global!(GLOBAL);
    declare_from_runtime_global!(RETURN_EXPORTS_FROM_RUNTIME);
    declare_from_runtime_global!(INSTANTIATE_WASM);
    declare_from_runtime_global!(ASYNC_MODULE);
    declare_from_runtime_global!(BASE_URI);
    declare_from_runtime_global!(MODULE_LOADED);
    declare_from_runtime_global!(STARTUP_ENTRYPOINT);
    declare_from_runtime_global!(CREATE_SCRIPT_URL);
    declare_from_runtime_global!(CREATE_SCRIPT);
    declare_from_runtime_global!(GET_TRUSTED_TYPES_POLICY);
    declare_from_runtime_global!(DEFINE_PROPERTY_GETTERS);
    declare_from_runtime_global!(ENTRY_MODULE_ID);
    declare_from_runtime_global!(STARTUP_NO_DEFAULT);
    declare_from_runtime_global!(ENSURE_CHUNK_INCLUDE_ENTRIES);
    declare_from_runtime_global!(STARTUP);
    declare_from_runtime_global!(MAKE_NAMESPACE_OBJECT);
    declare_from_runtime_global!(EXPORTS);
    declare_from_runtime_global!(COMPAT_GET_DEFAULT_EXPORT);
    declare_from_runtime_global!(CREATE_FAKE_NAMESPACE_OBJECT);
    declare_from_runtime_global!(NODE_MODULE_DECORATOR);
    declare_from_runtime_global!(HARMONY_MODULE_DECORATOR);
    declare_from_runtime_global!(SYSTEM_CONTEXT);
    declare_from_runtime_global!(THIS_AS_EXPORTS);
    declare_from_runtime_global!(CURRENT_REMOTE_GET_SCOPE);
    declare_from_runtime_global!(SHARE_SCOPE_MAP);
    declare_from_runtime_global!(INITIALIZE_SHARING);
    declare_from_runtime_global!(SCRIPT_NONCE);
    declare_from_runtime_global!(RELATIVE_URL);
    declare_from_runtime_global!(CHUNK_NAME);
    declare_from_runtime_global!(RUNTIME_ID);
    declare_from_runtime_global!(PREFETCH_CHUNK);
    declare_from_runtime_global!(PREFETCH_CHUNK_HANDLERS);
    declare_from_runtime_global!(PRELOAD_CHUNK);
    declare_from_runtime_global!(PRELOAD_CHUNK_HANDLERS);
    declare_from_runtime_global!(RSPACK_VERSION);
    declare_from_runtime_global!(HAS_CSS_MODULES);

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

      macro_rules! declare_to_runtime_global {
        ($name:ident) => {
          if name == stringify!($name) {
            runtime_requirements.insert(RuntimeGlobals::$name);
          }
        };
      }

      declare_to_runtime_global!(REQUIRE_SCOPE);
      declare_to_runtime_global!(MODULE);
      declare_to_runtime_global!(MODULE_ID);
      declare_to_runtime_global!(REQUIRE);
      declare_to_runtime_global!(MODULE_CACHE);
      declare_to_runtime_global!(ENSURE_CHUNK);
      declare_to_runtime_global!(ENSURE_CHUNK_HANDLERS);
      declare_to_runtime_global!(PUBLIC_PATH);
      declare_to_runtime_global!(GET_CHUNK_SCRIPT_FILENAME);
      declare_to_runtime_global!(GET_CHUNK_CSS_FILENAME);
      declare_to_runtime_global!(LOAD_SCRIPT);
      declare_to_runtime_global!(HAS_OWN_PROPERTY);
      declare_to_runtime_global!(MODULE_FACTORIES_ADD_ONLY);
      declare_to_runtime_global!(ON_CHUNKS_LOADED);
      declare_to_runtime_global!(CHUNK_CALLBACK);
      declare_to_runtime_global!(MODULE_FACTORIES);
      declare_to_runtime_global!(INTERCEPT_MODULE_EXECUTION);
      declare_to_runtime_global!(HMR_DOWNLOAD_MANIFEST);
      declare_to_runtime_global!(HMR_DOWNLOAD_UPDATE_HANDLERS);
      declare_to_runtime_global!(GET_UPDATE_MANIFEST_FILENAME);
      declare_to_runtime_global!(GET_CHUNK_UPDATE_SCRIPT_FILENAME);
      declare_to_runtime_global!(GET_CHUNK_UPDATE_CSS_FILENAME);
      declare_to_runtime_global!(HMR_MODULE_DATA);
      declare_to_runtime_global!(HMR_RUNTIME_STATE_PREFIX);
      declare_to_runtime_global!(EXTERNAL_INSTALL_CHUNK);
      declare_to_runtime_global!(GET_FULL_HASH);
      declare_to_runtime_global!(GLOBAL);
      declare_to_runtime_global!(RETURN_EXPORTS_FROM_RUNTIME);
      declare_to_runtime_global!(INSTANTIATE_WASM);
      declare_to_runtime_global!(ASYNC_MODULE);
      declare_to_runtime_global!(BASE_URI);
      declare_to_runtime_global!(MODULE_LOADED);
      declare_to_runtime_global!(STARTUP_ENTRYPOINT);
      declare_to_runtime_global!(CREATE_SCRIPT_URL);
      declare_to_runtime_global!(CREATE_SCRIPT);
      declare_to_runtime_global!(GET_TRUSTED_TYPES_POLICY);
      declare_to_runtime_global!(DEFINE_PROPERTY_GETTERS);
      declare_to_runtime_global!(ENTRY_MODULE_ID);
      declare_to_runtime_global!(STARTUP_NO_DEFAULT);
      declare_to_runtime_global!(ENSURE_CHUNK_INCLUDE_ENTRIES);
      declare_to_runtime_global!(STARTUP);
      declare_to_runtime_global!(MAKE_NAMESPACE_OBJECT);
      declare_to_runtime_global!(EXPORTS);
      declare_to_runtime_global!(COMPAT_GET_DEFAULT_EXPORT);
      declare_to_runtime_global!(CREATE_FAKE_NAMESPACE_OBJECT);
      declare_to_runtime_global!(NODE_MODULE_DECORATOR);
      declare_to_runtime_global!(HARMONY_MODULE_DECORATOR);
      declare_to_runtime_global!(SYSTEM_CONTEXT);
      declare_to_runtime_global!(THIS_AS_EXPORTS);
      declare_to_runtime_global!(CURRENT_REMOTE_GET_SCOPE);
      declare_to_runtime_global!(SHARE_SCOPE_MAP);
      declare_to_runtime_global!(INITIALIZE_SHARING);
      declare_to_runtime_global!(SCRIPT_NONCE);
      declare_to_runtime_global!(RELATIVE_URL);
      declare_to_runtime_global!(CHUNK_NAME);
      declare_to_runtime_global!(RUNTIME_ID);
      declare_to_runtime_global!(PREFETCH_CHUNK);
      declare_to_runtime_global!(PREFETCH_CHUNK_HANDLERS);
      declare_to_runtime_global!(PRELOAD_CHUNK);
      declare_to_runtime_global!(PRELOAD_CHUNK_HANDLERS);
      declare_to_runtime_global!(RSPACK_VERSION);
      declare_to_runtime_global!(HAS_CSS_MODULES);
    }

    runtime_requirements
  }
}
