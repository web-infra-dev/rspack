use std::{
  fmt::Debug,
  path::{Path, PathBuf},
  sync::Arc,
};

use anymap::CloneAny;
use once_cell::sync::OnceCell;
use rspack_error::{Error, Result};

use crate::{get_scheme, Scheme};

#[derive(Clone, PartialEq, Eq)]
pub enum Content {
  String(String),
  Buffer(Vec<u8>),
}

impl Content {
  pub fn try_into_string(self) -> Result<String> {
    match self {
      Content::String(s) => Ok(s),
      Content::Buffer(b) => String::from_utf8(b).map_err(|e| rspack_error::error!(e.to_string())),
    }
  }

  pub fn into_string_lossy(self) -> String {
    match self {
      Content::String(s) => s,
      Content::Buffer(b) => String::from_utf8_lossy(&b).into_owned(),
    }
  }

  pub fn as_bytes(&self) -> &[u8] {
    match self {
      Content::String(s) => s.as_bytes(),
      Content::Buffer(b) => b,
    }
  }

  pub fn into_bytes(self) -> Vec<u8> {
    match self {
      Content::String(s) => s.into_bytes(),
      Content::Buffer(b) => b,
    }
  }

  pub fn is_buffer(&self) -> bool {
    matches!(self, Content::Buffer(..))
  }

  pub fn is_string(&self) -> bool {
    matches!(self, Content::String(..))
  }
}

impl TryFrom<Content> for String {
  type Error = Error;

  fn try_from(content: Content) -> Result<Self> {
    content.try_into_string()
  }
}

impl From<Content> for Vec<u8> {
  fn from(content: Content) -> Self {
    content.into_bytes()
  }
}

impl From<String> for Content {
  fn from(s: String) -> Self {
    Self::String(s)
  }
}

impl From<Vec<u8>> for Content {
  fn from(buf: Vec<u8>) -> Self {
    Self::Buffer(buf)
  }
}

impl Debug for Content {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut content = f.debug_struct("Content");

    let s = match self {
      Self::String(s) => s.to_string(),
      Self::Buffer(b) => String::from_utf8_lossy(b).to_string(),
    };

    let ty = match self {
      Self::String(_) => "String",
      Self::Buffer(_) => "Buffer",
    };

    content
      .field(
        ty,
        &s[0..usize::min(s.len(), s.ceil_char_boundary(20))].to_owned(),
      )
      .finish()
  }
}

#[derive(Debug, Clone)]
pub struct ResourceData {
  /// Resource with absolute path, query and fragment
  pub resource: String,
  /// Absolute resource path only
  pub resource_path: PathBuf,
  /// Resource query with `?` prefix
  pub resource_query: Option<String>,
  /// Resource fragment with `#` prefix
  pub resource_fragment: Option<String>,
  pub resource_description: Option<DescriptionData>,
  pub mimetype: Option<String>,
  pub parameters: Option<String>,
  pub encoding: Option<String>,
  pub encoded_content: Option<String>,
  pub(crate) scheme: OnceCell<Scheme>,
  pub context: Option<String>,
}

impl ResourceData {
  pub fn new(resource: String, path: PathBuf) -> Self {
    Self {
      resource,
      resource_path: path,
      resource_query: None,
      resource_fragment: None,
      resource_description: None,
      mimetype: None,
      parameters: None,
      encoding: None,
      encoded_content: None,
      scheme: OnceCell::new(),
      context: None,
    }
  }

  pub fn get_scheme(&self) -> &Scheme {
    self.scheme.get_or_init(|| get_scheme(&self.resource))
  }

  pub fn set_resource(&mut self, v: String) {
    self.resource = v;
  }

  pub fn set_path(&mut self, v: PathBuf) {
    self.resource_path = v;
  }

  pub fn query(mut self, v: String) -> Self {
    self.resource_query = Some(v);
    self
  }

  pub fn set_query(&mut self, v: String) {
    self.resource_query = Some(v);
  }

  pub fn query_optional(mut self, v: Option<String>) -> Self {
    self.resource_query = v;
    self
  }

  pub fn set_query_optional(&mut self, v: Option<String>) {
    self.resource_query = v;
  }

  pub fn fragment(mut self, v: String) -> Self {
    self.resource_fragment = Some(v);
    self
  }

  pub fn set_fragment(&mut self, v: String) {
    self.resource_fragment = Some(v);
  }

  pub fn fragment_optional(mut self, v: Option<String>) -> Self {
    self.resource_fragment = v;
    self
  }

  pub fn set_fragment_optional(&mut self, v: Option<String>) {
    self.resource_fragment = v;
  }

  pub fn description(mut self, v: DescriptionData) -> Self {
    self.resource_description = Some(v);
    self
  }

  pub fn description_optional(mut self, v: Option<DescriptionData>) -> Self {
    self.resource_description = v;
    self
  }

  pub fn mimetype(mut self, v: String) -> Self {
    self.mimetype = Some(v);
    self
  }

  pub fn set_mimetype(&mut self, v: String) {
    self.mimetype = Some(v);
  }

  pub fn parameters(mut self, v: String) -> Self {
    self.parameters = Some(v);
    self
  }

  pub fn set_parameters(&mut self, v: String) {
    self.parameters = Some(v);
  }

  pub fn encoding(mut self, v: String) -> Self {
    self.encoding = Some(v);
    self
  }

  pub fn set_encoding(&mut self, v: String) {
    self.encoding = Some(v);
  }

  pub fn encoded_content(mut self, v: String) -> Self {
    self.encoded_content = Some(v);
    self
  }

  pub fn set_encoded_content(&mut self, v: String) {
    self.encoded_content = Some(v);
  }

  pub fn set_context(&mut self) {
    self.context = Some(
      self
        .resource_path
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .to_str()
        .unwrap_or("")
        .to_string(),
    );
  }
}
/// Used for [Rule.descriptionData](https://www.rspack.dev/config/module.html#ruledescriptiondata) and
/// package.json.sideEffects in tree shaking.
#[derive(Debug, Clone)]
pub struct DescriptionData {
  /// Path to package.json
  path: PathBuf,

  /// Raw package.json
  json: Arc<serde_json::Value>,
}

impl DescriptionData {
  pub fn new(path: PathBuf, json: Arc<serde_json::Value>) -> Self {
    Self { path, json }
  }

  pub fn path(&self) -> &Path {
    &self.path
  }

  pub fn json(&self) -> &serde_json::Value {
    self.json.as_ref()
  }
}

pub type AdditionalData = anymap::Map<dyn CloneAny + Send + Sync>;
