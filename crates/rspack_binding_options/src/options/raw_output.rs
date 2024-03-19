use std::fmt;

use napi::Either;
use napi_derive::napi;
use rspack_core::{
  CrossOriginLoading, LibraryCustomUmdObject, LibraryName, LibraryNonUmdObject, LibraryOptions,
  PathInfo,
};
use rspack_core::{LibraryAuxiliaryComment, OutputOptions, TrustedTypes};
use serde::{
  de::{self, Visitor},
  Deserialize, Deserializer,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawLibraryName {
  #[napi(ts_type = r#""string" | "array" | "umdObject""#)]
  pub r#type: String,
  pub string_payload: Option<String>,
  pub array_payload: Option<Vec<String>>,
  pub umd_object_payload: Option<RawLibraryCustomUmdObject>,
}

impl From<RawLibraryName> for LibraryName {
  fn from(value: RawLibraryName) -> Self {
    match value.r#type.as_str() {
      "string" => {
        Self::NonUmdObject(LibraryNonUmdObject::String(value.string_payload.expect(
          "should have a string_payload when RawLibraryName.type is \"string\"",
        )))
      }
      "array" => Self::NonUmdObject(LibraryNonUmdObject::Array(
        value
          .array_payload
          .expect("should have a array_payload when RawLibraryName.type is \"array\""),
      )),
      "umdObject" => Self::UmdObject(
        value
          .umd_object_payload
          .expect("should have a umd_object_payload when RawLibraryName.type is \"umdObject\"")
          .into(),
      ),
      _ => unreachable!(),
    }
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawLibraryCustomUmdObject {
  pub amd: Option<String>,
  pub commonjs: Option<String>,
  pub root: Option<Vec<String>>,
}

impl From<RawLibraryCustomUmdObject> for LibraryCustomUmdObject {
  fn from(value: RawLibraryCustomUmdObject) -> Self {
    Self {
      amd: value.amd,
      commonjs: value.commonjs,
      root: value.root,
    }
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawLibraryAuxiliaryComment {
  pub root: Option<String>,
  pub commonjs: Option<String>,
  pub commonjs2: Option<String>,
  pub amd: Option<String>,
}

impl From<RawLibraryAuxiliaryComment> for LibraryAuxiliaryComment {
  fn from(value: RawLibraryAuxiliaryComment) -> Self {
    Self {
      amd: value.amd,
      commonjs: value.commonjs,
      root: value.root,
      commonjs2: value.commonjs2,
    }
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawLibraryOptions {
  pub name: Option<RawLibraryName>,
  pub export: Option<Vec<String>>,
  // webpack type
  pub library_type: String,
  pub umd_named_define: Option<bool>,
  pub auxiliary_comment: Option<RawLibraryAuxiliaryComment>,
  pub amd_container: Option<String>,
}

impl From<RawLibraryOptions> for LibraryOptions {
  fn from(value: RawLibraryOptions) -> Self {
    Self {
      name: value.name.map(Into::into),
      export: value.export,
      library_type: value.library_type,
      umd_named_define: value.umd_named_define,
      auxiliary_comment: value.auxiliary_comment.map(Into::into),
      amd_container: value.amd_container,
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
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

struct PathInfoVisitor;

impl<'de> Visitor<'de> for PathInfoVisitor {
  type Value = Either<bool, String>;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("a boolean or a string")
  }

  fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Ok(Either::A(value))
  }

  fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Ok(Either::B(value.to_string()))
  }
}

fn deserialize_pathinfo<'de, D>(deserializer: D) -> Result<Either<bool, String>, D::Error>
where
  D: Deserializer<'de>,
{
  deserializer.deserialize_any(PathInfoVisitor)
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawOutputOptions {
  pub path: String,
  #[serde(deserialize_with = "deserialize_pathinfo")]
  #[napi(ts_type = "boolean | \"verbose\"")]
  pub pathinfo: Either<bool, String>,
  pub clean: bool,
  pub public_path: String,
  pub asset_module_filename: String,
  pub wasm_loading: String,
  pub enabled_wasm_loading_types: Vec<String>,
  pub webassembly_module_filename: String,
  pub filename: String,
  pub chunk_filename: String,
  pub cross_origin_loading: RawCrossOriginLoading,
  pub css_filename: String,
  pub css_chunk_filename: String,
  pub hot_update_main_filename: String,
  pub hot_update_chunk_filename: String,
  pub hot_update_global: String,
  pub unique_name: String,
  pub chunk_loading_global: String,
  pub library: Option<RawLibraryOptions>,
  pub strict_module_error_handling: bool,
  pub enabled_library_types: Option<Vec<String>>,
  pub global_object: String,
  pub import_function_name: String,
  pub iife: bool,
  pub module: bool,
  pub chunk_loading: String,
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
      hot_update_main_filename: value.hot_update_main_filename.into(),
      hot_update_chunk_filename: value.hot_update_chunk_filename.into(),
      hot_update_global: value.hot_update_global,
      library: value.library.map(Into::into),
      strict_module_error_handling: value.strict_module_error_handling,
      enabled_library_types: value.enabled_library_types,
      global_object: value.global_object,
      import_function_name: value.import_function_name,
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
    })
  }
}
