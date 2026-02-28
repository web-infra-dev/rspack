use std::{
  borrow::Cow,
  fmt::Debug,
  hash::{Hash, Hasher},
  ops::Deref,
  ptr,
  sync::Arc,
};

use rspack_cacheable::{
  cacheable,
  with::{AsPreset, Unsupported},
};
use rspack_error::ToStringResultToRspackResultExt;
use rspack_paths::Utf8PathBuf;
use rspack_util::{MergeFrom, atom::Atom, base64, ext::CowExt};

use crate::{AssetInfo, PathData, ReplaceAllPlaceholder, ResourceParsedData, parse_resource};

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

#[cacheable]
#[derive(PartialEq, Debug, Hash, Eq, Clone, PartialOrd, Ord)]
enum FilenameKind {
  Template(#[cacheable(with=AsPreset)] Atom),
  Fn(#[cacheable(with=Unsupported)] Arc<dyn FilenameFn>),
}

/// Filename placeholders or function
///
/// The function type is generic. The default function type `Arc<dyn FilenameFn>` is thread-safe,
/// implements `Hash` and `Eq`, and its error type is `rspack_error::Error`.
///
/// Other possible function types are `NoFilenameFn` and `LocalJsFilenameFn`
#[cacheable]
#[derive(PartialEq, Debug, Hash, Eq, Clone, PartialOrd, Ord)]
pub struct Filename(FilenameKind);

impl Filename {
  pub fn as_str(&self) -> &str {
    self.template().unwrap_or("")
  }
  pub fn has_hash_placeholder(&self) -> bool {
    match &self.0 {
      FilenameKind::Template(atom) => has_hash_placeholder(atom.as_str()),
      FilenameKind::Fn(_) => true,
    }
  }
  pub fn has_content_hash_placeholder(&self) -> bool {
    match &self.0 {
      FilenameKind::Template(atom) => has_content_hash_placeholder(atom.as_str()),
      FilenameKind::Fn(_) => true,
    }
  }
  pub fn template(&self) -> Option<&str> {
    match &self.0 {
      FilenameKind::Template(template) => Some(template.as_str()),
      _ => None,
    }
  }

  pub async fn render(
    &self,
    options: PathData<'_>,
    asset_info: Option<&mut AssetInfo>,
  ) -> rspack_error::Result<String> {
    let template = match &self.0 {
      FilenameKind::Template(template) => Cow::Borrowed(template.as_str()),
      FilenameKind::Fn(filename_fn) => {
        Cow::Owned(filename_fn.call(&options, asset_info.as_deref()).await?)
      }
    };
    Ok(render_template(template, options, asset_info))
  }
}

impl MergeFrom for Filename {
  fn merge_from(self, other: &Self) -> Self {
    other.clone()
  }
}

impl From<String> for Filename {
  fn from(value: String) -> Self {
    Self(FilenameKind::Template(Atom::from(value)))
  }
}
impl From<&Utf8PathBuf> for Filename {
  fn from(value: &Utf8PathBuf) -> Self {
    Self(FilenameKind::Template(Atom::from(value.as_str())))
  }
}
impl From<&str> for Filename {
  fn from(value: &str) -> Self {
    Self(FilenameKind::Template(Atom::from(value)))
  }
}
impl From<Arc<dyn FilenameFn>> for Filename {
  fn from(value: Arc<dyn FilenameFn>) -> Self {
    Self(FilenameKind::Fn(value))
  }
}

/// The minimum requirement for a filename fn.
#[async_trait::async_trait]
pub trait LocalFilenameFn {
  async fn call(
    &self,
    path_data: &PathData,
    asset_info: Option<&AssetInfo>,
  ) -> rspack_error::Result<String>;
}

/// The default filename fn trait.
pub trait FilenameFn: LocalFilenameFn + Debug + Send + Sync {}

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

#[async_trait::async_trait]
impl LocalFilenameFn for Arc<dyn FilenameFn> {
  async fn call(
    &self,
    path_data: &PathData,
    asset_info: Option<&AssetInfo>,
  ) -> rspack_error::Result<String> {
    self
      .deref()
      .call(path_data, asset_info)
      .await
      .to_rspack_result_with_message(|e| {
        format!("Failed to render filename function: {e}. Did you return the correct filename?")
      })
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
    if let Some(start) = template.find(&key[..offset])
      && template[start + offset..].find(']').is_some()
    {
      return true;
    }
  }
  false
}

pub fn has_content_hash_placeholder(template: &str) -> bool {
  let offset = CONTENT_HASH_PLACEHOLDER.len() - 1;
  if let Some(start) = template.find(&CONTENT_HASH_PLACEHOLDER[..offset])
    && template[start + offset..].find(']').is_some()
  {
    return true;
  }
  false
}

fn render_template(
  template: Cow<str>,
  options: PathData,
  mut asset_info: Option<&mut AssetInfo>,
) -> String {
  let mut t = template;
  // file-level
  if let Some(filename) = options.filename {
    if let Ok(caps) = data_uri(filename) {
      let ext = mime_guess::get_mime_extensions_str(caps).map(|exts| exts[0]);

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
        t.replace_all_with_len(key, |len, need_base64| {
          let content: Cow<str> = if need_base64 {
            base64::encode_to_string(hash).into()
          } else {
            hash.into()
          };
          let content = content.map(|s| s[..hash_len(s, len)].into());
          if let Some(asset_info) = asset_info.as_mut() {
            asset_info.set_immutable(Some(true));
            asset_info.set_full_hash(content.to_string());
          }
          content
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
      t.replace_all_with_len(CONTENT_HASH_PLACEHOLDER, |len, need_base64| {
        let content: Cow<str> = if need_base64 {
          base64::encode_to_string(content_hash).into()
        } else {
          content_hash.into()
        };
        let content = content.map(|s| s[..hash_len(s, len)].into());
        if let Some(asset_info) = asset_info.as_mut() {
          asset_info.set_immutable(Some(true));
          asset_info.set_content_hash(content.to_string());
        }
        content
      })
    });
  }
  // chunk-level
  if let Some(name) = options.chunk_name {
    t = t.map(|t| t.replace_all(NAME_PLACEHOLDER, name));
  } else if let Some(id) = options.chunk_id {
    t = t.map(|t| t.replace_all(NAME_PLACEHOLDER, id));
  }
  if let Some(hash) = options.chunk_hash {
    t = t.map(|t| {
      t.replace_all_with_len(CHUNK_HASH_PLACEHOLDER, |len, need_base64| {
        let content: Cow<str> = if need_base64 {
          base64::encode_to_string(hash).into()
        } else {
          hash.into()
        };
        let content = content.map(|s| s[..hash_len(s, len)].into());
        if let Some(asset_info) = asset_info.as_mut() {
          asset_info.set_immutable(Some(true));
          asset_info.set_chunk_hash(content.to_string());
        }
        content
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

fn data_uri(mut input: &str) -> winnow::ModalResult<&str> {
  use winnow::{combinator::preceded, prelude::*, token::take_till};

  preceded("data:", take_till(1.., (';', ','))).parse_next(&mut input)
}

#[test]
fn test_data_uri() {
  assert_eq!(data_uri("data:good").ok(), Some("good"));
  assert_eq!(data_uri("data:g;ood").ok(), Some("g"));
  assert_eq!(data_uri("data:;ood").ok(), None);
}
