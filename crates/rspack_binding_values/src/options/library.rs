use std::vec;

use napi::bindgen_prelude::Either3;
use napi_derive::napi;
use rspack_core::LibraryAuxiliaryComment;
use rspack_core::{LibraryCustomUmdObject, LibraryName, LibraryNonUmdObject, LibraryOptions};
use rspack_napi::napi::bindgen_prelude::Either;

#[derive(Debug)]
#[napi(object)]
pub struct JsLibraryCustomUmdObject {
  pub amd: Option<String>,
  pub commonjs: Option<String>,
  pub root: Option<Either<Vec<String>, String>>,
}

impl From<JsLibraryCustomUmdObject> for LibraryCustomUmdObject {
  fn from(value: JsLibraryCustomUmdObject) -> Self {
    Self {
      amd: value.amd,
      commonjs: value.commonjs,
      root: value.root.map(|v| match v {
        Either::A(v) => v,
        Either::B(v) => vec![v],
      }),
    }
  }
}

impl From<LibraryCustomUmdObject> for JsLibraryCustomUmdObject {
  fn from(value: LibraryCustomUmdObject) -> Self {
    Self {
      amd: value.amd,
      commonjs: value.commonjs,
      root: value.root.map(Either::A),
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

impl From<LibraryName> for JsLibraryName {
  fn from(value: LibraryName) -> Self {
    match value {
      LibraryName::NonUmdObject(l) => match l {
        LibraryNonUmdObject::Array(payload) => JsLibraryName {
          r#type: "array".to_string(),
          string_payload: None,
          array_payload: Some(payload),
          umd_object_payload: None,
        },
        LibraryNonUmdObject::String(payload) => JsLibraryName {
          r#type: "string".to_string(),
          string_payload: Some(payload),
          array_payload: None,
          umd_object_payload: None,
        },
      },
      LibraryName::UmdObject(payload) => JsLibraryName {
        r#type: "umdObject".to_string(),
        string_payload: None,
        array_payload: None,
        umd_object_payload: Some(payload.into()),
      },
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

impl From<String> for JsLibraryAuxiliaryComment {
  fn from(value: String) -> Self {
    Self {
      root: Some(value.clone()),
      commonjs: Some(value.clone()),
      commonjs2: Some(value.clone()),
      amd: Some(value),
    }
  }
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

impl From<LibraryAuxiliaryComment> for JsLibraryAuxiliaryComment {
  fn from(value: LibraryAuxiliaryComment) -> Self {
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
  pub name: Option<Either3<String, Vec<String>, JsLibraryCustomUmdObject>>,
  pub export: Option<Either<Vec<String>, String>>,
  // webpack type
  pub r#type: String,
  pub umd_named_define: Option<bool>,
  pub auxiliary_comment: Option<Either<String, JsLibraryAuxiliaryComment>>,
  pub amd_container: Option<String>,
}

impl From<JsLibraryOptions> for LibraryOptions {
  fn from(value: JsLibraryOptions) -> Self {
    Self {
      name: value.name.map(|name| match name {
        Either3::A(name) => LibraryName::NonUmdObject(LibraryNonUmdObject::String(name)),
        Either3::B(names) => LibraryName::NonUmdObject(LibraryNonUmdObject::Array(names)),
        Either3::C(umd_object) => LibraryName::UmdObject(umd_object.into()),
      }),
      export: value.export.map(|v| match v {
        Either::A(v) => v,
        Either::B(v) => vec![v],
      }),
      library_type: value.r#type,
      umd_named_define: value.umd_named_define,
      auxiliary_comment: value.auxiliary_comment.map(|v| match v {
        Either::A(v) => Into::<JsLibraryAuxiliaryComment>::into(v).into(),
        Either::B(v) => v.into(),
      }),
      amd_container: value.amd_container,
    }
  }
}

impl From<LibraryOptions> for JsLibraryOptions {
  fn from(value: LibraryOptions) -> Self {
    JsLibraryOptions {
      name: value.name.map(|name| match name {
        LibraryName::NonUmdObject(LibraryNonUmdObject::String(name)) => Either3::A(name),
        LibraryName::NonUmdObject(LibraryNonUmdObject::Array(names)) => Either3::B(names),
        LibraryName::UmdObject(umd_object) => Either3::C(umd_object.into()),
      }),
      export: value.export.map(Either::A),
      r#type: value.library_type,
      umd_named_define: value.umd_named_define,
      auxiliary_comment: value
        .auxiliary_comment
        .map(|comment| Either::B(comment.into())),
      amd_container: value.amd_container,
    }
  }
}
