use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;
use std::{borrow::Cow, convert::Infallible};

use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use rspack_error::error;
use rspack_macros::MergeFrom;
use rspack_util::atom::Atom;
use rspack_util::ext::CowExt;
use rspack_util::MergeFrom;

use crate::{parse_resource, AssetInfo, PathData, ResourceParsedData};

pub static FILE_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[file\]").expect("Should generate regex"));
pub static BASE_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[base\]").expect("Should generate regex"));
pub static NAME_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[name\]").expect("Should generate regex"));
pub static PATH_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[path\]").expect("Should generate regex"));
pub static EXT_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[ext\]").expect("Should generate regex"));
pub static QUERY_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[query\]").expect("Should generate regex"));
pub static FRAGMENT_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[fragment\]").expect("Should generate regex"));
pub static ID_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[id\]").expect("Should generate regex"));
pub static RUNTIME_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[runtime\]").expect("Should generate regex"));
pub static URL_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[url\]").expect("Should generate regex"));
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

#[derive(PartialEq, Debug, Hash, Eq, Clone, PartialOrd, Ord, MergeFrom)]
enum FilenameKind<F> {
  Template(Atom),
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

impl<F: Clone> MergeFrom for Filename<F> {
  fn merge_from(self, other: &Self) -> Self {
    other.clone()
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
    self.deref().call(path_data, asset_info).map_err(|err| {
      error!(
        "Failed to render filename function: {}. Did you return the correct filename?",
        err.to_string()
      )
    })
  }
}

impl<F> From<String> for Filename<F> {
  fn from(value: String) -> Self {
    Self(FilenameKind::Template(Atom::from(value)))
  }
}
impl<F> FromStr for Filename<F> {
  type Err = Infallible;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self(FilenameKind::Template(Atom::from(s))))
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
    Ok(render_template(template, options, asset_info))
  }
}

fn render_template(
  template: Cow<str>,
  options: PathData,
  mut asset_info: Option<&mut AssetInfo>,
) -> String {
  let mut t = template;
  if let Some(filename) = options.filename {
    if let Some(caps) = DATA_URI_REGEX.captures(filename) {
      let ext = mime_guess::get_mime_extensions_str(
        caps
          .get(1)
          .expect("should match mime for data uri")
          .as_str(),
      )
      .map(|exts| exts[0]);
      t = t
        .map(|t| FILE_PLACEHOLDER.replace(t, ""))
        .map(|t| QUERY_PLACEHOLDER.replace(t, ""))
        .map(|t| FRAGMENT_PLACEHOLDER.replace(t, ""))
        .map(|t| PATH_PLACEHOLDER.replace(t, ""))
        .map(|t| BASE_PLACEHOLDER.replace(t, ""))
        .map(|t| NAME_PLACEHOLDER.replace(t, ""))
        .map(|t| {
          EXT_PLACEHOLDER.replace(t, &ext.map(|ext| format!(".{}", ext)).unwrap_or_default())
        });
    } else if let Some(ResourceParsedData {
      path: file,
      query,
      fragment,
    }) = parse_resource(filename)
    {
      t = t
        .map(|t| FILE_PLACEHOLDER.replace(t, file.to_string_lossy()))
        .map(|t| {
          EXT_PLACEHOLDER.replace(
            t,
            file
              .extension()
              .map(|p| format!(".{}", p.to_string_lossy()))
              .unwrap_or_default(),
          )
        });

      if let Some(base) = file.file_name().map(|p| p.to_string_lossy()) {
        t = t.map(|t| BASE_PLACEHOLDER.replace(t, &base));
      }
      if let Some(name) = file.file_stem().map(|p| p.to_string_lossy()) {
        t = t.map(|t| NAME_PLACEHOLDER.replace(t, &name));
      }
      t = t
        .map(|t| {
          PATH_PLACEHOLDER.replace(
            t,
            &file
              .parent()
              .map(|p| p.to_string_lossy())
              // "" -> "", "folder" -> "folder/"
              .filter(|p| !p.is_empty())
              .map(|p| p + "/")
              .unwrap_or_default(),
          )
        })
        .map(|t| QUERY_PLACEHOLDER.replace(t, &query.unwrap_or_default()))
        .map(|t| FRAGMENT_PLACEHOLDER.replace(t, &fragment.unwrap_or_default()));
    }
  }
  if let Some(content_hash) = options.content_hash {
    if let Some(asset_info) = asset_info.as_mut() {
      // set version as content hash
      asset_info.version = content_hash.to_string();
    }
    t = t.map(|t| {
      CONTENT_HASH_PLACEHOLDER.replace_all(t, |caps: &Captures| {
        let content_hash = &content_hash[..hash_len(content_hash, caps)];
        if let Some(asset_info) = asset_info.as_mut() {
          asset_info.set_immutable(true);
          asset_info.set_content_hash(content_hash.to_owned());
        }
        content_hash
      })
    });
  }
  if let Some(hash) = options.hash {
    for reg in [&HASH_PLACEHOLDER, &FULL_HASH_PLACEHOLDER] {
      t = t.map(|t| {
        reg.replace_all(t, |caps: &Captures| {
          let hash = &hash[..hash_len(hash, caps)];
          if let Some(asset_info) = asset_info.as_mut() {
            asset_info.set_immutable(true);
            asset_info.set_content_hash(hash.to_owned());
          }
          hash
        })
      });
    }
  }
  if let Some(chunk) = options.chunk {
    if let Some(id) = &options.id {
      t = t.map(|t| ID_PLACEHOLDER.replace(t, *id));
    } else if let Some(id) = &chunk.id {
      t = t.map(|t| ID_PLACEHOLDER.replace(t, id));
    }
    if let Some(name) = chunk.name_for_filename_template() {
      t = t.map(|t| NAME_PLACEHOLDER.replace(t, name));
    }
    if let Some(d) = chunk.rendered_hash.as_ref() {
      t = t.map(|t| {
        CHUNK_HASH_PLACEHOLDER.replace_all(t, |caps: &Captures| {
          let hash = &**d;
          let hash = &hash[..hash_len(hash, caps)];
          if let Some(asset_info) = asset_info.as_mut() {
            asset_info.set_immutable(true);
            asset_info.set_chunk_hash(hash.to_owned());
          }
          hash
        })
      });
    }
  }

  if let Some(id) = &options.id {
    t = t.map(|t| ID_PLACEHOLDER.replace(t, *id));
  } else if let Some(module) = options.module {
    if let Some(chunk_graph) = options.chunk_graph {
      if let Some(id) = chunk_graph.get_module_id(module.identifier()) {
        t = t.map(|t| ID_PLACEHOLDER.replace(t, id));
      }
    }
  }
  t = t.map(|t| RUNTIME_PLACEHOLDER.replace(t, options.runtime.unwrap_or("_")));
  if let Some(url) = options.url {
    t = t.map(|t| URL_PLACEHOLDER.replace(t, url));
  }
  t.into_owned()
}
