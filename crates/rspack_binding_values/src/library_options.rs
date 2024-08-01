use napi_derive::napi;
use rspack_core::{
  LibraryAuxiliaryComment, LibraryCustomUmdObject, LibraryName, LibraryNonUmdObject, LibraryOptions,
};

#[derive(Debug)]
#[napi(object)]
pub struct JsLibraryCustomUmdObject {
  pub amd: Option<String>,
  pub commonjs: Option<String>,
  pub root: Option<Vec<String>>,
}

impl From<JsLibraryCustomUmdObject> for LibraryCustomUmdObject {
  fn from(value: JsLibraryCustomUmdObject) -> Self {
    Self {
      amd: value.amd,
      commonjs: value.commonjs,
      root: value.root,
    }
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct JsLibraryName {
  #[napi(ts_type = r#""string" | "array" | "umdObject""#)]
  pub r#type: String,
  pub string_payload: Option<String>,
  pub array_payload: Option<Vec<String>>,
  pub umd_object_payload: Option<JsLibraryCustomUmdObject>,
}

impl From<JsLibraryName> for LibraryName {
  fn from(value: JsLibraryName) -> Self {
    match value.r#type.as_str() {
      "string" => {
        Self::NonUmdObject(LibraryNonUmdObject::String(value.string_payload.expect(
          "should have a string_payload when JsLibraryName.type is \"string\"",
        )))
      }
      "array" => Self::NonUmdObject(LibraryNonUmdObject::Array(
        value
          .array_payload
          .expect("should have a array_payload when JsLibraryName.type is \"array\""),
      )),
      "umdObject" => Self::UmdObject(
        value
          .umd_object_payload
          .expect("should have a umd_object_payload when JsLibraryName.type is \"umdObject\"")
          .into(),
      ),
      _ => unreachable!(),
    }
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct JsLibraryAuxiliaryComment {
  pub root: Option<String>,
  pub commonjs: Option<String>,
  pub commonjs2: Option<String>,
  pub amd: Option<String>,
}

impl From<JsLibraryAuxiliaryComment> for LibraryAuxiliaryComment {
  fn from(value: JsLibraryAuxiliaryComment) -> Self {
    Self {
      amd: value.amd,
      commonjs: value.commonjs,
      root: value.root,
      commonjs2: value.commonjs2,
    }
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct JsLibraryOptions {
  pub name: Option<JsLibraryName>,
  pub export: Option<Vec<String>>,
  // webpack type
  pub library_type: String,
  pub umd_named_define: Option<bool>,
  pub auxiliary_comment: Option<JsLibraryAuxiliaryComment>,
  pub amd_container: Option<String>,
}

impl From<JsLibraryOptions> for LibraryOptions {
  fn from(value: JsLibraryOptions) -> Self {
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
