use napi::Either;
use napi_derive::napi;
use rspack_binding_values::library::JsLibraryOptions;
use rspack_binding_values::JsFilename;
use rspack_core::{CrossOriginLoading, Environment, PathInfo};
use rspack_core::{OutputOptions, TrustedTypes};

#[derive(Debug)]
#[napi(object)]
pub struct RawTrustedTypes {
  pub policy_name: Option<String>,
}

impl From<RawTrustedTypes> for TrustedTypes {
  fn from(value: RawTrustedTypes) -> Self {
    Self {
      policy_name: value.policy_name,
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct RawCrossOriginLoading {
  #[napi(ts_type = r#""bool" | "string""#)]
  pub r#type: String,
  pub string_payload: Option<String>,
  pub bool_payload: Option<bool>,
}

impl From<RawCrossOriginLoading> for CrossOriginLoading {
  fn from(value: RawCrossOriginLoading) -> Self {
    match value.r#type.as_str() {
      "string" => Self::Enable(
        value
          .string_payload
          .expect("should have a string_payload when RawCrossOriginLoading.type is \"string\""),
      ),
      "bool" => Self::Disable,
      _ => unreachable!(),
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct RawEnvironment {
  pub r#const: Option<bool>,
  pub arrow_function: Option<bool>,
}

impl From<RawEnvironment> for Environment {
  fn from(value: RawEnvironment) -> Self {
    Self {
      r#const: value.r#const,
      arrow_function: value.arrow_function,
    }
  }
}

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawOutputOptions {
  pub path: String,
  #[napi(ts_type = "boolean | \"verbose\"")]
  pub pathinfo: Either<bool, String>,
  pub clean: bool,
  #[napi(ts_type = "\"auto\" | JsFilename")]
  pub public_path: JsFilename,
  pub asset_module_filename: JsFilename,
  pub wasm_loading: String,
  pub enabled_wasm_loading_types: Vec<String>,
  pub webassembly_module_filename: String,
  pub filename: JsFilename,
  pub chunk_filename: JsFilename,
  pub cross_origin_loading: RawCrossOriginLoading,
  pub css_filename: JsFilename,
  pub css_chunk_filename: JsFilename,
  pub css_head_data_compression: bool,
  pub hot_update_main_filename: String,
  pub hot_update_chunk_filename: String,
  pub hot_update_global: String,
  pub unique_name: String,
  pub chunk_loading_global: String,
  pub library: Option<JsLibraryOptions>,
  pub strict_module_error_handling: bool,
  pub enabled_library_types: Option<Vec<String>>,
  pub global_object: String,
  pub import_function_name: String,
  pub import_meta_name: String,
  pub iife: bool,
  pub module: bool,
  pub chunk_loading: String,
  pub chunk_load_timeout: u32,
  pub charset: bool,
  pub enabled_chunk_loading_types: Option<Vec<String>>,
  pub trusted_types: Option<RawTrustedTypes>,
  pub source_map_filename: String,
  pub hash_function: String,
  pub hash_digest: String,
  pub hash_digest_length: u32,
  pub hash_salt: Option<String>,
  pub async_chunks: bool,
  pub worker_chunk_loading: String,
  pub worker_wasm_loading: String,
  pub worker_public_path: String,
  #[napi(ts_type = r#""module" | "text/javascript" | "false""#)]
  pub script_type: String,
  pub environment: RawEnvironment,
}

impl TryFrom<RawOutputOptions> for OutputOptions {
  type Error = rspack_error::Error;

  fn try_from(value: RawOutputOptions) -> rspack_error::Result<Self> {
    let pathinfo = match value.pathinfo {
      Either::A(b) => PathInfo::Bool(b),
      Either::B(s) => PathInfo::String(s),
    };

    Ok(OutputOptions {
      path: value.path.into(),
      pathinfo,
      clean: value.clean,
      public_path: value.public_path.into(),
      asset_module_filename: value.asset_module_filename.into(),
      wasm_loading: value.wasm_loading.as_str().into(),
      webassembly_module_filename: value.webassembly_module_filename.into(),
      unique_name: value.unique_name,
      chunk_loading: value.chunk_loading.as_str().into(),
      chunk_loading_global: value.chunk_loading_global.as_str().into(),
      filename: value.filename.into(),
      chunk_filename: value.chunk_filename.into(),
      cross_origin_loading: value.cross_origin_loading.into(),
      css_filename: value.css_filename.into(),
      css_chunk_filename: value.css_chunk_filename.into(),
      css_head_data_compression: value.css_head_data_compression,
      hot_update_main_filename: value.hot_update_main_filename.into(),
      hot_update_chunk_filename: value.hot_update_chunk_filename.into(),
      hot_update_global: value.hot_update_global,
      library: value.library.map(Into::into),
      strict_module_error_handling: value.strict_module_error_handling,
      enabled_library_types: value.enabled_library_types,
      global_object: value.global_object,
      import_function_name: value.import_function_name,
      import_meta_name: value.import_meta_name,
      iife: value.iife,
      module: value.module,
      trusted_types: value.trusted_types.map(Into::into),
      source_map_filename: value.source_map_filename.into(),
      hash_function: value.hash_function.as_str().into(),
      hash_digest: value.hash_digest.as_str().into(),
      hash_digest_length: value.hash_digest_length as usize,
      hash_salt: value.hash_salt.into(),
      async_chunks: value.async_chunks,
      worker_chunk_loading: value.worker_chunk_loading.as_str().into(),
      worker_wasm_loading: value.worker_wasm_loading.as_str().into(),
      worker_public_path: value.worker_public_path,
      script_type: value.script_type,
      environment: value.environment.into(),
      charset: value.charset,
      chunk_load_timeout: value.chunk_load_timeout,
    })
  }
}
