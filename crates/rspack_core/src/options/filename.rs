use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::LazyLock;
use std::{borrow::Cow, convert::Infallible, ptr};

use regex::Regex;
use rspack_cacheable::{
  cacheable,
  with::{AsPreset, Unsupported},
};
use rspack_error::error;
use rspack_macros::MergeFrom;
use rspack_util::atom::Atom;
use rspack_util::ext::CowExt;
use rspack_util::MergeFrom;

use crate::ReplaceAllPlaceholder;
use crate::{parse_resource, AssetInfo, PathData, ResourceParsedData};

static FILE_PLACEHOLDER: &str = "[file]";
static BASE_PLACEHOLDER: &str = "[base]";
static NAME_PLACEHOLDER: &str = "[name]";
static PATH_PLACEHOLDER: &str = "[path]";
static EXT_PLACEHOLDER: &str = "[ext]";
static QUERY_PLACEHOLDER: &str = "[query]";
static FRAGMENT_PLACEHOLDER: &str = "[fragment]";
static ID_PLACEHOLDER: &str = "[id]";
static RUNTIME_PLACEHOLDER: &str = "[runtime]";
static URL_PLACEHOLDER: &str = "[url]";

pub static HASH_PLACEHOLDER: &str = "[hash]";
pub static FULL_HASH_PLACEHOLDER: &str = "[fullhash]";
pub static CHUNK_HASH_PLACEHOLDER: &str = "[chunkhash]";
pub static CONTENT_HASH_PLACEHOLDER: &str = "[contenthash]";

static DATA_URI_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^data:([^;,]+)").expect("Invalid regex"));

#[cacheable]
#[derive(PartialEq, Debug, Hash, Eq, Clone, PartialOrd, Ord, MergeFrom)]
enum FilenameKind<F> {
  Template(#[cacheable(with=AsPreset)] Atom),
  Fn(#[cacheable(with=Unsupported)] F),
}

/// Filename template string or function
///
/// The function type is generic. The default function type `Arc<dyn FilenameFn>` is thread-safe,
/// implements `Hash` and `Eq`, and its error type is `rspack_error::Error`.
///
/// Other possible function types are `NoFilenameFn` and `LocalJsFilenameFn`
#[cacheable]
#[derive(PartialEq, Debug, Hash, Eq, Clone, PartialOrd, Ord)]
pub struct Filename<F = Arc<dyn FilenameFn>>(FilenameKind<F>);

impl<F> Filename<F> {
  pub fn from_fn(f: F) -> Self {
    Self(FilenameKind::Fn(f))
  }
}

impl Hash for dyn FilenameFn + '_ {
  fn hash<H: Hasher>(&self, _: &mut H) {}
}
impl PartialEq for dyn FilenameFn + '_ {
  fn eq(&self, other: &Self) -> bool {
    ptr::eq(self, other)
  }
}
impl Eq for dyn FilenameFn + '_ {}

impl PartialOrd for dyn FilenameFn + '_ {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}
impl Ord for dyn FilenameFn + '_ {
  fn cmp(&self, _: &Self) -> std::cmp::Ordering {
    std::cmp::Ordering::Equal
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
    unreachable!()
  }
}

impl From<FilenameTemplate> for Filename {
  fn from(value: FilenameTemplate) -> Self {
    let FilenameKind::Template(template) = value.0;

    Self(FilenameKind::Template(template))
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

impl<F> From<&str> for Filename<F> {
  fn from(value: &str) -> Self {
    Filename::from_str(value).expect("infallible")
  }
}

#[inline]
fn hash_len(hash: &str, len: Option<usize>) -> usize {
  let hash_len = hash.len();
  len.unwrap_or(hash_len).min(hash_len)
}

pub fn has_hash_placeholder(template: &str) -> bool {
  for key in [HASH_PLACEHOLDER, FULL_HASH_PLACEHOLDER] {
    let offset = key.len() - 1;
    if let Some(start) = template.find(&key[..offset]) {
      if template[start + offset..].find(']').is_some() {
        return true;
      }
    }
  }
  false
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
  // file-level
  if let Some(filename) = options.filename {
    if let Some(caps) = DATA_URI_REGEX.captures(filename) {
      let ext = mime_guess::get_mime_extensions_str(
        caps
          .get(1)
          .expect("should match mime for data uri")
          .as_str(),
      )
      .map(|exts| exts[0]);

      let replacer = options
        .content_hash
        // "XXXX" used for updateHash, so we don't need it here
        .filter(|hash| !hash.contains('X'))
        .unwrap_or("");

      t = t
        .map(|t| t.replace_all(FILE_PLACEHOLDER, ""))
        .map(|t| t.replace_all(QUERY_PLACEHOLDER, ""))
        .map(|t| t.replace_all(FRAGMENT_PLACEHOLDER, ""))
        .map(|t| t.replace_all(PATH_PLACEHOLDER, ""))
        .map(|t| t.replace_all(BASE_PLACEHOLDER, replacer))
        .map(|t| t.replace_all(NAME_PLACEHOLDER, replacer))
        .map(|t| {
          t.replace_all(
            EXT_PLACEHOLDER,
            &ext.map(|ext| format!(".{ext}")).unwrap_or_default(),
          )
        });
    } else if let Some(ResourceParsedData {
      path: file,
      query,
      fragment,
    }) = parse_resource(filename)
    {
      t = t
        .map(|t| t.replace_all(FILE_PLACEHOLDER, file.as_str()))
        .map(|t| {
          t.replace_all(
            EXT_PLACEHOLDER,
            &file
              .extension()
              .map(|p| format!(".{p}"))
              .unwrap_or_default(),
          )
        });

      if let Some(base) = file.file_name() {
        t = t.map(|t| t.replace_all(BASE_PLACEHOLDER, base));
      }
      if let Some(name) = file.file_stem() {
        t = t.map(|t| t.replace_all(NAME_PLACEHOLDER, name));
      }
      t = t
        .map(|t| {
          t.replace_all(
            PATH_PLACEHOLDER,
            &file
              .parent()
              // "" -> "", "folder" -> "folder/"
              .filter(|p| !p.as_str().is_empty())
              .map(|p| p.as_str().to_owned() + "/")
              .unwrap_or_default(),
          )
        })
        .map(|t| t.replace_all(QUERY_PLACEHOLDER, &query.unwrap_or_default()))
        .map(|t| t.replace_all(FRAGMENT_PLACEHOLDER, &fragment.unwrap_or_default()));
    }
  }
  // compilation-level
  if let Some(hash) = options.hash {
    for key in [HASH_PLACEHOLDER, FULL_HASH_PLACEHOLDER] {
      t = t.map(|t| {
        t.replace_all_with_len(key, |len| {
          let hash = &hash[..hash_len(hash, len)];
          if let Some(asset_info) = asset_info.as_mut() {
            asset_info.set_immutable(Some(true));
            asset_info.set_full_hash(hash.to_owned());
          }
          hash
        })
      });
    }
  }
  // shared by chunk-level and module-level
  if let Some(id) = options.id {
    t = t.map(|t| t.replace_all(ID_PLACEHOLDER, id));
  } else if let Some(chunk_id) = options.chunk_id {
    t = t.map(|t| t.replace_all(ID_PLACEHOLDER, chunk_id));
  } else if let Some(module_id) = options.module_id {
    t = t.map(|t| t.replace_all(ID_PLACEHOLDER, module_id));
  }
  if let Some(content_hash) = options.content_hash {
    if let Some(asset_info) = asset_info.as_mut() {
      // set version as content hash
      asset_info.version = content_hash.to_string();
    }
    t = t.map(|t| {
      t.replace_all_with_len(CONTENT_HASH_PLACEHOLDER, |len| {
        let hash: &str = &content_hash[..hash_len(content_hash, len)];
        if let Some(asset_info) = asset_info.as_mut() {
          asset_info.set_immutable(Some(true));
          asset_info.set_content_hash(hash.to_owned());
        }
        hash
      })
    });
  }
  // chunk-level
  if let Some(name) = options.chunk_name {
    t = t.map(|t| t.replace_all(NAME_PLACEHOLDER, name));
  }
  if let Some(hash) = options.chunk_hash {
    t = t.map(|t| {
      t.replace_all_with_len(CHUNK_HASH_PLACEHOLDER, |len| {
        let hash: &str = &hash[..hash_len(hash, len)];
        if let Some(asset_info) = asset_info.as_mut() {
          asset_info.set_immutable(Some(true));
          asset_info.set_chunk_hash(hash.to_owned());
        }
        hash
      })
    });
  }
  // other things
  t = t.map(|t| t.replace_all(RUNTIME_PLACEHOLDER, options.runtime.unwrap_or("_")));
  if let Some(url) = options.url {
    t = t.map(|t| t.replace_all(URL_PLACEHOLDER, url));
  }
  t.into_owned()
}
