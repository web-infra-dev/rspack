use std::{
  any::Any,
  borrow::Cow,
  fmt::Debug,
  ops::Range,
  path::{Path, PathBuf},
  sync::Arc,
};

use anymap::CloneAny;
use once_cell::sync::OnceCell;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  rkyv::string::ArchivedString,
  utils::PortablePath,
  with::{As, AsInner, AsPreset, Skip},
};
use rspack_error::{Error, Result, ToStringResultToRspackResultExt};
use rspack_hash::{RspackHash, RspackHasher};
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_util::identifier::strip_zero_width_space_for_fragment;
use rustc_hash::FxHashMap;

use crate::{Scheme, get_scheme, loader::path_query_fragment};

#[derive(Clone, PartialEq, Eq)]
pub enum Content {
  String(String),
  Buffer(Vec<u8>),
}

impl RspackHash for Content {
  fn hash(&self, state: &mut RspackHasher) {
    match self {
      Self::String(content) => content.hash(state),
      Self::Buffer(content) => state.write(content),
    }
  }
}

impl Content {
  pub fn try_into_string(self) -> Result<String> {
    match self {
      Content::String(s) => Ok(s),
      Content::Buffer(b) => String::from_utf8(b).to_rspack_result(),
    }
  }

  #[allow(unsafe_code)]
  pub fn into_string_lossy(self) -> String {
    match self {
      Content::String(s) => s,
      Content::Buffer(b) => {
        if simdutf8::basic::from_utf8(&b).is_ok() {
          // SAFETY: simdutf8 validated the buffer as UTF-8 above.
          unsafe { String::from_utf8_unchecked(b) }
        } else {
          String::from_utf8_lossy_owned(b)
        }
      }
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
      Self::String(s) => s.clone(),
      Self::Buffer(b) => String::from_utf8_lossy(b).to_string(),
    };

    let ty = match self {
      Self::String(_) => "String",
      Self::Buffer(_) => "Buffer",
    };

    // Safe string truncation to approximately 20 characters
    let truncated = if s.len() <= 20 {
      s.as_str()
    } else {
      // Find the last character boundary at or before position 20
      let mut end = 20;
      while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
      }
      &s[0..end]
    };

    content.field(ty, &truncated.to_owned()).finish()
  }
}

fn resource_offset(offset: usize) -> u32 {
  u32::try_from(offset).expect("resource data exceeds u32::MAX bytes")
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ResourceData {
  // The complete resource occupies the prefix. Independently supplied or decoded
  // components may use the rest of the same buffer.
  resource: String,
  resource_end: u32,
  resource_path: Option<Range<u32>>,
  resource_query: Option<Range<u32>>,
  resource_fragment: Option<Range<u32>>,
  path_escaped: bool,
  #[cacheable(with=Skip)]
  decoded_path: OnceCell<Utf8PathBuf>,
  resource_description: Option<DescriptionData>,
  mimetype: Option<String>,
  parameters: Option<String>,
  encoding: Option<String>,
  encoded_content: Option<String>,
  context: Option<String>,
  #[cacheable(with=AsInner)]
  scheme: OnceCell<Scheme>,
}

impl ResourceData {
  pub fn new_with_resource(resource: String) -> Self {
    let resource_end = resource_offset(resource.len());
    let (path, query, fragment) = path_query_fragment(&resource);
    let path_end = resource_offset(path.len());
    let path_escaped = path.contains("\u{200b}#");
    let resource_query = query.map(|query| path_end..resource_offset(path.len() + query.len()));
    let decoded_query = match query.map(strip_zero_width_space_for_fragment) {
      Some(Cow::Owned(query)) => Some(query),
      _ => None,
    };
    let resource_fragment =
      fragment.map(|fragment| resource_end - resource_offset(fragment.len())..resource_end);
    let mut data = Self {
      resource,
      resource_end,
      resource_path: Some(0..path_end),
      resource_query,
      resource_fragment,
      path_escaped,
      ..Default::default()
    };
    if let Some(query) = decoded_query {
      data.resource_query = Some(data.store_component(resource_end as usize, &query));
    }
    data
  }

  pub fn new_with_path(
    resource: String,
    path: Utf8PathBuf,
    query: Option<String>,
    fragment: Option<String>,
  ) -> Self {
    Self::from_parts(
      resource,
      Some(path.as_str()),
      false,
      query.as_deref(),
      fragment.as_deref(),
    )
  }

  fn from_parts(
    resource: String,
    path: Option<&str>,
    path_escaped: bool,
    query: Option<&str>,
    fragment: Option<&str>,
  ) -> Self {
    let resource_end = resource_offset(resource.len());
    let fragment_start = resource.len().saturating_sub(fragment.map_or(0, str::len));
    let query_start = fragment_start.saturating_sub(query.map_or(0, str::len));
    let mut data = Self {
      resource,
      resource_end,
      path_escaped,
      ..Default::default()
    };
    data.resource_path = path.map(|path| data.store_component(0, path));
    data.resource_query = query.map(|query| data.store_component(query_start, query));
    data.resource_fragment =
      fragment.map(|fragment| data.store_component(fragment_start, fragment));
    data
  }

  fn component(&self, range: Option<&Range<u32>>) -> Option<&str> {
    range.map(|range| &self.resource[range.start as usize..range.end as usize])
  }

  fn store_component(&mut self, offset: usize, value: &str) -> Range<u32> {
    let end = offset.saturating_add(value.len());
    if self.resource().get(offset..end) == Some(value) {
      resource_offset(offset)..resource_offset(end)
    } else {
      let start = resource_offset(self.resource.len());
      let end = resource_offset(self.resource.len().saturating_add(value.len()));
      self.resource.push_str(value);
      start..end
    }
  }

  fn replace_components(&mut self, data: Self) {
    self.resource = data.resource;
    self.resource_end = data.resource_end;
    self.resource_path = data.resource_path;
    self.resource_query = data.resource_query;
    self.resource_fragment = data.resource_fragment;
    self.path_escaped = data.path_escaped;
    self.decoded_path = data.decoded_path;
  }

  /// Build a virtual resource from known components without parsing its delimiters.
  pub fn with_matching_path(&self, mut path: String) -> Self {
    let path_end = resource_offset(path.len());
    let resource_query = self.query().map(|query| {
      path.push_str(query);
      path_end..resource_offset(path.len())
    });
    let resource_fragment = self.fragment().map(|fragment| {
      let start = resource_offset(path.len());
      path.push_str(fragment);
      start..resource_offset(path.len())
    });
    let resource_end = resource_offset(path.len());
    Self {
      resource: path,
      resource_end,
      resource_path: Some(0..path_end),
      resource_query,
      resource_fragment,
      resource_description: self.resource_description.clone(),
      mimetype: self.mimetype.clone(),
      parameters: self.parameters.clone(),
      encoding: self.encoding.clone(),
      encoded_content: self.encoded_content.clone(),
      context: self.context.clone(),
      ..Default::default()
    }
  }

  pub fn set_context(&mut self, context: Option<String>) {
    self.context = context;
  }

  pub fn context(&self) -> Option<&str> {
    self.context.as_deref()
  }

  pub fn get_scheme(&self) -> &Scheme {
    self.scheme.get_or_init(|| {
      if let Some(path) = self.path() {
        get_scheme(path.as_str())
      } else {
        Scheme::None
      }
    })
  }

  pub fn resource(&self) -> &str {
    &self.resource[..self.resource_end as usize]
  }

  pub fn set_resource(&mut self, v: String) {
    self.replace_components(Self::from_parts(
      v,
      self.component(self.resource_path.as_ref()),
      self.path_escaped,
      self.query(),
      self.fragment(),
    ));
  }

  pub fn path(&self) -> Option<&Utf8Path> {
    let path = self.component(self.resource_path.as_ref())?;
    if self.path_escaped {
      Some(self.decoded_path.get_or_init(|| {
        strip_zero_width_space_for_fragment(path)
          .into_owned()
          .into()
      }))
    } else {
      Some(Utf8Path::new(path))
    }
  }

  pub fn set_path<P: Into<Utf8PathBuf>>(&mut self, v: P) {
    let new_path = v.into();
    if let Some(path) = self.path()
      && path != new_path
    {
      self.scheme.take();
      self.replace_components(Self::from_parts(
        self.resource().to_owned(),
        Some(new_path.as_str()),
        false,
        self.query(),
        self.fragment(),
      ));
    }
  }

  pub fn set_path_optional<P: Into<Utf8PathBuf>>(&mut self, v: Option<P>) {
    if let Some(v) = v {
      self.set_path(v);
    }
  }

  pub fn query(&self) -> Option<&str> {
    self.component(self.resource_query.as_ref())
  }

  pub fn set_query(&mut self, v: String) {
    self.set_query_optional(Some(v));
  }

  pub fn set_query_optional(&mut self, v: Option<String>) {
    self.replace_components(Self::from_parts(
      self.resource().to_owned(),
      self.component(self.resource_path.as_ref()),
      self.path_escaped,
      v.as_deref(),
      self.fragment(),
    ));
  }

  pub fn fragment(&self) -> Option<&str> {
    self.component(self.resource_fragment.as_ref())
  }

  pub fn set_fragment(&mut self, v: String) {
    self.set_fragment_optional(Some(v));
  }

  pub fn set_fragment_optional(&mut self, v: Option<String>) {
    self.replace_components(Self::from_parts(
      self.resource().to_owned(),
      self.component(self.resource_path.as_ref()),
      self.path_escaped,
      self.query(),
      v.as_deref(),
    ));
  }

  pub fn description(&self) -> Option<&DescriptionData> {
    self.resource_description.as_ref()
  }

  pub fn set_description_optional(&mut self, v: Option<DescriptionData>) {
    self.resource_description = v;
  }

  pub fn mimetype(&self) -> Option<&str> {
    self.mimetype.as_deref()
  }

  pub fn set_mimetype(&mut self, v: String) {
    self.mimetype = Some(v);
  }

  pub fn parameters(&self) -> Option<&str> {
    self.parameters.as_deref()
  }

  pub fn set_parameters(&mut self, v: String) {
    self.parameters = Some(v);
  }

  pub fn encoding(&self) -> Option<&str> {
    self.encoding.as_deref()
  }

  pub fn set_encoding(&mut self, v: String) {
    self.encoding = Some(v);
  }

  pub fn encoded_content(&self) -> Option<&str> {
    self.encoded_content.as_deref()
  }

  pub fn set_encoded_content(&mut self, v: String) {
    self.encoded_content = Some(v);
  }

  pub fn update_resource_data(&mut self, new_resource: String) {
    if let Some(old_path) = self.path() {
      let mut data = Self::new_with_resource(new_resource);
      if data.path() != Some(old_path) {
        self.scheme.take();
      } else if data.path().map(Utf8Path::as_str) != Some(old_path.as_str()) {
        data = Self::from_parts(
          data.resource().to_owned(),
          Some(old_path.as_str()),
          false,
          data.query(),
          data.fragment(),
        );
      }
      self.replace_components(data);
    } else {
      self.set_resource(new_resource);
    }
  }
}

/// Used for [Rule.descriptionData](https://rspack.rs/config/module.html#ruledescriptiondata) and
/// package.json.sideEffects in tree shaking.
#[cacheable]
#[derive(Debug, Clone)]
pub struct DescriptionData {
  /// Path to package.json
  #[cacheable(with=As<PortablePath>)]
  path: PathBuf,

  /// Raw package.json
  #[cacheable(with=AsInner<AsPreset>)]
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

  pub fn into_parts(self) -> (PathBuf, Arc<serde_json::Value>) {
    (self.path, self.json)
  }
}

pub type AdditionalData = anymap::Map<dyn Any + Send + Sync>;

#[cacheable_dyn]
pub trait ParseMetaValue: CloneAny + Send + Sync {
  fn clone_parse_meta(&self) -> Box<dyn ParseMetaValue>;
  fn into_any(self: Box<Self>) -> Box<dyn Any + Send + Sync>;
}

impl Clone for Box<dyn ParseMetaValue> {
  fn clone(&self) -> Self {
    self.clone_parse_meta()
  }
}

impl Debug for dyn ParseMetaValue {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ParseMetaValue").finish_non_exhaustive()
  }
}

#[cacheable_dyn]
impl ParseMetaValue for String {
  fn clone_parse_meta(&self) -> Box<dyn ParseMetaValue> {
    Box::new(self.clone())
  }

  fn into_any(self: Box<Self>) -> Box<dyn Any + Send + Sync> {
    self
  }
}

pub type ParseMeta = FxHashMap<String, Box<dyn ParseMetaValue>>;
