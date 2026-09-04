use std::{fmt::Display, ops::Deref, sync::Arc};

use async_trait::async_trait;
use derive_more::Debug;
use rspack_cacheable::cacheable_dyn;
use rspack_collections::Identifier;
use rspack_error::Result;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_util::identifier::strip_zero_width_space_for_fragment;

use super::{LoaderContext, LoaderExecutionKind, LoaderRunnerOptions};

#[derive(Debug)]
pub struct LoaderItem<Context: Send> {
  #[debug("{}", loader.identifier())]
  loader: Arc<dyn Loader<Context>>,
  /// Loader identifier
  request: Identifier,
  /// An absolute path or a virtual path for represent the loader.
  /// The absolute path is used to represent a loader stayed on the JS side.
  /// `$` split chain may be used to represent a composed loader chain from the JS side.
  /// Virtual path with a builtin protocol to represent a loader from the native side. e.g "builtin:".
  #[allow(dead_code)]
  path: Utf8PathBuf,
  /// Query of a loader, starts with `?`
  #[allow(dead_code)]
  query: Option<String>,
  /// Fragment of a loader, starts with `#`.
  #[allow(dead_code)]
  fragment: Option<String>,
  r#type: String,
  cache_options: Option<Box<LoaderRunnerOptions>>,
  execution_kind: LoaderExecutionKind,
}

#[derive(Debug, Default)]
pub struct LoaderItemState {
  /// Data shared between pitching and normal.
  data: serde_json::Value,
  pitch_executed: bool,
  normal_executed: bool,
  /// Whether loader was called with [LoaderContext::finish_with].
  finish_called: bool,
}

impl<C: Send> LoaderItem<C> {
  pub fn loader(&self) -> &Arc<dyn Loader<C>> {
    &self.loader
  }

  #[inline]
  pub fn execution_kind(&self) -> LoaderExecutionKind {
    self.execution_kind
  }

  #[inline]
  pub fn request(&self) -> Identifier {
    self.request
  }

  #[inline]
  pub fn path(&self) -> &Utf8Path {
    &self.path
  }

  #[inline]
  pub fn query(&self) -> Option<&str> {
    self.query.as_deref()
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
}

impl LoaderItemState {
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
    self.pitch_executed
  }

  #[inline]
  pub fn normal_executed(&self) -> bool {
    self.normal_executed
  }

  #[inline]
  #[doc(hidden)]
  pub fn finish_called(&self) -> bool {
    self.finish_called
  }

  #[inline]
  #[doc(hidden)]
  pub fn set_pitch_executed(&mut self) {
    self.pitch_executed = true;
  }

  #[inline]
  #[doc(hidden)]
  pub fn set_normal_executed(&mut self) {
    self.normal_executed = true;
  }

  #[inline]
  #[doc(hidden)]
  pub fn set_finish_called(&mut self) {
    self.finish_called = true;
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
    loader_context.set_current_loader_finish_called();
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
    let ident = &**loader.identifier();
    let execution_kind = loader.execution_kind();
    if let Some(r#type) = loader.r#type() {
      let ResourceParsedData {
        path,
        query,
        fragment,
      } = parse_resource(ident).expect("identifier should be valid");
      let ty = r#type.to_string();
      return Self {
        loader,
        request: ident.into(),
        path,
        query,
        fragment,
        r#type: ty,
        cache_options,
        execution_kind,
      };
    }
    let ident = loader.identifier();
    let ResourceParsedData {
      path,
      query,
      fragment,
    } = parse_resource(&ident).expect("identifier should be valid");
    Self {
      loader,
      request: ident,
      path,
      query,
      fragment,
      r#type: String::default(),
      cache_options,
      execution_kind,
    }
  }
}

#[derive(Debug)]
pub struct ResourceParsedData {
  pub path: Utf8PathBuf,
  pub query: Option<String>,
  pub fragment: Option<String>,
}

pub fn parse_resource(resource: &str) -> Option<ResourceParsedData> {
  let (path, query, fragment) = path_query_fragment(resource).ok()?;

  Some(ResourceParsedData {
    path: strip_zero_width_space_for_fragment(path)
      .into_owned()
      .into(),
    query: query.map(|q| strip_zero_width_space_for_fragment(q).into_owned()),
    fragment: fragment.map(|f| f.to_owned()),
  })
}

fn path_query_fragment(mut input: &str) -> winnow::ModalResult<(&str, Option<&str>, Option<&str>)> {
  use winnow::{
    combinator::{alt, opt, repeat},
    prelude::*,
    token::{any, none_of, rest},
  };

  let path = alt((
    ('\u{200b}', any).take(),
    none_of(('?', '#', '\u{200b}')).take(),
  ));
  let query = alt((('\u{200b}', any).take(), none_of(('#', '\u{200b}')).take()));
  let fragment = rest;

  let mut parser = (
    repeat::<_, _, (), _, _>(.., path).take(),
    opt(('?', repeat::<_, _, (), _, _>(.., query)).take()),
    opt(('#', fragment).take()),
  );

  parser.parse_next(&mut input)
}

#[cfg(test)]
pub(crate) mod test {
  use std::{path::PathBuf, sync::Arc};

  use rspack_cacheable::{cacheable, cacheable_dyn};
  use rspack_collections::Identifier;

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
    assert_eq!(l.path, PathBuf::from("/a/b/c.js"));
    assert_eq!(l.query, Some("?{\"c\": \"#foo\"}".into()));
    assert_eq!(l.fragment, None);
  }

  #[test]
  fn should_handle_win_non_len_blank_unicode_correctly() {
    let c1 = Arc::new(WinNonLenBlankUnicode) as Arc<dyn Loader<()>>;
    let l: LoaderItem<()> = c1.into();
    assert_eq!(l.path, PathBuf::from(r#"\a\b\c.js"#));
    assert_eq!(l.query, Some("?{\"c\": \"#foo\"}".into()));
    assert_eq!(l.fragment, None);
  }
}
