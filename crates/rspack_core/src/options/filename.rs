use std::{
  borrow::Cow,
  fmt::Debug,
  hash::{BuildHasherDefault, Hasher},
  ops::{Deref, Range},
  ptr,
  sync::{Arc, LazyLock, OnceLock},
};

use dashmap::DashMap;
use memchr::memchr2_iter;
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

const MAX_TEMPLATE_LEN: usize = u16::MAX as usize;
const ESTIMATED_GROWTH_PER_PLACEHOLDER: usize = 16;
const MAX_ESTIMATED_TEMPLATE_GROWTH: usize = 128;

type PlaceholderId = u16;
type CompiledTemplateCache =
  DashMap<Ustr, Arc<CompiledStringTemplate<'static>>, BuildHasherDefault<IdentityHasher>>;

static COMPILED_STRING_TEMPLATES: LazyLock<CompiledTemplateCache> = LazyLock::new(Default::default);

#[derive(Debug, Clone, Copy, PartialEq, Eq, StringEnum)]
#[repr(u8)]
pub enum PlaceholderKind {
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
  #[string_enum(rename = "uniqueName")]
  UniqueName,
  Local,
  Folder,
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

#[derive(Debug, Clone, Copy)]
pub struct StringTemplatePlaceholder {
  kind: PlaceholderKind,
  parameters: PlaceholderParameters,
}

impl StringTemplatePlaceholder {
  pub fn kind(self) -> PlaceholderKind {
    self.kind
  }

  pub fn hash_len(self) -> Option<usize> {
    match self.parameters {
      PlaceholderParameters::Hash { len, .. } => len.map(usize::from),
      PlaceholderParameters::None => None,
    }
  }

  pub fn hash_encoding(self) -> Option<HashDigest> {
    match self.parameters {
      PlaceholderParameters::Hash { encoding, .. } => encoding,
      PlaceholderParameters::None => None,
    }
  }
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
pub struct CompiledStringTemplate<'template> {
  template: Cow<'template, str>,
  placeholder_data: Vec<PlaceholderData>,
  segments: Vec<StringTemplateSegment>,
  has_hash_placeholder: bool,
  has_content_hash_placeholder: bool,
  hash_len: Option<u16>,
  full_hash_len: Option<u16>,
  chunk_hash_len: Option<u16>,
  content_hash_len: Option<u16>,
  template_without_hash_length: OnceLock<String>,
}

impl<'template> CompiledStringTemplate<'template> {
  fn compile(template: &'template str) -> Self {
    Self::compile_cow(Cow::Borrowed(template))
  }

  fn compile_cow(template: Cow<'template, str>) -> Self {
    debug_assert!(template.len() <= MAX_TEMPLATE_LEN);

    let mut placeholder_data = Vec::new();
    let mut segments = Vec::new();
    let mut plain_start = 0;
    let mut placeholder_start = None;
    let mut has_hash_placeholder = false;
    let mut has_content_hash_placeholder = false;
    let mut hash_len = None;
    let mut full_hash_len = None;
    let mut chunk_hash_len = None;
    let mut content_hash_len = None;
    let bytes = template.as_bytes();

    for index in memchr2_iter(b'[', b']', bytes) {
      if bytes[index] == b'[' {
        placeholder_start = Some(index);
        continue;
      }

      let Some(start) = placeholder_start.take() else {
        continue;
      };
      let end = index;
      let token = &template[start + 1..end];

      let Some((kind, parameters)) = parse_placeholder(token) else {
        continue;
      };

      has_hash_placeholder |= matches!(kind, PlaceholderKind::Hash | PlaceholderKind::FullHash);
      has_content_hash_placeholder |= kind == PlaceholderKind::ContentHash;
      if let PlaceholderParameters::Hash { len: Some(len), .. } = parameters {
        match kind {
          PlaceholderKind::Hash => hash_len.get_or_insert(len),
          PlaceholderKind::FullHash => full_hash_len.get_or_insert(len),
          PlaceholderKind::ChunkHash => chunk_hash_len.get_or_insert(len),
          PlaceholderKind::ContentHash => content_hash_len.get_or_insert(len),
          _ => unreachable!("only hash placeholders have hash parameters"),
        };
      }

      if plain_start < start {
        segments.push(StringTemplateSegment::Plain(to_u16_range(
          plain_start,
          start,
        )));
      }

      let id = PlaceholderId::try_from(placeholder_data.len())
        .expect("filename template contains too many placeholders");
      placeholder_data.push(PlaceholderData { kind, parameters });
      segments.push(StringTemplateSegment::Placeholder {
        id,
        range: to_u16_range(start, end + 1),
      });

      plain_start = end + 1;
    }

    if plain_start < template.len() {
      segments.push(StringTemplateSegment::Plain(to_u16_range(
        plain_start,
        template.len(),
      )));
    }

    Self {
      template,
      placeholder_data,
      segments,
      has_hash_placeholder,
      has_content_hash_placeholder,
      hash_len,
      full_hash_len,
      chunk_hash_len,
      content_hash_len,
      template_without_hash_length: OnceLock::new(),
    }
  }

  fn build_template_without_hash_length(&self) -> String {
    let mut output = String::with_capacity(self.template.len());

    for segment in &self.segments {
      match segment {
        StringTemplateSegment::Plain(range) => {
          output.push_str(&self.template[usize::from(range.start)..usize::from(range.end)]);
        }
        StringTemplateSegment::Placeholder { id, range } => {
          let data = self
            .placeholder_data
            .get(usize::from(*id))
            .expect("filename placeholder segment references missing side-table data");
          if let PlaceholderParameters::Hash { .. } = data.parameters {
            output.push('[');
            output.push_str(data.kind.as_str());
            output.push(']');
          } else {
            output.push_str(&self.template[usize::from(range.start)..usize::from(range.end)]);
          }
        }
      }
    }

    output
  }

  pub fn template(&self) -> &str {
    self.template.as_ref()
  }

  pub fn without_hash_length(&self) -> &str {
    if self.hash_len.is_none()
      && self.full_hash_len.is_none()
      && self.chunk_hash_len.is_none()
      && self.content_hash_len.is_none()
    {
      self.template.as_ref()
    } else {
      self
        .template_without_hash_length
        .get_or_init(|| self.build_template_without_hash_length())
    }
  }

  pub fn hash_len(&self) -> Option<usize> {
    self.hash_len.map(usize::from)
  }

  pub fn full_hash_len(&self) -> Option<usize> {
    self.full_hash_len.map(usize::from)
  }

  pub fn chunk_hash_len(&self) -> Option<usize> {
    self.chunk_hash_len.map(usize::from)
  }

  pub fn content_hash_len(&self) -> Option<usize> {
    self.content_hash_len.map(usize::from)
  }

  pub fn render_with(
    &self,
    mut renderer: impl FnMut(StringTemplatePlaceholder, &mut String) -> bool,
  ) -> String {
    let estimated_growth = self
      .placeholder_data
      .len()
      .saturating_mul(ESTIMATED_GROWTH_PER_PLACEHOLDER)
      .min(MAX_ESTIMATED_TEMPLATE_GROWTH);
    let mut output = String::with_capacity(self.template.len().saturating_add(estimated_growth));

    for segment in &self.segments {
      match segment {
        StringTemplateSegment::Plain(range) => {
          output.push_str(&self.template[usize::from(range.start)..usize::from(range.end)]);
        }
        StringTemplateSegment::Placeholder { id, range } => {
          let data = self
            .placeholder_data
            .get(usize::from(*id))
            .expect("filename placeholder segment references missing side-table data");
          let start = usize::from(range.start);
          let end = usize::from(range.end);
          let placeholder = StringTemplatePlaceholder {
            kind: data.kind,
            parameters: data.parameters,
          };
          if !renderer(placeholder, &mut output) {
            output.push_str(&self.template[start..end]);
          }
        }
      }
    }

    output
  }

  pub fn render_with_path_data(
    &self,
    options: PathData<'_>,
    mut asset_info: Option<&mut AssetInfo>,
    mut renderer: impl FnMut(StringTemplatePlaceholder, &mut String) -> bool,
  ) -> String {
    if let Some(content_hash) = options.content_hash
      && let Some(asset_info) = asset_info.as_deref_mut()
    {
      // Set version even when the template has no content hash placeholder.
      asset_info.version = content_hash.to_string();
    }

    let file_replacements = FileReplacements::new(options);
    self.render_with(|placeholder, output| {
      renderer(placeholder, output)
        || render_placeholder(
          placeholder,
          options,
          &file_replacements,
          &mut asset_info,
          output,
        )
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

fn get_or_compile(template: Ustr) -> Arc<CompiledStringTemplate<'static>> {
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

  pub fn compiled_template(&self) -> Option<Arc<CompiledStringTemplate<'static>>> {
    match self.0 {
      FilenameKind::Template(template) => Some(get_or_compile(template)),
      FilenameKind::Fn(_) => None,
    }
  }

  pub async fn compiled(
    &self,
    options: PathData<'_>,
    asset_info: Option<&AssetInfo>,
  ) -> rspack_error::Result<Arc<CompiledStringTemplate<'static>>> {
    match &self.0 {
      FilenameKind::Template(template) => Ok(get_or_compile(*template)),
      FilenameKind::Fn(filename_fn) => {
        let template = filename_fn.call(&options, asset_info).await?;
        assert_template_len(&template);
        Ok(Arc::new(CompiledStringTemplate::compile_cow(Cow::Owned(
          template,
        ))))
      }
    }
  }

  pub fn has_hash_placeholder(&self) -> bool {
    match self.0 {
      FilenameKind::Template(template) => get_or_compile(template).has_hash_placeholder,
      FilenameKind::Fn(_) => true,
    }
  }

  pub fn has_content_hash_placeholder(&self) -> bool {
    match self.0 {
      FilenameKind::Template(template) => get_or_compile(template).has_content_hash_placeholder,
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
    self.render_with(options, asset_info, |_, _| false).await
  }

  pub async fn render_with(
    &self,
    options: PathData<'_>,
    asset_info: Option<&mut AssetInfo>,
    renderer: impl FnMut(StringTemplatePlaceholder, &mut String) -> bool,
  ) -> rspack_error::Result<String> {
    let compiled = self.compiled(options, asset_info.as_deref()).await?;
    Ok(compiled.render_with_path_data(options, asset_info, renderer))
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

pub fn has_content_hash_placeholder(template: &str) -> bool {
  let compiled = get_or_compile(intern_template(template));
  compiled.has_content_hash_placeholder
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

fn render_placeholder(
  placeholder: StringTemplatePlaceholder,
  options: PathData<'_>,
  file_replacements: &FileReplacements,
  asset_info: &mut Option<&mut AssetInfo>,
  output: &mut String,
) -> bool {
  let kind = placeholder.kind;
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
    PlaceholderKind::UniqueName | PlaceholderKind::Local | PlaceholderKind::Folder => false,
    PlaceholderKind::Hash => try_render_hash(
      options.hash,
      kind,
      placeholder.parameters,
      asset_info,
      output,
    ),
    PlaceholderKind::FullHash => try_render_hash(
      options.hash,
      kind,
      placeholder.parameters,
      asset_info,
      output,
    ),
    PlaceholderKind::ContentHash => try_render_hash(
      options.content_hash,
      kind,
      placeholder.parameters,
      asset_info,
      output,
    ),
    PlaceholderKind::ChunkHash => try_render_hash(
      options.chunk_hash,
      kind,
      placeholder.parameters,
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
