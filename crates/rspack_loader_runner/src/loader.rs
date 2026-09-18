use std::{
  fmt::Display,
  ops::Deref,
  sync::{
    Arc, LazyLock,
    atomic::{AtomicBool, Ordering},
  },
};

use async_trait::async_trait;
use derive_more::Debug;
use rspack_cacheable::cacheable_dyn;
use rspack_collections::{Identifier, IdentifierDashMap};
use rspack_error::Result;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_util::identifier::strip_zero_width_space_for_fragment;

use super::{LoaderContext, LoaderRunnerOptions};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoaderExecutionKind {
  Native,
  JavaScript,
}

#[derive(Debug)]
pub struct LoaderItem<Context: Send> {
  #[debug("{}", loader.identifier())]
  loader: Arc<dyn Loader<Context>>,
  /// Loader identifier
  request: Identifier,
  /// The loader request split into its path, query and fragment. Requests are shared by every
  /// module that uses the same loader, so the parsed parts are shared through an [`Arc`] as well.
  ///
  /// An absolute path or a virtual path for represent the loader.
  /// The absolute path is used to represent a loader stayed on the JS side.
  /// `$` split chain may be used to represent a composed loader chain from the JS side.
  /// Virtual path with a builtin protocol to represent a loader from the native side. e.g "builtin:".
  parsed_request: Arc<ResourceParsedData>,
  /// Data shared between pitching and normal
  data: serde_json::Value,
  r#type: String,
  cache_options: Option<Box<LoaderRunnerOptions>>,
  execution_kind: LoaderExecutionKind,
  pitch_executed: AtomicBool,
  normal_executed: AtomicBool,
  /// Whether loader was called with [LoaderContext::finish_with].
  ///
  /// Indicates that the loader has finished its work,
  /// otherwise loader runner will reset [`LoaderContext::content`], [`LoaderContext::source_map`], [`LoaderContext::additional_data`].
  ///
  /// This flag is used to align with webpack's behavior:
  /// If nothing is modified in the loader, the loader will reset the content, source map, and additional data.
  finish_called: AtomicBool,
}

impl<C: Send> LoaderItem<C> {
  #[inline]
  pub fn execution_kind(&self) -> LoaderExecutionKind {
    self.execution_kind
  }

  pub fn loader(&self) -> &Arc<dyn Loader<C>> {
    &self.loader
  }

  #[inline]
  pub fn request(&self) -> Identifier {
    self.request
  }

  #[inline]
  pub fn path(&self) -> &Utf8Path {
    &self.parsed_request.path
  }

  #[inline]
  pub fn query(&self) -> Option<&str> {
    self.parsed_request.query.as_deref()
  }

  #[inline]
  pub fn r#type(&self) -> &str {
    &self.r#type
  }

  #[inline]
  pub fn cache(&self) -> bool {
    self.cache_options.is_some()
  }

  #[inline]
  pub fn loader_name(&self) -> &str {
    self
      .cache_options
      .as_deref()
      .map_or("", |options| &options.loader_name)
  }

  #[inline]
  pub fn options_cache_key(&self) -> &str {
    self
      .cache_options
      .as_deref()
      .map_or("", |options| &options.options_cache_key)
  }

  #[inline]
  pub fn loader_version(&self) -> &str {
    self
      .cache_options
      .as_deref()
      .map_or("", |options| &options.loader_version)
  }

  #[inline]
  pub fn cache_options(&self) -> Option<&LoaderRunnerOptions> {
    self.cache_options.as_deref()
  }

  #[inline]
  pub fn data(&self) -> &serde_json::Value {
    &self.data
  }

  #[inline]
  #[doc(hidden)]
  pub fn set_data(&mut self, data: serde_json::Value) {
    self.data = data;
  }

  #[inline]
  #[doc(hidden)]
  pub fn pitch_executed(&self) -> bool {
    self.pitch_executed.load(Ordering::Relaxed)
  }

  #[inline]
  pub fn normal_executed(&self) -> bool {
    self.normal_executed.load(Ordering::Relaxed)
  }

  #[inline]
  #[doc(hidden)]
  pub fn finish_called(&self) -> bool {
    self.finish_called.load(Ordering::Relaxed)
  }

  #[inline]
  #[doc(hidden)]
  pub fn set_pitch_executed(&self) {
    self.pitch_executed.store(true, Ordering::Relaxed)
  }

  #[inline]
  #[doc(hidden)]
  pub fn set_normal_executed(&self) {
    self.normal_executed.store(true, Ordering::Relaxed)
  }

  #[inline]
  #[doc(hidden)]
  pub fn set_finish_called(&self) {
    self.finish_called.store(true, Ordering::Relaxed)
  }
}

impl<C: Send> Display for LoaderItem<C> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.loader.identifier())
  }
}

#[derive(Debug)]
pub struct LoaderItemList<'a, Context: Send>(pub &'a [LoaderItem<Context>]);

impl<Context: Send> Deref for LoaderItemList<'_, Context> {
  type Target = [LoaderItem<Context>];

  fn deref(&self) -> &Self::Target {
    self.0
  }
}

impl<Context: Send> Default for LoaderItemList<'_, Context> {
  fn default() -> Self {
    Self(&[])
  }
}

pub trait DisplayWithSuffix: Display {
  fn display_with_suffix(&self, suffix: &str) -> String {
    let s = self.to_string();
    if s.is_empty() {
      return suffix.to_string();
    }
    self.to_string() + "!" + suffix
  }
}

impl<Context: Send> DisplayWithSuffix for LoaderItemList<'_, Context> {}
impl<Context: Send> DisplayWithSuffix for LoaderItem<Context> {}
impl<Context: Send> Display for LoaderItemList<'_, Context> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = self
      .0
      .iter()
      .map(|item| item.to_string())
      .collect::<Vec<_>>()
      .join("!");

    write!(f, "{s}")
  }
}

#[cacheable_dyn]
#[async_trait]
pub trait Loader<Context = ()>: Send + Sync
where
  Context: Send,
{
  /// Returns the unique identifier for this loader
  fn identifier(&self) -> Identifier;

  async fn run(&self, loader_context: &mut LoaderContext<Context>) -> Result<()> {
    // If loader does not implement normal stage,
    // it should inherit the result from the previous loader.
    loader_context.current_loader().set_finish_called();
    Ok(())
  }

  async fn pitch(&self, _loader_context: &mut LoaderContext<Context>) -> Result<()> {
    // noop
    Ok(())
  }

  /// Returns the loader type based on the module's package.json type field or file extension.
  /// This affects how the loader context interprets the module (e.g., "commonjs", "module").
  fn r#type(&self) -> Option<&str> {
    None
  }

  /// Version identity used by loader caching.
  fn cache_version(&self) -> Option<&str> {
    None
  }

  /// Selects the runtime responsible for executing this loader.
  fn execution_kind(&self) -> LoaderExecutionKind {
    LoaderExecutionKind::Native
  }
}

impl<C: Send> From<Arc<dyn Loader<C>>> for LoaderItem<C> {
  fn from(loader: Arc<dyn Loader<C>>) -> Self {
    Self::new(loader, LoaderRunnerOptions::default())
  }
}

impl<C: Send> LoaderItem<C> {
  pub(crate) fn new(loader: Arc<dyn Loader<C>>, options: LoaderRunnerOptions) -> Self {
    let cache_options = options.cache.then(|| Box::new(options));
    let execution_kind = loader.execution_kind();
    let request = loader.identifier();
    let parsed_request = cached_loader_request(request);
    let r#type = loader
      .r#type()
      .map_or_else(String::default, ToOwned::to_owned);
    Self {
      loader,
      request,
      parsed_request,
      data: serde_json::Value::Null,
      r#type,
      cache_options,
      execution_kind,
      pitch_executed: AtomicBool::new(false),
      normal_executed: AtomicBool::new(false),
      finish_called: AtomicBool::new(false),
    }
  }
}

#[derive(Debug)]
pub struct ResourceParsedData {
  pub path: Utf8PathBuf,
  pub query: Option<String>,
  pub fragment: Option<String>,
}

/// Splits a resource into its path, query and fragment.
///
/// The query and the fragment keep their leading `?` and `#`. A zero-width space escapes the
/// next character, so an escaped `?` or `#` stays part of the current segment instead of
/// starting a new one.
pub fn parse_resource(resource: &str) -> Option<ResourceParsedData> {
  let (path, query, fragment) = path_query_fragment(resource);

  Some(ResourceParsedData {
    path: strip_zero_width_space_for_fragment(path)
      .into_owned()
      .into(),
    query: query.map(|q| strip_zero_width_space_for_fragment(q).into_owned()),
    fragment: fragment.map(|f| f.to_owned()),
  })
}

/// Loader requests are shared by every module that uses the same loader, so the parsed parts are
/// cached instead of being parsed for each module.
static LOADER_REQUEST_CACHE: LazyLock<IdentifierDashMap<Arc<ResourceParsedData>>> =
  LazyLock::new(Default::default);

/// Upper bound for [`LOADER_REQUEST_CACHE`]. Loader requests are a small, stable set in practice;
/// the bound only keeps configs that generate unique requests per module from growing the cache
/// without limit. Requests beyond the bound are parsed on demand.
const MAX_CACHED_LOADER_REQUESTS: usize = 4096;

fn cached_loader_request(request: Identifier) -> Arc<ResourceParsedData> {
  if let Some(parsed) = LOADER_REQUEST_CACHE.get(&request) {
    return parsed.clone();
  }

  let parsed = Arc::new(parse_resource(request.as_str()).expect("identifier should be valid"));
  if LOADER_REQUEST_CACHE.len() < MAX_CACHED_LOADER_REQUESTS {
    LOADER_REQUEST_CACHE.insert(request, parsed.clone());
  }
  parsed
}

#[cfg(not(windows))]
fn path_query_fragment(input: &str) -> (&str, Option<&str>, Option<&str>) {
  let (path_len, query, fragment) = split_path_query_fragment(input);
  (&input[..path_len], query, fragment)
}

#[cfg(windows)]
fn path_query_fragment(input: &str) -> (&str, Option<&str>, Option<&str>) {
  let prefix_len = rspack_paths::dos_device_path_prefix_len(input);
  let (path_len, query, fragment) = split_path_query_fragment(&input[prefix_len..]);
  (&input[..prefix_len + path_len], query, fragment)
}

/// Splits `input` into the path length, the query and the fragment.
fn split_path_query_fragment(input: &str) -> (usize, Option<&str>, Option<&str>) {
  let bytes = input.as_bytes();
  let path_len = scan_segment(bytes, 0, false);
  let mut index = path_len;

  // Both the query and the fragment keep their leading `?`/`#`.
  let query = if bytes.get(index) == Some(&b'?') {
    let query_start = index;
    index = scan_segment(bytes, index + 1, true);
    Some(&input[query_start..index])
  } else {
    None
  };

  let fragment = if bytes.get(index) == Some(&b'#') {
    Some(&input[index..])
  } else {
    None
  };

  (path_len, query, fragment)
}

/// Returns the end offset of a path or query segment.
///
/// `#` ends both segments while `?` only ends a path. The scan skips whole characters, so
/// multi-byte content survives the split.
#[inline]
fn scan_segment(bytes: &[u8], start: usize, in_query: bool) -> usize {
  // A zero-width space escapes the following character.
  const ZERO_WIDTH_SPACE: &[u8] = "\u{200b}".as_bytes();
  const ZERO_WIDTH_SPACE_FIRST_BYTE: u8 = ZERO_WIDTH_SPACE[0];

  let mut index = start;
  loop {
    let rest = &bytes[index..];
    let offset = if in_query {
      memchr::memchr2(b'#', ZERO_WIDTH_SPACE_FIRST_BYTE, rest)
    } else {
      memchr::memchr3(b'?', b'#', ZERO_WIDTH_SPACE_FIRST_BYTE, rest)
    };
    let Some(offset) = offset else {
      return bytes.len();
    };
    index += offset;

    match bytes[index] {
      b'#' => return index,
      b'?' if !in_query => return index,
      _ => {}
    }

    if bytes[index..].starts_with(ZERO_WIDTH_SPACE) {
      let escaped = index + ZERO_WIDTH_SPACE.len();
      let Some(&escaped_first_byte) = bytes.get(escaped) else {
        // A trailing zero-width space escapes nothing, so it is not part of the segment.
        return index;
      };
      index = escaped + char_width(escaped_first_byte);
    } else {
      // A leading byte of some other multi-byte character. The remaining bytes of that
      // character are continuation bytes, which the next scan ignores.
      index += 1;
    }
  }
}

/// Returns the byte width of a character from its leading byte.
#[inline]
const fn char_width(leading_byte: u8) -> usize {
  match leading_byte {
    0xc0..=0xdf => 2,
    0xe0..=0xef => 3,
    0xf0..=0xf7 => 4,
    // ASCII leading bytes and continuation bytes, which never start a character.
    _ => 1,
  }
}

#[cfg(test)]
pub(crate) mod test {
  use std::sync::Arc;

  use rspack_cacheable::{cacheable, cacheable_dyn};
  use rspack_collections::Identifier;
  use rspack_paths::Utf8Path;

  use super::{Loader, LoaderItem};

  #[cacheable]
  #[allow(dead_code)]
  pub(crate) struct Custom;
  #[cacheable_dyn]
  #[async_trait::async_trait]
  impl Loader<()> for Custom {
    fn identifier(&self) -> Identifier {
      "/rspack/custom-loader-1/index.js?foo=1#baz".into()
    }
  }

  #[cacheable]
  #[allow(dead_code)]
  pub(crate) struct Custom2;
  #[cacheable_dyn]
  #[async_trait::async_trait]
  impl Loader<()> for Custom2 {
    fn identifier(&self) -> Identifier {
      "/rspack/custom-loader-2/index.js?bar=2#baz".into()
    }
  }

  #[cacheable]
  #[allow(dead_code)]
  pub(crate) struct Builtin;
  #[cacheable_dyn]
  #[async_trait::async_trait]
  impl Loader<()> for Builtin {
    fn identifier(&self) -> Identifier {
      "builtin:test-loader".into()
    }
  }

  #[cacheable]
  pub(crate) struct PosixNonLenBlankUnicode;

  #[cacheable_dyn]
  #[async_trait::async_trait]
  impl Loader<()> for PosixNonLenBlankUnicode {
    fn identifier(&self) -> Identifier {
      "/a/b/c.js?{\"c\": \"\u{200b}#foo\"}".into()
    }
  }

  #[cacheable]
  pub(crate) struct WinNonLenBlankUnicode;
  #[cacheable_dyn]
  #[async_trait::async_trait]
  impl Loader<()> for WinNonLenBlankUnicode {
    fn identifier(&self) -> Identifier {
      "\\a\\b\\c.js?{\"c\": \"\u{200b}#foo\"}".into()
    }
  }

  #[test]
  fn should_handle_posix_non_len_blank_unicode_correctly() {
    let c1 = Arc::new(PosixNonLenBlankUnicode) as Arc<dyn Loader<()>>;
    let l: LoaderItem<()> = c1.into();
    assert_eq!(l.path(), Utf8Path::new("/a/b/c.js"));
    assert_eq!(l.query(), Some("?{\"c\": \"#foo\"}"));
    assert_eq!(l.parsed_request.fragment, None);
  }

  #[test]
  fn should_handle_win_non_len_blank_unicode_correctly() {
    let c1 = Arc::new(WinNonLenBlankUnicode) as Arc<dyn Loader<()>>;
    let l: LoaderItem<()> = c1.into();
    assert_eq!(l.path(), Utf8Path::new(r#"\a\b\c.js"#));
    assert_eq!(l.query(), Some("?{\"c\": \"#foo\"}"));
    assert_eq!(l.parsed_request.fragment, None);
  }
}
