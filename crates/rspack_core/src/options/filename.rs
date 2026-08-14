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
use rspack_hash::RspackHasher;
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
type PlaceholderKindId = u8;
type PlaceholderMap = HashMap<Ustr, PlaceholderKindId, BuildHasherDefault<IdentityHasher>>;
type CompiledTemplateCache =
  DashMap<Ustr, Arc<CompiledStringTemplate>, BuildHasherDefault<IdentityHasher>>;

static COMPILED_STRING_TEMPLATES: LazyLock<CompiledTemplateCache> = LazyLock::new(Default::default);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
  FullHash,
  ChunkHash,
  ContentHash,
}

impl PlaceholderKind {
  fn name(self) -> &'static str {
    match self {
      Self::File => "file",
      Self::Base => "base",
      Self::Name => "name",
      Self::Path => "path",
      Self::Ext => "ext",
      Self::Query => "query",
      Self::Fragment => "fragment",
      Self::Id => "id",
      Self::Runtime => "runtime",
      Self::Url => "url",
      Self::Hash => "hash",
      Self::FullHash => "fullhash",
      Self::ChunkHash => "chunkhash",
      Self::ContentHash => "contenthash",
    }
  }

  fn from_id(id: PlaceholderKindId) -> Self {
    match id {
      id if id == Self::File as PlaceholderKindId => Self::File,
      id if id == Self::Base as PlaceholderKindId => Self::Base,
      id if id == Self::Name as PlaceholderKindId => Self::Name,
      id if id == Self::Path as PlaceholderKindId => Self::Path,
      id if id == Self::Ext as PlaceholderKindId => Self::Ext,
      id if id == Self::Query as PlaceholderKindId => Self::Query,
      id if id == Self::Fragment as PlaceholderKindId => Self::Fragment,
      id if id == Self::Id as PlaceholderKindId => Self::Id,
      id if id == Self::Runtime as PlaceholderKindId => Self::Runtime,
      id if id == Self::Url as PlaceholderKindId => Self::Url,
      id if id == Self::Hash as PlaceholderKindId => Self::Hash,
      id if id == Self::FullHash as PlaceholderKindId => Self::FullHash,
      id if id == Self::ChunkHash as PlaceholderKindId => Self::ChunkHash,
      id if id == Self::ContentHash as PlaceholderKindId => Self::ContentHash,
      _ => unreachable!("invalid filename placeholder kind id: {id}"),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HashEncoding {
  Raw,
  Base64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlaceholderParameters {
  None,
  Hash {
    len: Option<u16>,
    encoding: HashEncoding,
  },
}

#[derive(Debug, Clone)]
struct PlaceholderData {
  kind: PlaceholderKindId,
  parameters: PlaceholderParameters,
  raw: Range<u16>,
}

#[derive(Debug, Clone)]
enum StringTemplateSegment {
  Plain(Range<u16>),
  Placeholder(PlaceholderId),
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

      let id = PlaceholderId::try_from(placeholder_data.len())
        .expect("filename template contains too many placeholders");
      let raw = to_u16_range(start, end + 1);
      placeholder_data.push(PlaceholderData {
        kind: kind as PlaceholderKindId,
        parameters,
        raw,
      });
      segments.push(StringTemplateSegment::Placeholder(id));
      placeholder_indices
        .entry(Ustr::from(kind.name()))
        .or_insert(kind as PlaceholderKindId);

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
}

fn to_u16_range(start: usize, end: usize) -> Range<u16> {
  u16::try_from(start).expect("filename template range start exceeds u16::MAX")
    ..u16::try_from(end).expect("filename template range end exceeds u16::MAX")
}

fn parse_placeholder(token: &str) -> Option<(PlaceholderKind, PlaceholderParameters)> {
  let kind = match token {
    "file" => PlaceholderKind::File,
    "base" => PlaceholderKind::Base,
    "name" => PlaceholderKind::Name,
    "path" => PlaceholderKind::Path,
    "ext" => PlaceholderKind::Ext,
    "query" => PlaceholderKind::Query,
    "fragment" => PlaceholderKind::Fragment,
    "id" => PlaceholderKind::Id,
    "runtime" => PlaceholderKind::Runtime,
    "url" => PlaceholderKind::Url,
    "hash" => PlaceholderKind::Hash,
    "fullhash" => PlaceholderKind::FullHash,
    "chunkhash" => PlaceholderKind::ChunkHash,
    "contenthash" => PlaceholderKind::ContentHash,
    _ => return parse_hash_placeholder(token),
  };

  let parameters = if matches!(
    kind,
    PlaceholderKind::Hash
      | PlaceholderKind::FullHash
      | PlaceholderKind::ChunkHash
      | PlaceholderKind::ContentHash
  ) {
    PlaceholderParameters::Hash {
      len: None,
      encoding: HashEncoding::Raw,
    }
  } else {
    PlaceholderParameters::None
  };
  Some((kind, parameters))
}

fn parse_hash_placeholder(token: &str) -> Option<(PlaceholderKind, PlaceholderParameters)> {
  let (kind, parameters) = [
    (PlaceholderKind::Hash, "hash:"),
    (PlaceholderKind::FullHash, "fullhash:"),
    (PlaceholderKind::ChunkHash, "chunkhash:"),
    (PlaceholderKind::ContentHash, "contenthash:"),
  ]
  .into_iter()
  .find_map(|(kind, prefix)| token.strip_prefix(prefix).map(|value| (kind, value)))?;

  let mut configs = parameters.split(':');
  let first = configs.next()?;
  let (len, encoding) = if first == "base64" {
    (configs.next()?.parse::<usize>().ok()?, HashEncoding::Base64)
  } else {
    (first.parse::<usize>().ok()?, HashEncoding::Raw)
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
  let template = Ustr::from(template);
  get_or_compile(template);
  template
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
          || compiled
            .placeholder_indices
            .contains_key(&Ustr::from("hash"))
          || compiled
            .placeholder_indices
            .contains_key(&Ustr::from("fullhash"))
      }
      FilenameKind::Fn(_) => true,
    }
  }

  pub fn has_content_hash_placeholder(&self) -> bool {
    match self.0 {
      FilenameKind::Template(template) => {
        let compiled = get_or_compile(template);
        compiled.has_content_hash_placeholder
          || compiled
            .placeholder_indices
            .contains_key(&Ustr::from("contenthash"))
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
    let template = match &self.0 {
      FilenameKind::Template(template) => *template,
      FilenameKind::Fn(filename_fn) => {
        let template = filename_fn.call(&options, asset_info.as_deref()).await?;
        intern_template(&template)
      }
    };
    let compiled = get_or_compile(template);
    Ok(render_template(
      template.as_str(),
      &compiled,
      options,
      asset_info,
    ))
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

#[derive(Debug)]
struct StagedReplacement {
  value: String,
  stage: i8,
}

#[derive(Debug, Default)]
struct FileReplacements {
  file: Option<StagedReplacement>,
  base: Option<StagedReplacement>,
  name: Option<StagedReplacement>,
  path: Option<StagedReplacement>,
  ext: Option<StagedReplacement>,
  query: Option<StagedReplacement>,
  fragment: Option<StagedReplacement>,
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
        file: Some(staged("", 0)),
        query: Some(staged("", 1)),
        fragment: Some(staged("", 2)),
        path: Some(staged("", 3)),
        base: Some(staged(replacer.clone(), 4)),
        name: Some(staged(replacer, 5)),
        ext: Some(staged(ext, 6)),
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
      file: Some(staged(file.as_str(), 0)),
      ext: Some(staged(ext, 1)),
      base: base.map(|base| staged(base, 2)),
      name: name.map(|name| staged(name, 3)),
      path: Some(staged(path, 4)),
      query: Some(staged(query.unwrap_or_default(), 5)),
      fragment: Some(staged(fragment.unwrap_or_default(), 6)),
    }
  }
}

fn staged(value: impl Into<String>, stage: i8) -> StagedReplacement {
  StagedReplacement {
    value: value.into(),
    stage,
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
    -1,
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
  min_stage: i8,
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
      StringTemplateSegment::Placeholder(id) => {
        let data = compiled
          .placeholder_data
          .get(usize::from(*id))
          .expect("filename placeholder segment references missing side-table data");
        if !render_placeholder(
          data,
          min_stage,
          options,
          file_replacements,
          asset_info,
          output,
        ) {
          output.push_str(&template[usize::from(data.raw.start)..usize::from(data.raw.end)]);
        }
      }
    }
  }
}

fn render_placeholder(
  data: &PlaceholderData,
  min_stage: i8,
  options: PathData<'_>,
  file_replacements: &FileReplacements,
  asset_info: &mut Option<&mut AssetInfo>,
  output: &mut String,
) -> bool {
  let kind = PlaceholderKind::from_id(data.kind);
  match kind {
    PlaceholderKind::File => try_render_staged(
      file_replacements.file.as_ref(),
      min_stage,
      options,
      file_replacements,
      asset_info,
      output,
    ),
    PlaceholderKind::Base => try_render_staged(
      file_replacements.base.as_ref(),
      min_stage,
      options,
      file_replacements,
      asset_info,
      output,
    ),
    PlaceholderKind::Name => {
      try_render_staged(
        file_replacements.name.as_ref(),
        min_stage,
        options,
        file_replacements,
        asset_info,
        output,
      ) || try_render_value(
        options.chunk_name.or(options.chunk_id),
        11,
        min_stage,
        options,
        file_replacements,
        asset_info,
        output,
      )
    }
    PlaceholderKind::Path => try_render_staged(
      file_replacements.path.as_ref(),
      min_stage,
      options,
      file_replacements,
      asset_info,
      output,
    ),
    PlaceholderKind::Ext => try_render_staged(
      file_replacements.ext.as_ref(),
      min_stage,
      options,
      file_replacements,
      asset_info,
      output,
    ),
    PlaceholderKind::Query => try_render_staged(
      file_replacements.query.as_ref(),
      min_stage,
      options,
      file_replacements,
      asset_info,
      output,
    ),
    PlaceholderKind::Fragment => try_render_staged(
      file_replacements.fragment.as_ref(),
      min_stage,
      options,
      file_replacements,
      asset_info,
      output,
    ),
    PlaceholderKind::Id => try_render_value(
      options.id.or(options.chunk_id).or(options.module_id),
      9,
      min_stage,
      options,
      file_replacements,
      asset_info,
      output,
    ),
    PlaceholderKind::Runtime => try_render_value(
      Some(options.runtime.unwrap_or("_")),
      13,
      min_stage,
      options,
      file_replacements,
      asset_info,
      output,
    ),
    PlaceholderKind::Url => try_render_value(
      options.url,
      14,
      min_stage,
      options,
      file_replacements,
      asset_info,
      output,
    ),
    PlaceholderKind::Hash => try_render_hash(
      options.hash,
      7,
      kind,
      data.parameters,
      min_stage,
      options,
      file_replacements,
      asset_info,
      output,
    ),
    PlaceholderKind::FullHash => try_render_hash(
      options.hash,
      8,
      kind,
      data.parameters,
      min_stage,
      options,
      file_replacements,
      asset_info,
      output,
    ),
    PlaceholderKind::ContentHash => try_render_hash(
      options.content_hash,
      10,
      kind,
      data.parameters,
      min_stage,
      options,
      file_replacements,
      asset_info,
      output,
    ),
    PlaceholderKind::ChunkHash => try_render_hash(
      options.chunk_hash,
      12,
      kind,
      data.parameters,
      min_stage,
      options,
      file_replacements,
      asset_info,
      output,
    ),
  }
}

fn try_render_staged(
  replacement: Option<&StagedReplacement>,
  min_stage: i8,
  options: PathData<'_>,
  file_replacements: &FileReplacements,
  asset_info: &mut Option<&mut AssetInfo>,
  output: &mut String,
) -> bool {
  let Some(replacement) = replacement.filter(|replacement| replacement.stage > min_stage) else {
    return false;
  };
  push_replacement(
    &replacement.value,
    replacement.stage,
    options,
    file_replacements,
    asset_info,
    output,
  );
  true
}

#[allow(clippy::too_many_arguments)]
fn try_render_value(
  replacement: Option<&str>,
  stage: i8,
  min_stage: i8,
  options: PathData<'_>,
  file_replacements: &FileReplacements,
  asset_info: &mut Option<&mut AssetInfo>,
  output: &mut String,
) -> bool {
  let Some(replacement) = replacement.filter(|_| stage > min_stage) else {
    return false;
  };
  push_replacement(
    replacement,
    stage,
    options,
    file_replacements,
    asset_info,
    output,
  );
  true
}

#[allow(clippy::too_many_arguments)]
fn try_render_hash(
  hash: Option<&str>,
  stage: i8,
  kind: PlaceholderKind,
  parameters: PlaceholderParameters,
  min_stage: i8,
  options: PathData<'_>,
  file_replacements: &FileReplacements,
  asset_info: &mut Option<&mut AssetInfo>,
  output: &mut String,
) -> bool {
  let Some(hash) = hash.filter(|_| stage > min_stage) else {
    return false;
  };
  let PlaceholderParameters::Hash { len, encoding } = parameters else {
    unreachable!("hash placeholder must have hash parameters");
  };

  let content: Cow<'_, str> = match encoding {
    HashEncoding::Raw => hash.into(),
    HashEncoding::Base64 => base64::encode_to_string(hash).into(),
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

  push_replacement(
    content,
    stage,
    options,
    file_replacements,
    asset_info,
    output,
  );
  true
}

fn push_replacement(
  replacement: &str,
  stage: i8,
  options: PathData<'_>,
  file_replacements: &FileReplacements,
  asset_info: &mut Option<&mut AssetInfo>,
  output: &mut String,
) {
  if replacement.contains('[') && replacement.len() <= MAX_TEMPLATE_LEN {
    let compiled = CompiledStringTemplate::compile(replacement);
    render_compiled_template(
      replacement,
      &compiled,
      stage,
      options,
      file_replacements,
      asset_info,
      output,
    );
  } else {
    output.push_str(replacement);
  }
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
