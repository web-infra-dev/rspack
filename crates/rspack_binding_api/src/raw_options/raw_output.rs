use napi::Either;
use napi_derive::napi;
use rspack_core::{
  ChunkLoading, CleanOptions, CrossOriginLoading, Environment, OnPolicyCreationFailure,
  OutputOptions, PathInfo, TrustedTypes, WasmLoading,
};

use crate::{
  clean_options::JsCleanOptions, filename::JsFilename, options::library::JsLibraryOptions,
  raw_options::WithFalse,
};

#[derive(Debug)]
#[napi(object)]
pub struct RawTrustedTypes {
  pub policy_name: Option<String>,
  pub on_policy_creation_failure: Option<String>,
}

impl From<RawTrustedTypes> for TrustedTypes {
  fn from(value: RawTrustedTypes) -> Self {
    Self {
      policy_name: value.policy_name,
      on_policy_creation_failure: match value.on_policy_creation_failure {
        Some(v) => OnPolicyCreationFailure::from(v),
        None => OnPolicyCreationFailure::Stop,
      },
    }
  }
}

type RawCrossOriginLoading = WithFalse<String>;

impl From<RawCrossOriginLoading> for CrossOriginLoading {
  fn from(value: RawCrossOriginLoading) -> Self {
    match value {
      WithFalse::True(s) => Self::Enable(s),
      WithFalse::False => Self::Disable,
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct RawEnvironment {
  pub r#const: bool,
  pub method_shorthand: bool,
  pub arrow_function: bool,
  pub node_prefix_for_core_modules: bool,
  pub async_function: bool,
  pub big_int_literal: bool,
  pub destructuring: bool,
  pub document: bool,
  pub dynamic_import: bool,
  pub for_of: bool,
  pub global_this: bool,
  pub module: bool,
  pub optional_chaining: bool,
  pub template_literal: bool,
  pub dynamic_import_in_worker: bool,
  pub import_meta_dirname_and_filename: bool,
}

impl From<RawEnvironment> for Environment {
  fn from(value: RawEnvironment) -> Self {
    Self {
      r#const: value.r#const,
      method_shorthand: value.method_shorthand,
      arrow_function: value.arrow_function,
      node_prefix_for_core_modules: value.node_prefix_for_core_modules,
      async_function: value.async_function,
      big_int_literal: value.big_int_literal,
      destructuring: value.destructuring,
      document: value.document,
      dynamic_import: value.dynamic_import,
      for_of: value.for_of,
      global_this: value.global_this,
      module: value.module,
      optional_chaining: value.optional_chaining,
      template_literal: value.template_literal,
      dynamic_import_in_worker: value.dynamic_import_in_worker,
      import_meta_dirname_and_filename: value.import_meta_dirname_and_filename,
    }
  }
}

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawOutputOptions {
  pub path: String,
  #[napi(ts_type = "boolean | \"verbose\"")]
  pub pathinfo: Either<bool, String>,
  pub clean: Either<bool, JsCleanOptions>,
  #[napi(ts_type = "\"auto\" | JsFilename")]
  pub public_path: JsFilename,
  pub asset_module_filename: JsFilename,
  #[napi(ts_type = "string | false")]
  pub wasm_loading: RawWasmLoading,
  pub enabled_wasm_loading_types: Vec<String>,
  pub webassembly_module_filename: String,
  pub filename: JsFilename,
  pub chunk_filename: JsFilename,
  #[napi(ts_type = "string | false")]
  pub cross_origin_loading: RawCrossOriginLoading,
  pub css_filename: JsFilename,
  pub css_chunk_filename: JsFilename,
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
  #[napi(ts_type = "string | false")]
  pub chunk_loading: RawChunkLoading,
  pub chunk_load_timeout: u32,
  pub enabled_chunk_loading_types: Option<Vec<String>>,
  pub trusted_types: Option<RawTrustedTypes>,
  pub source_map_filename: String,
  pub hash_function: String,
  pub hash_digest: String,
  pub hash_digest_length: u32,
  pub hash_salt: Option<String>,
  pub async_chunks: bool,
  #[napi(ts_type = "string | false")]
  pub worker_chunk_loading: RawChunkLoading,
  #[napi(ts_type = "string | false")]
  pub worker_wasm_loading: RawWasmLoading,
  pub worker_public_path: String,
  #[napi(ts_type = r#""module" | "text/javascript" | false"#)]
  pub script_type: WithFalse<String>,
  pub environment: RawEnvironment,
  pub compare_before_emit: bool,
}

pub type RawWasmLoading = WithFalse<String>;
pub type RawChunkLoading = WithFalse<String>;

impl From<RawChunkLoading> for ChunkLoading {
  fn from(value: RawChunkLoading) -> Self {
    match value {
      WithFalse::False => Self::Disable,
      WithFalse::True(s) => Self::Enable(s.as_str().into()),
    }
  }
}

impl From<RawWasmLoading> for WasmLoading {
  fn from(value: RawWasmLoading) -> Self {
    match value {
      WithFalse::False => Self::Disable,
      WithFalse::True(s) => Self::Enable(s.as_str().into()),
    }
  }
}

impl TryFrom<RawOutputOptions> for OutputOptions {
  type Error = rspack_error::Error;

  fn try_from(value: RawOutputOptions) -> rspack_error::Result<Self> {
    let pathinfo = match value.pathinfo {
      Either::A(b) => PathInfo::Bool(b),
      Either::B(s) => PathInfo::String(s),
    };

    let clean = match value.clean {
      Either::A(b) => CleanOptions::CleanAll(b),
      Either::B(cop) => cop.into(),
    };

    Ok(OutputOptions {
      path: value.path.into(),
      pathinfo,
      clean,
      public_path: value.public_path.into(),
      asset_module_filename: value.asset_module_filename.into(),
      wasm_loading: value.wasm_loading.into(),
      webassembly_module_filename: value.webassembly_module_filename.into(),
      unique_name: value.unique_name,
      chunk_loading: value.chunk_loading.into(),
      chunk_loading_global: value.chunk_loading_global.as_str().into(),
      filename: value.filename.into(),
      chunk_filename: value.chunk_filename.into(),
      cross_origin_loading: value.cross_origin_loading.into(),
      css_filename: value.css_filename.into(),
      css_chunk_filename: value.css_chunk_filename.into(),
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
      worker_chunk_loading: value.worker_chunk_loading.into(),
      worker_wasm_loading: value.worker_wasm_loading.into(),
      worker_public_path: value.worker_public_path,
      script_type: match value.script_type {
        WithFalse::False => "false".to_string(),
        WithFalse::True(s) => s,
      },
      environment: value.environment.into(),
      chunk_load_timeout: value.chunk_load_timeout,
      compare_before_emit: value.compare_before_emit,
    })
  }
}
