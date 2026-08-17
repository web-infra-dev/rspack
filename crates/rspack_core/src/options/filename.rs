use std::{
  borrow::Cow,
  collections::HashMap,
  fmt::Debug,
  hash::{BuildHasherDefault, Hasher},
  ops::{Deref, Range},
  ptr,
  sync::{Arc, LazyLock},
};

use dashmap::DashMap;
use rspack_cacheable::{
  cacheable,
  with::{AsPreset, Unsupported},
};
use rspack_error::ToStringResultToRspackResultExt;
use rspack_hash::{HashDigest, RspackHasher};
use rspack_macros::StringEnum;
use rspack_paths::Utf8PathBuf;
use rspack_util::{MergeFrom, base64};
use ustr::{IdentityHasher, Ustr};

use crate::{AssetInfo, PathData, ResourceParsedData, parse_resource};

pub static HASH_PLACEHOLDER: &str = "[hash]";
pub static FULL_HASH_PLACEHOLDER: &str = "[fullhash]";
pub static CHUNK_HASH_PLACEHOLDER: &str = "[chunkhash]";
pub static CONTENT_HASH_PLACEHOLDER: &str = "[contenthash]";

const MAX_TEMPLATE_LEN: usize = u16::MAX as usize;

type PlaceholderId = u16;
type PlaceholderMap = HashMap<Ustr, PlaceholderId, BuildHasherDefault<IdentityHasher>>;
type CompiledTemplateCache =
  DashMap<Ustr, Arc<CompiledStringTemplate>, BuildHasherDefault<IdentityHasher>>;

static COMPILED_STRING_TEMPLATES: LazyLock<CompiledTemplateCache> = LazyLock::new(Default::default);

#[derive(Debug, Clone, Copy, PartialEq, Eq, StringEnum)]
#[repr(u8)]
enum PlaceholderKind {
  File,
  Base,
  Name,
  Path,
  Ext,
  Query,
  Fragment,
  Id,
  Runtime,
  Url,
  Hash,
  #[string_enum(rename = "fullhash")]
  FullHash,
  #[string_enum(rename = "chunkhash")]
  ChunkHash,
  #[string_enum(rename = "contenthash")]
  ContentHash,
  #[string_enum(fallback)]
  Unknown,
}

#[derive(Debug, Clone, Copy)]
enum PlaceholderParameters {
  None,
  Hash {
    len: Option<u16>,
    encoding: Option<HashDigest>,
  },
}

#[derive(Debug, Clone)]
struct PlaceholderData {
  kind: PlaceholderKind,
  parameters: PlaceholderParameters,
}

#[derive(Debug, Clone)]
enum StringTemplateSegment {
  Plain(Range<u16>),
  Placeholder {
    id: PlaceholderId,
    range: Range<u16>,
  },
}

#[derive(Debug)]
struct CompiledStringTemplate {
  placeholder_indices: PlaceholderMap,
  placeholder_data: Vec<PlaceholderData>,
  segments: Vec<StringTemplateSegment>,
  has_hash_placeholder: bool,
  has_content_hash_placeholder: bool,
}

impl CompiledStringTemplate {
  fn compile(template: &str) -> Self {
    debug_assert!(template.len() <= MAX_TEMPLATE_LEN);

    let mut placeholder_indices = PlaceholderMap::default();
    let mut placeholder_data = Vec::new();
    let mut segments = Vec::new();
    let mut plain_start = 0;
    let mut cursor = 0;

    while let Some(relative_start) = template[cursor..].find('[') {
      let start = cursor + relative_start;
      let Some(relative_end) = template[start + 1..].find(']') else {
        break;
      };
      let end = start + 1 + relative_end;
      let token = &template[start + 1..end];

      let Some((kind, parameters)) = parse_placeholder(token) else {
        cursor = start + 1;
        continue;
      };

      if plain_start < start {
        segments.push(StringTemplateSegment::Plain(to_u16_range(
          plain_start,
          start,
        )));
      }

      let raw = if token == kind.as_str() {
        Ustr::from(kind.as_str())
      } else {
        Ustr::from(token)
      };
      let id = if let Some(id) = placeholder_indices.get(&raw) {
        *id
      } else {
        let id = PlaceholderId::try_from(placeholder_data.len())
          .expect("filename template contains too many unique placeholders");
        placeholder_data.push(PlaceholderData { kind, parameters });
        placeholder_indices.insert(raw, id);
        id
      };
      segments.push(StringTemplateSegment::Placeholder {
        id,
        range: to_u16_range(start, end + 1),
      });

      cursor = end + 1;
      plain_start = cursor;
    }

    if plain_start < template.len() {
      segments.push(StringTemplateSegment::Plain(to_u16_range(
        plain_start,
        template.len(),
      )));
    }

    Self {
      placeholder_indices,
      placeholder_data,
      segments,
      has_hash_placeholder: has_hash_placeholder_uncompiled(template),
      has_content_hash_placeholder: has_content_hash_placeholder_uncompiled(template),
    }
  }

  fn contains_kind(&self, kind: PlaceholderKind) -> bool {
    self.placeholder_indices.values().any(|id| {
      self
        .placeholder_data
        .get(usize::from(*id))
        .is_some_and(|data| data.kind == kind)
    })
  }
}

fn to_u16_range(start: usize, end: usize) -> Range<u16> {
  u16::try_from(start).expect("filename template range start exceeds u16::MAX")
    ..u16::try_from(end).expect("filename template range end exceeds u16::MAX")
}

fn parse_placeholder(token: &str) -> Option<(PlaceholderKind, PlaceholderParameters)> {
  let kind = PlaceholderKind::from(token);
  if kind == PlaceholderKind::Unknown {
    return parse_hash_placeholder(token);
  }

  let parameters = if matches!(
    kind,
    PlaceholderKind::Hash
      | PlaceholderKind::FullHash
      | PlaceholderKind::ChunkHash
      | PlaceholderKind::ContentHash
  ) {
    PlaceholderParameters::Hash {
      len: None,
      encoding: None,
    }
  } else {
    PlaceholderParameters::None
  };
  Some((kind, parameters))
}

fn parse_hash_placeholder(token: &str) -> Option<(PlaceholderKind, PlaceholderParameters)> {
  let (kind, parameters) = token.split_once(':')?;
  let kind = PlaceholderKind::from(kind);
  if !matches!(
    kind,
    PlaceholderKind::Hash
      | PlaceholderKind::FullHash
      | PlaceholderKind::ChunkHash
      | PlaceholderKind::ContentHash
  ) {
    return None;
  }

  let mut configs = parameters.split(':');
  let first = configs.next()?;
  let (len, encoding) = if first == "base64" {
    (
      configs.next()?.parse::<usize>().ok()?,
      Some(HashDigest::Base64),
    )
  } else {
    (first.parse::<usize>().ok()?, None)
  };

  Some((
    kind,
    PlaceholderParameters::Hash {
      len: Some(u16::try_from(len).unwrap_or(u16::MAX)),
      encoding,
    },
  ))
}

fn assert_template_len(template: &str) {
  assert!(
    template.len() <= MAX_TEMPLATE_LEN,
    "filename template exceeds {MAX_TEMPLATE_LEN} bytes (got {})",
    template.len()
  );
}

fn intern_template(template: &str) -> Ustr {
  assert_template_len(template);
  Ustr::from(template)
}

fn get_or_compile(template: Ustr) -> Arc<CompiledStringTemplate> {
  if let Some(compiled) = COMPILED_STRING_TEMPLATES.get(&template) {
    return Arc::clone(compiled.value());
  }

  let compiled = Arc::new(CompiledStringTemplate::compile(template.as_str()));
  Arc::clone(
    COMPILED_STRING_TEMPLATES
      .entry(template)
      .or_insert(compiled)
      .value(),
  )
}

#[cacheable]
#[derive(PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
enum FilenameKind {
  Template(#[cacheable(with=AsPreset)] Ustr),
  Fn(#[cacheable(with=Unsupported)] Arc<dyn FilenameFn>),
}
impl Debug for FilenameKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Template(template) => f.debug_tuple("Template").field(&template.as_str()).finish(),
      Self::Fn(filename_fn) => f.debug_tuple("Fn").field(filename_fn).finish(),
    }
  }
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
    match self.0 {
      FilenameKind::Template(template) => {
        let compiled = get_or_compile(template);
        compiled.has_hash_placeholder
          || compiled.contains_kind(PlaceholderKind::Hash)
          || compiled.contains_kind(PlaceholderKind::FullHash)
      }
      FilenameKind::Fn(_) => true,
    }
  }

  pub fn has_content_hash_placeholder(&self) -> bool {
    match self.0 {
      FilenameKind::Template(template) => {
        let compiled = get_or_compile(template);
        compiled.has_content_hash_placeholder
          || compiled.contains_kind(PlaceholderKind::ContentHash)
      }
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
    match &self.0 {
      FilenameKind::Template(template) => {
        let compiled = get_or_compile(*template);
        Ok(render_template(
          template.as_str(),
          &compiled,
          options,
          asset_info,
        ))
      }
      FilenameKind::Fn(filename_fn) => {
        let template = filename_fn.call(&options, asset_info.as_deref()).await?;
        assert_template_len(&template);
        let compiled = CompiledStringTemplate::compile(&template);
        Ok(render_template(&template, &compiled, options, asset_info))
      }
    }
  }
}

impl rspack_hash::RspackHash for Filename {
  fn hash(&self, state: &mut RspackHasher) {
    if let FilenameKind::Template(template) = &self.0 {
      template.hash(state);
    }
  }
}

impl MergeFrom for Filename {
  fn merge_from(self, other: &Self) -> Self {
    other.clone()
  }
}

impl From<String> for Filename {
  fn from(value: String) -> Self {
    Self(FilenameKind::Template(intern_template(&value)))
  }
}
impl From<&Utf8PathBuf> for Filename {
  fn from(value: &Utf8PathBuf) -> Self {
    Self(FilenameKind::Template(intern_template(value.as_str())))
  }
}
impl From<&str> for Filename {
  fn from(value: &str) -> Self {
    Self(FilenameKind::Template(intern_template(value)))
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

impl std::hash::Hash for dyn FilenameFn + '_ {
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
fn hash_len(hash: &str, len: Option<u16>) -> usize {
  let hash_len = hash.len();
  len.map_or(hash_len, usize::from).min(hash_len)
}

pub fn has_hash_placeholder(template: &str) -> bool {
  let compiled = get_or_compile(intern_template(template));
  compiled.has_hash_placeholder
}

fn has_hash_placeholder_uncompiled(template: &str) -> bool {
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
  let compiled = get_or_compile(intern_template(template));
  compiled.has_content_hash_placeholder
}

fn has_content_hash_placeholder_uncompiled(template: &str) -> bool {
  let offset = CONTENT_HASH_PLACEHOLDER.len() - 1;
  if let Some(start) = template.find(&CONTENT_HASH_PLACEHOLDER[..offset])
    && template[start + offset..].find(']').is_some()
  {
    return true;
  }
  false
}

#[derive(Debug, Default)]
struct FileReplacements {
  file: Option<String>,
  base: Option<String>,
  name: Option<String>,
  path: Option<String>,
  ext: Option<String>,
  query: Option<String>,
  fragment: Option<String>,
}

impl FileReplacements {
  fn new(options: PathData<'_>) -> Self {
    let Some(filename) = options.filename else {
      return Self::default();
    };

    if let Ok(caps) = data_uri(filename) {
      let replacer = options
        .content_hash
        // "XXXX" is used for updateHash and must not become a filename here.
        .filter(|hash| !hash.contains('X'))
        .unwrap_or("")
        .to_owned();
      let ext = mime_guess::get_mime_extensions_str(caps)
        .map(|exts| format!(".{}", exts[0]))
        .unwrap_or_default();

      return Self {
        file: Some(String::new()),
        query: Some(String::new()),
        fragment: Some(String::new()),
        path: Some(String::new()),
        base: Some(replacer.clone()),
        name: Some(replacer),
        ext: Some(ext),
      };
    }

    let Some(ResourceParsedData {
      path: file,
      query,
      fragment,
    }) = parse_resource(filename)
    else {
      return Self::default();
    };

    let ext = file
      .extension()
      .map(|extension| format!(".{extension}"))
      .unwrap_or_default();
    let base = file.file_name().map(ToOwned::to_owned);
    let name = file.file_stem().map(ToOwned::to_owned);
    let path = file
      .parent()
      // "" -> "", "folder" -> "folder/"
      .filter(|path| !path.as_str().is_empty())
      .map(|path| path.as_str().to_owned() + "/")
      .unwrap_or_default();

    Self {
      file: Some(file.as_str().to_owned()),
      ext: Some(ext),
      base,
      name,
      path: Some(path),
      query: Some(query.unwrap_or_default()),
      fragment: Some(fragment.unwrap_or_default()),
    }
  }
}

fn render_template(
  template: &str,
  compiled: &CompiledStringTemplate,
  options: PathData<'_>,
  mut asset_info: Option<&mut AssetInfo>,
) -> String {
  if let Some(content_hash) = options.content_hash
    && let Some(asset_info) = asset_info.as_deref_mut()
  {
    // Set version even when the template has no content hash placeholder.
    asset_info.version = content_hash.to_string();
  }

  let file_replacements = FileReplacements::new(options);
  let mut output = String::with_capacity(template.len());
  render_compiled_template(
    template,
    compiled,
    options,
    &file_replacements,
    &mut asset_info,
    &mut output,
  );
  output
}

fn render_compiled_template(
  template: &str,
  compiled: &CompiledStringTemplate,
  options: PathData<'_>,
  file_replacements: &FileReplacements,
  asset_info: &mut Option<&mut AssetInfo>,
  output: &mut String,
) {
  for segment in &compiled.segments {
    match segment {
      StringTemplateSegment::Plain(range) => {
        output.push_str(&template[usize::from(range.start)..usize::from(range.end)]);
      }
      StringTemplateSegment::Placeholder { id, range } => {
        let data = compiled
          .placeholder_data
          .get(usize::from(*id))
          .expect("filename placeholder segment references missing side-table data");
        if !render_placeholder(data, options, file_replacements, asset_info, output) {
          output.push_str(&template[usize::from(range.start)..usize::from(range.end)]);
        }
      }
    }
  }
}

fn render_placeholder(
  data: &PlaceholderData,
  options: PathData<'_>,
  file_replacements: &FileReplacements,
  asset_info: &mut Option<&mut AssetInfo>,
  output: &mut String,
) -> bool {
  let kind = data.kind;
  match kind {
    PlaceholderKind::File => try_render_value(file_replacements.file.as_deref(), output),
    PlaceholderKind::Base => try_render_value(file_replacements.base.as_deref(), output),
    PlaceholderKind::Name => {
      try_render_value(file_replacements.name.as_deref(), output)
        || try_render_value(options.chunk_name.or(options.chunk_id), output)
    }
    PlaceholderKind::Path => try_render_value(file_replacements.path.as_deref(), output),
    PlaceholderKind::Ext => try_render_value(file_replacements.ext.as_deref(), output),
    PlaceholderKind::Query => try_render_value(file_replacements.query.as_deref(), output),
    PlaceholderKind::Fragment => try_render_value(file_replacements.fragment.as_deref(), output),
    PlaceholderKind::Id => try_render_value(
      options.id.or(options.chunk_id).or(options.module_id),
      output,
    ),
    PlaceholderKind::Runtime => try_render_value(Some(options.runtime.unwrap_or("_")), output),
    PlaceholderKind::Url => try_render_value(options.url, output),
    PlaceholderKind::Hash => {
      try_render_hash(options.hash, kind, data.parameters, asset_info, output)
    }
    PlaceholderKind::FullHash => {
      try_render_hash(options.hash, kind, data.parameters, asset_info, output)
    }
    PlaceholderKind::ContentHash => try_render_hash(
      options.content_hash,
      kind,
      data.parameters,
      asset_info,
      output,
    ),
    PlaceholderKind::ChunkHash => try_render_hash(
      options.chunk_hash,
      kind,
      data.parameters,
      asset_info,
      output,
    ),
    PlaceholderKind::Unknown => false,
  }
}

fn try_render_value(replacement: Option<&str>, output: &mut String) -> bool {
  let Some(replacement) = replacement else {
    return false;
  };
  output.push_str(replacement);
  true
}

fn try_render_hash(
  hash: Option<&str>,
  kind: PlaceholderKind,
  parameters: PlaceholderParameters,
  asset_info: &mut Option<&mut AssetInfo>,
  output: &mut String,
) -> bool {
  let Some(hash) = hash else {
    return false;
  };
  let PlaceholderParameters::Hash { len, encoding } = parameters else {
    unreachable!("hash placeholder must have hash parameters");
  };

  let content: Cow<'_, str> = match encoding {
    None => hash.into(),
    Some(HashDigest::Base64) => base64::encode_to_string(hash).into(),
    Some(encoding) => unreachable!("unsupported filename hash encoding: {encoding:?}"),
  };
  let content = &content[..hash_len(&content, len)];

  if let Some(asset_info) = asset_info.as_deref_mut() {
    asset_info.set_immutable(Some(true));
    match kind {
      PlaceholderKind::Hash | PlaceholderKind::FullHash => {
        asset_info.set_full_hash(content.to_string());
      }
      PlaceholderKind::ContentHash => {
        asset_info.set_content_hash(content.to_string());
      }
      PlaceholderKind::ChunkHash => {
        asset_info.set_chunk_hash(content.to_string());
      }
      _ => unreachable!("non-hash placeholder passed to hash renderer"),
    }
  }

  output.push_str(content);
  true
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
