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
use rspack_util::identifier::{
  escape_literal_fragment_marker, insert_zero_width_space_for_fragment,
  strip_zero_width_space_for_fragment,
};
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

fn decode_resource_part(value: &str) -> Option<String> {
  match strip_zero_width_space_for_fragment(value) {
    Cow::Borrowed(_) => None,
    Cow::Owned(decoded) => Some(decoded),
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ResourceData {
  resource: String,
  resource_path: Option<Range<u32>>,
  resource_query: Option<Range<u32>>,
  resource_fragment: Option<Range<u32>>,
  // Ranges use this buffer when independently supplied components differ from the request.
  component_resource: Option<Box<str>>,
  #[cacheable(with=Skip)]
  decoded_path: OnceCell<Option<Utf8PathBuf>>,
  #[cacheable(with=Skip)]
  decoded_query: OnceCell<Option<String>>,
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
    let resource_query = query.map(|query| path_end..resource_offset(path.len() + query.len()));
    let resource_fragment =
      fragment.map(|fragment| resource_end - resource_offset(fragment.len())..resource_end);
    Self {
      resource,
      resource_path: Some(0..path_end),
      resource_query,
      resource_fragment,
      ..Default::default()
    }
  }

  /// Construct the escaped request from components returned by the resolver.
  pub fn new_with_resolved_path(path: Utf8PathBuf, query: &str, fragment: &str) -> Self {
    let path = match insert_zero_width_space_for_fragment(path.as_str()) {
      Cow::Borrowed(_) => path.into_string(),
      Cow::Owned(path) => path,
    };
    let query = insert_zero_width_space_for_fragment(query);
    Self::from_concatenated_parts(Some(path), Some(query.as_ref()), Some(fragment))
  }

  pub fn new_with_path(
    resource: String,
    path: Utf8PathBuf,
    query: Option<String>,
    fragment: Option<String>,
  ) -> Self {
    let path = escape_literal_fragment_marker(path.as_str());
    let query = query.as_deref().map(escape_literal_fragment_marker);
    Self::from_parts(resource, Some(&path), query.as_deref(), fragment.as_deref())
  }

  fn from_parts(
    resource: String,
    path: Option<&str>,
    query: Option<&str>,
    fragment: Option<&str>,
  ) -> Self {
    let fragment_start = resource.len().saturating_sub(fragment.map_or(0, str::len));
    let query_start = fragment_start.saturating_sub(query.map_or(0, str::len));
    let resource_path = path.and_then(|path| Self::locate_component(&resource, 0, path));
    let resource_query =
      query.and_then(|query| Self::locate_component(&resource, query_start, query));
    let resource_fragment =
      fragment.and_then(|fragment| Self::locate_component(&resource, fragment_start, fragment));
    if resource_path.is_some() == path.is_some()
      && resource_query.is_some() == query.is_some()
      && resource_fragment.is_some() == fragment.is_some()
    {
      Self {
        resource,
        resource_path,
        resource_query,
        resource_fragment,
        ..Default::default()
      }
    } else {
      let mut data = Self::from_concatenated_parts(path.map(str::to_owned), query, fragment);
      let components = std::mem::replace(&mut data.resource, resource);
      data.component_resource = Some(components.into_boxed_str());
      data
    }
  }

  fn locate_component(resource: &str, offset: usize, value: &str) -> Option<Range<u32>> {
    let end = offset.saturating_add(value.len());
    (resource.get(offset..end) == Some(value))
      .then(|| resource_offset(offset)..resource_offset(end))
  }

  fn component(&self, range: &Range<u32>) -> &str {
    let resource = self.component_resource.as_deref().unwrap_or(&self.resource);
    &resource[range.start as usize..range.end as usize]
  }

  fn raw_path(&self) -> Option<&str> {
    self
      .resource_path
      .as_ref()
      .map(|range| self.component(range))
  }

  fn raw_query(&self) -> Option<&str> {
    self
      .resource_query
      .as_ref()
      .map(|range| self.component(range))
  }

  fn replace_components(&mut self, data: Self) {
    self.resource = data.resource;
    self.resource_path = data.resource_path;
    self.resource_query = data.resource_query;
    self.resource_fragment = data.resource_fragment;
    self.component_resource = data.component_resource;
    self.decoded_path = data.decoded_path;
    self.decoded_query = data.decoded_query;
  }

  fn from_concatenated_parts(
    path: Option<String>,
    query: Option<&str>,
    fragment: Option<&str>,
  ) -> Self {
    let path_present = path.is_some();
    let mut resource = path.unwrap_or_default();
    let path_end = resource_offset(resource.len());
    let resource_query = query.map(|query| {
      resource.push_str(query);
      path_end..resource_offset(resource.len())
    });
    let resource_fragment = fragment.map(|fragment| {
      let start = resource_offset(resource.len());
      resource.push_str(fragment);
      start..resource_offset(resource.len())
    });
    Self {
      resource,
      resource_path: path_present.then_some(0..path_end),
      resource_query,
      resource_fragment,
      ..Default::default()
    }
  }

  /// Build a virtual resource from known components without parsing its delimiters.
  pub fn with_matching_path(&self, path: String) -> Self {
    self.with_matching_components(Some(path))
  }

  /// Match separately supplied components without changing the resource used by the module.
  pub fn for_rule_matching(&self) -> Cow<'_, Self> {
    if self.component_resource.is_some() {
      Cow::Owned(self.with_matching_components(self.path().map(|path| path.as_str().to_owned())))
    } else {
      Cow::Borrowed(self)
    }
  }

  fn with_matching_components(&self, path: Option<String>) -> Self {
    Self {
      resource_description: self.resource_description.clone(),
      mimetype: self.mimetype.clone(),
      parameters: self.parameters.clone(),
      encoding: self.encoding.clone(),
      encoded_content: self.encoded_content.clone(),
      context: self.context.clone(),
      // These temporary match components already contain their final values.
      decoded_path: OnceCell::with_value(None),
      decoded_query: OnceCell::with_value(None),
      ..Self::from_concatenated_parts(path, self.query(), self.fragment())
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
    &self.resource
  }

  pub fn set_resource(&mut self, v: String) {
    self.replace_components(Self::from_parts(
      v,
      self.raw_path(),
      self.raw_query(),
      self.fragment(),
    ));
  }

  pub fn path(&self) -> Option<&Utf8Path> {
    let path = self.raw_path()?;
    Some(
      self
        .decoded_path
        .get_or_init(|| decode_resource_part(path).map(Utf8PathBuf::from))
        .as_deref()
        .unwrap_or_else(|| Utf8Path::new(path)),
    )
  }

  pub fn set_path<P: Into<Utf8PathBuf>>(&mut self, v: P) {
    let new_path = v.into();
    if let Some(path) = self.path()
      && path != new_path
    {
      self.scheme.take();
      let new_path = escape_literal_fragment_marker(new_path.as_str());
      self.replace_components(Self::from_parts(
        self.resource().to_owned(),
        Some(&new_path),
        self.raw_query(),
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
    let query = self.raw_query()?;
    Some(
      self
        .decoded_query
        .get_or_init(|| decode_resource_part(query))
        .as_deref()
        .unwrap_or(query),
    )
  }

  pub fn set_query(&mut self, v: String) {
    self.set_query_optional(Some(v));
  }

  pub fn set_query_optional(&mut self, v: Option<String>) {
    let query = v.as_deref().map(escape_literal_fragment_marker);
    self.replace_components(Self::from_parts(
      self.resource().to_owned(),
      self.raw_path(),
      query.as_deref(),
      self.fragment(),
    ));
  }

  pub fn fragment(&self) -> Option<&str> {
    self
      .resource_fragment
      .as_ref()
      .map(|range| self.component(range))
  }

  pub fn set_fragment(&mut self, v: String) {
    self.set_fragment_optional(Some(v));
  }

  pub fn set_fragment_optional(&mut self, v: Option<String>) {
    self.replace_components(Self::from_parts(
      self.resource().to_owned(),
      self.raw_path(),
      self.raw_query(),
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
          self.raw_path(),
          data.raw_query(),
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
