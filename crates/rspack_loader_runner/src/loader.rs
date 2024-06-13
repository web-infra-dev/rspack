use std::{
  fmt::Display,
  ops::Deref,
  path::PathBuf,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};

use async_trait::async_trait;
use derivative::Derivative;
use once_cell::sync::Lazy;
use regex::Regex;
use rspack_error::Result;
use rspack_identifier::{Identifiable, Identifier};

use super::LoaderContext;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct LoaderItem<Context> {
  #[derivative(Debug(format_with = "fmt_loader"))]
  loader: Arc<dyn Loader<Context>>,
  /// Loader identifier
  request: Identifier,
  /// An absolute path or a virtual path for represent the loader.
  /// The absolute path is used to represent a loader stayed on the JS side.
  /// `$` split chain may be used to represent a composed loader chain from the JS side.
  /// Virtual path with a builtin protocol to represent a loader from the native side. e.g "builtin:".
  path: PathBuf,
  /// Query of a loader, starts with `?`
  query: Option<String>,
  /// Fragment of a loader, starts with `#`.
  fragment: Option<String>,
  /// Data shared between pitching and normal
  data: serde_json::Value,
  r#type: String,
  pitch_executed: AtomicBool,
  normal_executed: AtomicBool,
}

impl<C> LoaderItem<C> {
  pub fn loader(&self) -> &Arc<dyn Loader<C>> {
    &self.loader
  }

  #[inline]
  pub fn request(&self) -> Identifier {
    self.request
  }

  #[inline]
  pub fn r#type(&self) -> &str {
    &self.r#type
  }

  #[inline]
  pub fn data(&self) -> &serde_json::Value {
    &self.data
  }

  #[inline]
  pub fn set_data(&mut self, data: serde_json::Value) {
    self.data = data;
  }

  #[inline]
  pub fn pitch_executed(&self) -> bool {
    self.pitch_executed.load(Ordering::Relaxed)
  }

  #[inline]
  pub fn normal_executed(&self) -> bool {
    self.normal_executed.load(Ordering::Relaxed)
  }

  #[inline]
  pub fn set_pitch_executed(&self) {
    self.pitch_executed.store(true, Ordering::Relaxed)
  }

  #[inline]
  pub fn set_normal_executed(&self) {
    self.normal_executed.store(true, Ordering::Relaxed)
  }
}

impl<C> Display for LoaderItem<C> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.loader.identifier())
  }
}

pub struct LoaderItemList<'a, Context>(pub &'a [LoaderItem<Context>]);

impl<'a, Context> Deref for LoaderItemList<'a, Context> {
  type Target = [LoaderItem<Context>];

  fn deref(&self) -> &Self::Target {
    self.0
  }
}

impl<'a, Context> Default for LoaderItemList<'a, Context> {
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

impl<'a, Context> DisplayWithSuffix for LoaderItemList<'a, Context> {}
impl<Context> DisplayWithSuffix for LoaderItem<Context> {}
impl<'a, Context> Display for LoaderItemList<'a, Context> {
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

impl<C> Identifiable for LoaderItem<C> {
  fn identifier(&self) -> Identifier {
    self.loader.identifier()
  }
}

#[async_trait]
pub trait Loader<Context = ()>: Identifiable + Send + Sync {
  async fn run(&self, _loader_context: &mut LoaderContext<Context>) -> Result<()> {
    // noop
    Ok(())
  }
  async fn pitch(&self, _loader_context: &mut LoaderContext<Context>) -> Result<()> {
    // noop
    Ok(())
  }
}

pub fn fmt_loader<T>(
  loader: &Arc<dyn Loader<T>>,
  fmt: &mut std::fmt::Formatter<'_>,
) -> Result<(), std::fmt::Error> {
  write!(fmt, "{}", loader.identifier())
}

impl<C> From<Arc<dyn Loader<C>>> for LoaderItem<C> {
  fn from(loader: Arc<dyn Loader<C>>) -> Self {
    if let Some((r#type, ident)) = loader.identifier().split_once('|') {
      let ResourceParsedData {
        path,
        query,
        fragment,
      } = parse_resource(ident).expect("identifier should be valid");
      return Self {
        loader,
        request: ident.into(),
        path,
        query,
        fragment,
        data: serde_json::Value::Null,
        r#type: r#type.to_string(),
        pitch_executed: AtomicBool::new(false),
        normal_executed: AtomicBool::new(false),
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
      data: serde_json::Value::Null,
      r#type: String::default(),
      pitch_executed: AtomicBool::new(false),
      normal_executed: AtomicBool::new(false),
    }
  }
}
struct ResourceParsedData {
  pub path: PathBuf,
  pub query: Option<String>,
  pub fragment: Option<String>,
}

fn parse_resource(resource: &str) -> Option<ResourceParsedData> {
  let groups = PATH_QUERY_FRAGMENT_REGEXP.captures(resource)?;

  Some(ResourceParsedData {
    path: groups.get(1)?.as_str().into(),
    query: groups.get(2).map(|q| q.as_str().to_owned()),
    fragment: groups.get(3).map(|q| q.as_str().to_owned()),
  })
}

static PATH_QUERY_FRAGMENT_REGEXP: Lazy<Regex> = Lazy::new(|| {
  Regex::new("^((?:\u{200b}.|[^?#\u{200b}])*)(\\?(?:\u{200b}.|[^#\u{200b}])*)?(#.*)?$")
    .expect("Failed to initialize `PATH_QUERY_FRAGMENT_REGEXP`")
});

#[cfg(test)]
pub(crate) mod test {
  use std::{path::PathBuf, sync::Arc};

  use rspack_identifier::{Identifiable, Identifier};

  use super::{Loader, LoaderItem};

  pub(crate) struct Custom;

  #[async_trait::async_trait]
  impl Loader<()> for Custom {}
  impl Identifiable for Custom {
    fn identifier(&self) -> Identifier {
      "/rspack/custom-loader-1/index.js?foo=1#baz".into()
    }
  }

  pub(crate) struct Custom2;
  #[async_trait::async_trait]
  impl Loader<()> for Custom2 {}
  impl Identifiable for Custom2 {
    fn identifier(&self) -> Identifier {
      "/rspack/custom-loader-2/index.js?bar=2#baz".into()
    }
  }

  pub(crate) struct Builtin;
  #[async_trait::async_trait]
  impl Loader<()> for Builtin {}
  impl Identifiable for Builtin {
    fn identifier(&self) -> Identifier {
      "builtin:test-loader".into()
    }
  }

  pub(crate) struct PosixNonLenBlankUnicode;

  #[async_trait::async_trait]
  impl Loader<()> for PosixNonLenBlankUnicode {}
  impl Identifiable for PosixNonLenBlankUnicode {
    fn identifier(&self) -> Identifier {
      "/a/b/c.js?{\"c\": \"\u{200b}#foo\"}".into()
    }
  }

  pub(crate) struct WinNonLenBlankUnicode;
  #[async_trait::async_trait]
  impl Loader<()> for WinNonLenBlankUnicode {}
  impl Identifiable for WinNonLenBlankUnicode {
    fn identifier(&self) -> Identifier {
      "\\a\\b\\c.js?{\"c\": \"\u{200b}#foo\"}".into()
    }
  }

  #[test]
  fn should_handle_posix_non_len_blank_unicode_correctly() {
    let c1 = Arc::new(PosixNonLenBlankUnicode) as Arc<dyn Loader<()>>;
    let l: LoaderItem<()> = c1.into();
    assert_eq!(l.path, PathBuf::from("/a/b/c.js"));
    assert_eq!(l.query, Some("?{\"c\": \"\u{200b}#foo\"}".into()));
    assert_eq!(l.fragment, None);
  }

  #[test]
  fn should_handle_win_non_len_blank_unicode_correctly() {
    let c1 = Arc::new(WinNonLenBlankUnicode) as Arc<dyn Loader<()>>;
    let l: LoaderItem<()> = c1.into();
    assert_eq!(l.path, PathBuf::from(r#"\a\b\c.js"#));
    assert_eq!(l.query, Some("?{\"c\": \"\u{200b}#foo\"}".into()));
    assert_eq!(l.fragment, None);
  }
}
