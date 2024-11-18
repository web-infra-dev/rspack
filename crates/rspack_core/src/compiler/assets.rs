use std::borrow::Borrow;

use rspack_sources::BoxSource;
use rustc_hash::{FxHashMap, FxHashSet};

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct AssetFilename(rspack_util::atom::Atom);

impl AssetFilename {
  pub fn new<S>(s: S) -> Self
  where
    rspack_util::atom::Atom: From<S>,
  {
    Self(rspack_util::atom::Atom::from(s))
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl std::ops::Deref for AssetFilename {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

macro_rules! impl_eq {
  ($T:ty) => {
    impl PartialEq<$T> for AssetFilename {
      fn eq(&self, other: &$T) -> bool {
        &**self == &**other
      }
    }
  };
}

macro_rules! impl_from {
  ($T:ty) => {
    impl From<$T> for AssetFilename {
      fn from(s: $T) -> Self {
        Self::new(s)
      }
    }
  };
}

impl PartialEq<str> for AssetFilename {
  fn eq(&self, other: &str) -> bool {
    &**self == other
  }
}

impl_eq!(&'_ str);
impl_eq!(Box<str>);
impl_eq!(std::sync::Arc<str>);
impl_eq!(std::rc::Rc<str>);
impl_eq!(std::borrow::Cow<'_, str>);
impl_eq!(String);

impl_from!(&'_ str);
impl_from!(Box<str>);
impl_from!(String);
impl_from!(std::borrow::Cow<'_, str>);

impl AsRef<str> for AssetFilename {
  fn as_ref(&self) -> &str {
    self
  }
}

impl Borrow<str> for AssetFilename {
  fn borrow(&self) -> &str {
    self
  }
}

impl std::fmt::Debug for AssetFilename {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Debug::fmt(&**self, f)
  }
}

impl std::fmt::Display for AssetFilename {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Display::fmt(&**self, f)
  }
}

impl PartialOrd for AssetFilename {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for AssetFilename {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.as_str().cmp(other.as_str())
  }
}

pub type CompilationAssets = FxHashMap<AssetFilename, CompilationAsset>;

#[derive(Debug, Clone)]
pub struct CompilationAsset {
  pub source: Option<BoxSource>,
  pub info: AssetInfo,
}

impl From<BoxSource> for CompilationAsset {
  fn from(value: BoxSource) -> Self {
    Self::new(Some(value), Default::default())
  }
}

impl CompilationAsset {
  pub fn new(source: Option<BoxSource>, info: AssetInfo) -> Self {
    Self { source, info }
  }

  pub fn get_source(&self) -> Option<&BoxSource> {
    self.source.as_ref()
  }

  pub fn get_source_mut(&mut self) -> Option<&mut BoxSource> {
    self.source.as_mut()
  }

  pub fn set_source(&mut self, source: Option<BoxSource>) {
    self.source = source;
  }

  pub fn get_info(&self) -> &AssetInfo {
    &self.info
  }

  pub fn get_info_mut(&mut self) -> &mut AssetInfo {
    &mut self.info
  }

  pub fn set_info(&mut self, info: AssetInfo) {
    self.info = info;
  }
}

#[derive(Debug, Default, Clone)]
pub struct AssetInfo {
  /// if the asset can be long term cached forever (contains a hash)
  pub immutable: Option<bool>,
  /// whether the asset is minimized
  pub minimized: Option<bool>,
  /// the value(s) of the full hash used for this asset
  pub full_hash: FxHashSet<String>,
  /// the value(s) of the chunk hash used for this asset
  pub chunk_hash: FxHashSet<String>,
  /// the value(s) of the module hash used for this asset
  // pub module_hash:
  /// the value(s) of the content hash used for this asset
  pub content_hash: FxHashSet<String>,
  /// when asset was created from a source file (potentially transformed), the original filename relative to compilation context
  pub source_filename: Option<String>,
  /// when asset was created from a source file (potentially transformed), it should be flagged as copied
  pub copied: Option<bool>,
  /// size in bytes, only set after asset has been emitted
  // pub size: f64,
  /// when asset is only used for development and doesn't count towards user-facing assets
  pub development: Option<bool>,
  /// when asset ships data for updating an existing application (HMR)
  pub hot_module_replacement: Option<bool>,
  /// when asset is javascript and an ESM
  pub javascript_module: Option<bool>,
  /// related object to other assets, keyed by type of relation (only points from parent to child)
  pub related: AssetInfoRelated,
  /// the asset version, emit can be skipped when both filename and version are the same
  /// An empty string means no version, it will always emit
  pub version: String,
  /// unused local idents of the chunk
  pub css_unused_idents: Option<FxHashSet<String>>,
  /// Webpack: AssetInfo = KnownAssetInfo & Record<string, any>
  /// But Napi.rs does not support Intersectiont types. This is a hack to store the additional fields
  /// in the rust struct and have the Js side to reshape and align with webpack.
  /// Related: packages/rspack/src/Compilation.ts
  pub extras: serde_json::Map<String, serde_json::Value>,
  /// whether this asset is over the size limit
  pub is_over_size_limit: Option<bool>,
}

impl AssetInfo {
  pub fn with_minimized(mut self, v: Option<bool>) -> Self {
    self.minimized = v;
    self
  }

  pub fn with_development(mut self, v: Option<bool>) -> Self {
    self.development = v;
    self
  }

  pub fn with_hot_module_replacement(mut self, v: Option<bool>) -> Self {
    self.hot_module_replacement = v;
    self
  }

  pub fn with_related(mut self, v: AssetInfoRelated) -> Self {
    self.related = v;
    self
  }

  pub fn with_content_hashes(mut self, v: FxHashSet<String>) -> Self {
    self.content_hash = v;
    self
  }

  pub fn with_version(mut self, v: String) -> Self {
    self.version = v;
    self
  }

  pub fn set_full_hash(&mut self, v: String) {
    self.full_hash.insert(v);
  }

  pub fn set_content_hash(&mut self, v: String) {
    self.content_hash.insert(v);
  }

  pub fn set_chunk_hash(&mut self, v: String) {
    self.chunk_hash.insert(v);
  }

  pub fn set_immutable(&mut self, v: Option<bool>) {
    self.immutable = v;
  }

  pub fn set_source_filename(&mut self, v: String) {
    self.source_filename = Some(v);
  }

  pub fn set_javascript_module(&mut self, v: bool) {
    self.javascript_module = Some(v);
  }

  pub fn set_css_unused_idents(&mut self, v: FxHashSet<String>) {
    self.css_unused_idents = Some(v);
  }

  pub fn set_is_over_size_limit(&mut self, v: bool) {
    self.is_over_size_limit = Some(v);
  }
  // another should have high priority than self
  // self = { immutable:true}
  // merge_another_asset({immutable: false})
  // self == { immutable: false}
  // align with https://github.com/webpack/webpack/blob/899f06934391baede59da3dcd35b5ef51c675dbe/lib/Compilation.js#L4554
  pub fn merge_another_asset(&mut self, another: AssetInfo) {
    // "another" first fields
    self.minimized = another.minimized;

    self.source_filename = another.source_filename.or(self.source_filename.take());
    self.version = another.version;
    self.related.merge_another(another.related);

    // merge vec fields
    self.chunk_hash.extend(another.chunk_hash);
    self.content_hash.extend(another.content_hash);
    self.extras.extend(another.extras);
    // self.full_hash.extend(another.full_hash.iter().cloned());
    // self.module_hash.extend(another.module_hash.iter().cloned());

    // old first fields or truthy first fields
    self.javascript_module = another.javascript_module.or(self.javascript_module.take());
    self.immutable = another.immutable.or(self.immutable);
    self.development = another.development.or(self.development);
    self.hot_module_replacement = another
      .hot_module_replacement
      .or(self.hot_module_replacement);
    self.is_over_size_limit = another.is_over_size_limit.or(self.is_over_size_limit);
  }
}

#[derive(Debug, Default, Clone)]
pub struct AssetInfoRelated {
  pub source_map: Option<AssetFilename>,
}

impl AssetInfoRelated {
  pub fn merge_another(&mut self, another: AssetInfoRelated) {
    if let Some(source_map) = another.source_map {
      self.source_map = Some(source_map);
    }
  }
}
