use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;
use std::{borrow::Cow, convert::Infallible};

use once_cell::sync::Lazy;
use regex::{Captures, Regex};

use crate::{parse_resource, AssetInfo, PathData, ResourceParsedData};

pub const FILE_PLACEHOLDER: &str = "[file]";
pub const BASE_PLACEHOLDER: &str = "[base]";
pub const NAME_PLACEHOLDER: &str = "[name]";
pub const PATH_PLACEHOLDER: &str = "[path]";
pub const EXT_PLACEHOLDER: &str = "[ext]";
pub const QUERY_PLACEHOLDER: &str = "[query]";
pub const FRAGMENT_PLACEHOLDER: &str = "[fragment]";
pub const ID_PLACEHOLDER: &str = "[id]";
pub const RUNTIME_PLACEHOLDER: &str = "[runtime]";
pub const URL_PLACEHOLDER: &str = "[url]";
pub static HASH_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[hash(:(\d*))?]").expect("Invalid regex"));
pub static CHUNK_HASH_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[chunkhash(:(\d*))?]").expect("Invalid regex"));
pub static CONTENT_HASH_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[contenthash(:(\d*))?]").expect("Invalid regex"));
pub static FULL_HASH_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[fullhash(:(\d*))?]").expect("Invalid regex"));

static DATA_URI_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^data:([^;,]+)").expect("Invalid regex"));

#[derive(PartialEq, Debug, Hash, Eq, Clone, PartialOrd, Ord)]
enum FilenameKind<F> {
  Template(String),
  Fn(F),
}

/// Filename template string or function
///
/// The function type is generic. The default function type `Arc<dyn FilenameFn>` is thread-safe,
/// implements `Hash` and `Eq`, and its error type is `rspack_error::Error`.
///
/// Other possible function types are `NoFilenameFn` and `LocalJsFilenameFn`
#[derive(PartialEq, Debug, Hash, Eq, Clone, PartialOrd, Ord)]
pub struct Filename<F = Arc<dyn FilenameFn>>(FilenameKind<F>);

impl<F> Filename<F> {
  pub fn from_fn(f: F) -> Self {
    Self(FilenameKind::Fn(f))
  }
}

/// A `never` type of filename function. It marks the filename as template string only.
///
/// The error type of it is `Infallible`.
#[derive(PartialEq, Debug, Hash, Eq, Clone, PartialOrd, Ord)]
pub struct NoFilenameFn(Infallible);

/// Filename template string. No function allowed.
///
/// Its render result is `Result<String, Infallible>`, which can be unwrapped with `ResultInfallibleExt::always_ok`
pub type FilenameTemplate = Filename<NoFilenameFn>;

impl FilenameTemplate {
  pub fn as_str(&self) -> &str {
    match &self.0 {
      FilenameKind::Template(template) => template.as_str(),
      FilenameKind::Fn(no_fn) => match no_fn.0 {},
    }
  }
}
impl LocalFilenameFn for NoFilenameFn {
  type Error = Infallible;

  fn call(
    &self,
    _path_data: &PathData,
    _asset_info: Option<&AssetInfo>,
  ) -> Result<String, Self::Error> {
    match self.0 {}
  }
}

impl From<FilenameTemplate> for Filename {
  fn from(value: FilenameTemplate) -> Self {
    Self(match value.0 {
      FilenameKind::Template(template) => FilenameKind::Template(template),
      FilenameKind::Fn(no_fn) => match no_fn.0 {},
    })
  }
}

/// The minimum requirement for a filename fn.
pub trait LocalFilenameFn {
  type Error;
  fn call(
    &self,
    path_data: &PathData,
    asset_info: Option<&AssetInfo>,
  ) -> Result<String, Self::Error>;
}

/// The default filename fn trait.
pub trait FilenameFn: LocalFilenameFn<Error = rspack_error::Error> + Debug + Send + Sync {}

impl LocalFilenameFn for Arc<dyn FilenameFn> {
  type Error = rspack_error::Error;
  fn call(
    &self,
    path_data: &PathData,
    asset_info: Option<&AssetInfo>,
  ) -> Result<String, Self::Error> {
    self.deref().call(path_data, asset_info)
  }
}

impl<F> From<String> for Filename<F> {
  fn from(value: String) -> Self {
    Self(FilenameKind::Template(value))
  }
}
impl<F> FromStr for Filename<F> {
  type Err = Infallible;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self(FilenameKind::Template(s.to_owned())))
  }
}

fn hash_len(hash: &str, caps: &Captures) -> usize {
  let hash_len = hash.len();
  caps
    .get(2)
    .and_then(|m| m.as_str().parse().ok())
    .unwrap_or(hash_len)
    .min(hash_len)
}

pub fn has_hash_placeholder(template: &str) -> bool {
  HASH_PLACEHOLDER.is_match(template) || FULL_HASH_PLACEHOLDER.is_match(template)
}

impl<F> Filename<F> {
  pub fn template(&self) -> Option<&str> {
    match &self.0 {
      FilenameKind::Template(template) => Some(template.as_str()),
      _ => None,
    }
  }
}

impl<F: LocalFilenameFn> Filename<F> {
  pub fn render(
    &self,
    options: PathData,
    asset_info: Option<&mut AssetInfo>,
  ) -> Result<String, F::Error> {
    let template = match &self.0 {
      FilenameKind::Template(template) => Cow::Borrowed(template.as_str()),
      FilenameKind::Fn(filename_fn) => {
        Cow::Owned(filename_fn.call(&options, asset_info.as_deref())?)
      }
    };
    Ok(render_template(template.as_ref(), options, asset_info))
  }
}

fn render_template(
  template: &str,
  options: PathData,
  mut asset_info: Option<&mut AssetInfo>,
) -> String {
  let mut template = template.to_string();
  if let Some(filename) = options.filename {
    if let Some(caps) = DATA_URI_REGEX.captures(filename) {
      let ext = mime_guess::get_mime_extensions_str(
        caps
          .get(1)
          .expect("should match mime for data uri")
          .as_str(),
      )
      .map(|exts| exts[0]);
      template = template.replace(FILE_PLACEHOLDER, "");
      template = template.replace(QUERY_PLACEHOLDER, "");
      template = template.replace(FRAGMENT_PLACEHOLDER, "");
      template = template.replace(PATH_PLACEHOLDER, "");
      template = template.replace(BASE_PLACEHOLDER, "");
      template = template.replace(NAME_PLACEHOLDER, "");
      template = template.replace(
        EXT_PLACEHOLDER,
        &ext.map(|ext| format!(".{}", ext)).unwrap_or_default(),
      );
    } else if let Some(ResourceParsedData {
      path: file,
      query,
      fragment,
    }) = parse_resource(filename)
    {
      template = template.replace(FILE_PLACEHOLDER, &file.to_string_lossy());
      template = template.replace(
        EXT_PLACEHOLDER,
        &file
          .extension()
          .map(|p| format!(".{}", p.to_string_lossy()))
          .unwrap_or_default(),
      );
      if let Some(base) = file.file_name().map(|p| p.to_string_lossy()) {
        template = template.replace(BASE_PLACEHOLDER, &base);
      }
      if let Some(name) = file.file_stem().map(|p| p.to_string_lossy()) {
        template = template.replace(NAME_PLACEHOLDER, &name);
      }
      template = template.replace(
        PATH_PLACEHOLDER,
        &file
          .parent()
          .map(|p| p.to_string_lossy())
          // "" -> "", "folder" -> "folder/"
          .filter(|p| !p.is_empty())
          .map(|p| p + "/")
          .unwrap_or_default(),
      );
      template = template.replace(QUERY_PLACEHOLDER, &query.unwrap_or_default());
      template = template.replace(FRAGMENT_PLACEHOLDER, &fragment.unwrap_or_default());
    }
  }
  if let Some(content_hash) = options.content_hash {
    if let Some(asset_info) = asset_info.as_mut() {
      // set version as content hash
      asset_info.version = content_hash.to_string();
    }
    template = CONTENT_HASH_PLACEHOLDER
      .replace_all(&template, |caps: &Captures| {
        let content_hash = &content_hash[..hash_len(content_hash, caps)];
        if let Some(asset_info) = asset_info.as_mut() {
          asset_info.set_immutable(true);
          asset_info.set_content_hash(content_hash.to_owned());
        }
        content_hash
      })
      .into_owned();
  }
  if let Some(hash) = options.hash {
    for reg in [&HASH_PLACEHOLDER, &FULL_HASH_PLACEHOLDER] {
      template = reg
        .replace_all(&template, |caps: &Captures| {
          let hash = &hash[..hash_len(hash, caps)];
          if let Some(asset_info) = asset_info.as_mut() {
            asset_info.set_immutable(true);
            asset_info.set_content_hash(hash.to_owned());
          }
          hash
        })
        .into_owned();
    }
  }
  if let Some(chunk) = options.chunk {
    if let Some(id) = &options.id {
      template = template.replace(ID_PLACEHOLDER, id);
    } else if let Some(id) = &chunk.id {
      template = template.replace(ID_PLACEHOLDER, &id.to_string());
    }
    if let Some(name) = chunk.name_for_filename_template() {
      template = template.replace(NAME_PLACEHOLDER, name);
    }
    if let Some(d) = chunk.rendered_hash.as_ref() {
      template = CHUNK_HASH_PLACEHOLDER
        .replace_all(&template, |caps: &Captures| {
          let hash = &**d;
          let hash = &hash[..hash_len(hash, caps)];
          if let Some(asset_info) = asset_info.as_mut() {
            asset_info.set_immutable(true);
            asset_info.set_chunk_hash(hash.to_owned());
          }
          hash
        })
        .into_owned();
    }
  }

  if let Some(id) = &options.id {
    template = template.replace(ID_PLACEHOLDER, id);
  } else if let Some(module) = options.module {
    if let Some(chunk_graph) = options.chunk_graph {
      if let Some(id) = chunk_graph.get_module_id(module.identifier()) {
        template = template.replace(ID_PLACEHOLDER, id);
      }
    }
  }
  template = template.replace(RUNTIME_PLACEHOLDER, options.runtime.unwrap_or("_"));
  if let Some(url) = options.url {
    template = template.replace(URL_PLACEHOLDER, url);
  }
  template
}
