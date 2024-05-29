use std::{
  fmt::{Debug, Display},
  ops::Deref,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};

use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::{Match, Regex};
use rspack_error::Result;
use rspack_identifier::{Identifiable, Identifier};

use crate::{runner::LoaderContext, BUILTIN_LOADER_PREFIX};

#[derive(Debug)]
struct LoaderItemDataInner {
  /// Loader identifier
  identifier: Box<str>,
  /// An absolute path or a virtual path for represent the loader.
  /// The absolute path is used to represent a loader stayed on the JS side.
  /// `$` split chain may be used to represent a composed loader chain from the JS side.
  /// Virtual path with a builtin protocol to represent a loader from the native side. e.g "builtin:".
  path: Box<str>,
  /// Query of a loader, starts with `?`
  query: Option<Box<str>>,
  /// Fragment of a loader, starts with `#`.
  fragment: Option<Box<str>>,
  #[allow(unused)]
  meta: LoaderItemDataMeta,
}

#[derive(Debug)]
enum LoaderItemData {
  /// Composed JavaScript loaders
  Composed(Vec<LoaderItemDataInner>),
  /// Normal Loader
  Normal(LoaderItemDataInner),
}

impl LoaderItemData {
  fn is_composed(&self) -> bool {
    matches!(self, LoaderItemData::Composed(_))
  }
}

pub struct LoaderItem<C> {
  pub(crate) loader: Arc<dyn Loader<C>>,
  data: LoaderItemData,
  pitch_executed: AtomicBool,
  normal_executed: AtomicBool,
}

impl<C> Debug for LoaderItem<C> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("LoaderItem")
      .field("data", &self.data)
      .field("loader", &self.loader.identifier())
      .finish()
  }
}

impl<C> LoaderItem<C> {
  pub fn is_composed(&self) -> bool {
    self.data.is_composed()
  }

  pub fn composed_index_by_identifier(&self, ident: &str) -> Option<u32> {
    if let LoaderItemData::Composed(c) = &self.data {
      c.iter().enumerate().find_map(|(idx, inner)| {
        if &*inner.identifier == ident {
          return Some(idx as u32);
        }
        None
      })
    } else {
      None
    }
  }

  pub(crate) fn pitch_executed(&self) -> bool {
    self.pitch_executed.load(Ordering::Relaxed)
  }

  pub(crate) fn normal_executed(&self) -> bool {
    self.normal_executed.load(Ordering::Relaxed)
  }

  pub(crate) fn set_pitch_executed(&self) {
    self.pitch_executed.store(true, Ordering::Relaxed)
  }

  pub(crate) fn set_normal_executed(&self) {
    self.normal_executed.store(true, Ordering::Relaxed)
  }

  pub fn __do_not_use_or_you_will_be_fired_set_normal_executed(&self) {
    self.set_normal_executed()
  }
}

bitflags::bitflags! {
  #[derive(Debug)]
  struct LoaderItemDataMeta: u8 {
    /// Builtin loader
    const BUILTIN = 1 << 0;
    /// JS loader
    const JS = 1 << 1;
  }
}

impl LoaderItemDataMeta {
  fn insert_builtin(&mut self) {
    self.insert(Self::BUILTIN);
  }
  fn insert_js(&mut self) {
    self.insert(Self::JS)
  }
  fn has_builtin(&self) -> bool {
    self.contains(Self::BUILTIN)
  }
  fn has_js(&self) -> bool {
    self.contains(Self::JS)
  }
}

impl Display for LoaderItemDataMeta {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let proto = if self.has_js() {
      "js:"
    } else if self.has_builtin() {
      BUILTIN_LOADER_PREFIX
    } else {
      ""
    };

    write!(f, "{proto}")
  }
}

impl From<&str> for LoaderItemDataMeta {
  fn from(value: &str) -> Self {
    let mut meta = Self::empty();

    if value.starts_with(BUILTIN_LOADER_PREFIX) {
      meta.insert_builtin();
    }

    if value.starts_with("js:") {
      meta.insert_js();
    }

    meta
  }
}

impl Display for LoaderItemDataInner {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.path)?;

    if let Some(query) = &self.query {
      if !query.starts_with('?') {
        write!(f, "?")?;
      }
      write!(f, "{query}")?;
    }
    if let Some(fragment) = &self.fragment {
      if !fragment.starts_with('#') {
        write!(f, "#")?;
      }
      write!(f, "{fragment}")?;
    }

    Ok(())
  }
}

impl Display for LoaderItemData {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Composed(l) => l
          .iter()
          .map(|i| i.to_string())
          .collect::<Vec<_>>()
          .join("$"),
        Self::Normal(l) => l.to_string(),
      }
    )?;

    Ok(())
  }
}

impl<C> Display for LoaderItem<C> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.data)?;

    Ok(())
  }
}

#[derive(Debug)]
pub struct LoaderItemList<'l, C>(pub &'l [LoaderItem<C>]);

impl<'l, C> Deref for LoaderItemList<'l, C> {
  type Target = [LoaderItem<C>];

  fn deref(&self) -> &Self::Target {
    self.0
  }
}

impl<'l, C> Default for LoaderItemList<'l, C> {
  fn default() -> Self {
    Self(&[])
  }
}

pub trait DisplayWithSuffix: Display {
  fn display_with_suffix(&self, suffix: &str) -> String {
    self.to_string() + "!" + suffix
  }
}

impl<'l, C> DisplayWithSuffix for LoaderItemList<'l, C> {}
impl<C> DisplayWithSuffix for LoaderItem<C> {}
impl DisplayWithSuffix for LoaderItemData {}

impl<'l, C> Display for LoaderItemList<'l, C> {
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
pub trait Loader<C = ()>: Identifiable + Send + Sync {
  async fn run(&self, _loader_context: &mut LoaderContext<'_, C>) -> Result<()> {
    // noop
    Ok(())
  }
  async fn pitch(&self, _loader_context: &mut LoaderContext<'_, C>) -> Result<()> {
    // noop
    Ok(())
  }
}

impl<C> From<Arc<dyn Loader<C>>> for LoaderItem<C> {
  fn from(loader: Arc<dyn Loader<C>>) -> Self {
    let ident = loader.identifier();

    // Passthrough composed Node loader
    let data = if ident.contains('$') {
      let loaders = ident
        .split('$')
        .map(convert_to_loader_item_inner)
        .collect::<Vec<_>>();
      assert!(loaders.len() > 1);
      LoaderItemData::Composed(loaders)
    } else {
      LoaderItemData::Normal(convert_to_loader_item_inner(&ident))
    };

    Self {
      data,
      loader,
      pitch_executed: AtomicBool::new(false),
      normal_executed: AtomicBool::new(false),
    }
  }
}

static PATH_QUERY_FRAGMENT_REGEXP: Lazy<Regex> = Lazy::new(|| {
  Regex::new("^((?:\\u200b.|[^?#\\u200b])*)(\\?(?:\\u200b.|[^#\\u200b])*)?(#.*)?$")
    .expect("Failed to initialize `PATH_QUERY_FRAGMENT_REGEXP`")
});

fn to_segment_owned(v: Option<Match<'_>>) -> Option<String> {
  v.map(|v| v.as_str().to_owned())
}

fn convert_to_loader_item_inner(ident: &str) -> LoaderItemDataInner {
  let groups = PATH_QUERY_FRAGMENT_REGEXP
    .captures(ident)
    .expect("Group expected");

  LoaderItemDataInner {
    identifier: ident.into(),
    path: to_segment_owned(groups.get(1))
      .expect("Path expected")
      .into(),
    query: to_segment_owned(groups.get(2)).map(Into::into),
    fragment: to_segment_owned(groups.get(3)).map(Into::into),
    meta: ident.into(),
  }
}

#[cfg(test)]
pub(crate) mod test {
  use std::sync::Arc;

  use rspack_error::{error, Result};
  use rspack_identifier::{Identifiable, Identifier};

  use super::{
    Loader, LoaderContext, LoaderItem, LoaderItemData, LoaderItemDataInner, LoaderItemDataMeta,
    LoaderItemList,
  };

  impl LoaderItemData {
    fn try_as_normal(&self) -> Result<&LoaderItemDataInner> {
      match self {
        LoaderItemData::Normal(i) => Ok(i),
        _ => Err(error!("Failed to cast to normal loader")),
      }
    }

    fn try_as_composed(&self) -> Result<&[LoaderItemDataInner]> {
      match self {
        LoaderItemData::Composed(i) => Ok(i),
        _ => Err(error!("Failed to cast to composed loader")),
      }
    }
  }

  pub(crate) struct Custom;

  #[async_trait::async_trait]
  impl Loader<()> for Custom {
    async fn run(&self, _loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
      Ok(())
    }
    async fn pitch(&self, _loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
      Ok(())
    }
  }

  impl Identifiable for Custom {
    fn identifier(&self) -> Identifier {
      "/rspack/custom-loader-1/index.js?foo=1#baz".into()
    }
  }

  pub(crate) struct Custom2;

  #[async_trait::async_trait]
  impl Loader<()> for Custom2 {
    async fn run(&self, _loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
      Ok(())
    }
    async fn pitch(&self, _loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
      Ok(())
    }
  }

  impl Identifiable for Custom2 {
    fn identifier(&self) -> Identifier {
      "/rspack/custom-loader-2/index.js?bar=2#baz".into()
    }
  }

  pub(crate) struct Composed;

  #[async_trait::async_trait]
  impl Loader<()> for Composed {
    async fn run(&self, _loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
      Ok(())
    }
    async fn pitch(&self, _loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
      Ok(())
    }
  }

  impl Identifiable for Composed {
    fn identifier(&self) -> Identifier {
      "/rspack/style-loader/index.js?foo=1#baz$/rspack/css-loader/index.js?bar=2#qux".into()
    }
  }

  pub(crate) struct Builtin;

  #[async_trait::async_trait]
  impl Loader<()> for Builtin {
    async fn run(&self, _loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
      Ok(())
    }
    async fn pitch(&self, _loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
      Ok(())
    }
  }

  impl Identifiable for Builtin {
    fn identifier(&self) -> Identifier {
      "builtin:test-loader".into()
    }
  }

  #[test]
  fn should_extract_and_compose_loader_info_correctly() {
    let c1 = Arc::new(Custom) as Arc<dyn Loader<()>>;
    let c2 = Arc::new(Custom2) as Arc<dyn Loader<()>>;
    let l: Vec<LoaderItem<()>> = vec![c1.into(), c2.into()];
    let item = l[0].data.try_as_normal().unwrap();
    assert_eq!(item.path, "/rspack/custom-loader-1/index.js".into());
    assert_eq!(item.query, Some("?foo=1".into()));
    assert_eq!(item.fragment, Some("#baz".into()));
    let ll = LoaderItemList(&l[..]);
    assert_eq!(
      ll.to_string(),
      "/rspack/custom-loader-1/index.js?foo=1#baz!/rspack/custom-loader-2/index.js?bar=2#baz"
    );
  }

  #[test]
  fn should_extract_composed_loader_correctly() {
    let c1 = Arc::new(Composed) as Arc<dyn Loader<()>>;
    let ident = c1.identifier();
    let l: LoaderItem<()> = c1.into();
    let items = l.data.try_as_composed().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].path, "/rspack/style-loader/index.js".into());
    assert_eq!(items[0].query, Some("?foo=1".into()));
    assert_eq!(items[0].fragment, Some("#baz".into()));

    assert_eq!(items[1].path, "/rspack/css-loader/index.js".into());
    assert_eq!(items[1].query, Some("?bar=2".into()));
    assert_eq!(items[1].fragment, Some("#qux".into()));

    let i = &[l];
    let ll = LoaderItemList(i);
    assert_eq!(ll.to_string(), ident.to_string());
  }

  #[test]
  fn should_handle_builtin_loader_correctly() {
    let c1 = Arc::new(Builtin) as Arc<dyn Loader<()>>;
    let ident1 = c1.identifier();
    let l: LoaderItem<()> = c1.into();
    let item = l.data.try_as_normal().unwrap();
    assert!(item.meta.contains(LoaderItemDataMeta::BUILTIN));

    let c2 = Arc::new(Composed) as Arc<dyn Loader<()>>;
    let ident2 = c2.identifier();
    let l = vec![l, c2.into()];
    let ll = LoaderItemList(&l[..]);
    assert_eq!(ll.to_string(), ident1.to_string() + "!" + ident2.as_str());
  }
}
