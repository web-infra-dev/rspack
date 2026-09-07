use std::{
  borrow::{Borrow, Cow},
  convert::AsRef,
  future::Future,
  hash::{BuildHasherDefault, Hash, Hasher},
  io,
  ops::Deref,
  path::PathBuf,
  sync::Arc,
};

use camino::{Utf8Component, Utf8Path, Utf8PathBuf};
use dashmap::{DashMap, DashSet};
use futures::future::BoxFuture;
use rspack_paths::{InternedPath, hash_path};
use rustc_hash::FxHasher;
use tokio::sync::OnceCell as OnceLock;

use crate::{
  FileMetadata, FileSystem, JSONError, ResolveError, ResolveOptions, TsConfig,
  context::ResolveContext as Ctx,
  package_json::{PackageJson, off_to_location},
};

#[derive(Default)]
pub struct Cache<Fs> {
  pub(crate) fs: Fs,
  paths: DashSet<CachedPath, BuildHasherDefault<IdentityHasher>>,
  tsconfigs: DashMap<PathBuf, Arc<TsConfig>, BuildHasherDefault<FxHasher>>,
}

impl<Fs: Send + Sync + FileSystem> Cache<Fs> {
  pub fn new(fs: Fs) -> Self {
    Self {
      fs,
      paths: DashSet::default(),
      tsconfigs: DashMap::default(),
    }
  }

  pub fn clear(&self) {
    self.paths.clear();
    self.tsconfigs.clear();
  }

  pub fn value(&self, path: &Utf8Path) -> CachedPath {
    let hash = hash_path(path.as_std_path());
    if let Some(cache_entry) = self.paths.get((hash, path).borrow() as &dyn CacheKey) {
      return cache_entry.clone();
    }
    let parent = path.parent().map(|p| self.value(p));
    let data = CachedPath(Arc::new(CachedPathImpl::new(
      hash,
      path.to_path_buf().into_boxed_path(),
      parent,
    )));
    self.paths.insert(data.clone());
    data
  }

  pub async fn tsconfig<F, Fut>(
    &self,
    root: bool,
    path: &Utf8Path,
    callback: F, // callback for modifying tsconfig with `extends`
  ) -> Result<Arc<TsConfig>, ResolveError>
  where
    F: FnOnce(TsConfig) -> Fut + Send,
    Fut: Send + Future<Output = Result<TsConfig, ResolveError>>,
  {
    if let Some(tsconfig_ref) = self.tsconfigs.get(path.as_std_path()) {
      return Ok(Arc::clone(tsconfig_ref.value()));
    }
    let meta = self.fs.metadata(path.as_std_path()).await.ok();
    let tsconfig_path = if meta.is_some_and(|m| m.is_file) {
      Cow::Borrowed(path)
    } else if meta.is_some_and(|m| m.is_dir) {
      Cow::Owned(path.join("tsconfig.json"))
    } else {
      let mut string = path.as_str().to_string();
      string.push_str(".json");
      Cow::Owned(Utf8PathBuf::from(string))
    };
    let mut tsconfig_string = self
      .fs
      .read_to_string(tsconfig_path.as_std_path())
      .await
      .map_err(|_| ResolveError::TsconfigNotFound(path.as_std_path().to_path_buf()))?;
    let mut tsconfig =
      TsConfig::parse(root, &tsconfig_path, &mut tsconfig_string).map_err(|error| {
        ResolveError::from_serde_json_error(
          tsconfig_path.as_std_path().to_path_buf(),
          &error,
          Some(tsconfig_string),
        )
      })?;
    tsconfig = callback(tsconfig).await?;
    let tsconfig = Arc::new(tsconfig.build());
    self
      .tsconfigs
      .insert(path.as_std_path().to_path_buf(), Arc::clone(&tsconfig));
    Ok(tsconfig)
  }
}

#[derive(Clone)]
pub struct CachedPath(Arc<CachedPathImpl>);

impl Hash for CachedPath {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.0.hash.hash(state);
  }
}

impl PartialEq for CachedPath {
  fn eq(&self, other: &Self) -> bool {
    // Compare through std `Path`, not camino's `Utf8Path`. std `Path`/`Components`
    // equality has a raw-byte `memcmp` fast path (its own comment: "for hashmap
    // lookups"), whereas camino's `Utf8Path` always walks components with no fast
    // path — that regressed this hottest cache-lookup equality. `as_std_path()`
    // also preserves the per-platform semantics that match `hash_path`
    // (byte-wise on Unix; component-wise on Windows, so `pack1/foo` and
    // `pack1\foo` stay the same entry). A plain `as_str()` byte compare would be
    // inconsistent with the component-wise hash on Windows and break dedup there.
    self.0.path.as_std_path() == other.0.path.as_std_path()
  }
}
impl Eq for CachedPath {}

impl Deref for CachedPath {
  type Target = CachedPathImpl;

  fn deref(&self) -> &Self::Target {
    self.0.as_ref()
  }
}

impl<'a> Borrow<dyn CacheKey + 'a> for CachedPath {
  fn borrow(&self) -> &(dyn CacheKey + 'a) {
    self
  }
}

impl AsRef<CachedPathImpl> for CachedPath {
  fn as_ref(&self) -> &CachedPathImpl {
    self.0.as_ref()
  }
}

impl CacheKey for CachedPath {
  fn tuple(&self) -> (u64, &Utf8Path) {
    (self.hash, &self.path)
  }
}

pub struct CachedPathImpl {
  hash: u64,
  path: Box<Utf8Path>,
  parent: Option<CachedPath>,
  meta: OnceLock<Option<FileMetadata>>,
  canonicalized: OnceLock<Option<Utf8PathBuf>>,
  node_modules: OnceLock<Option<CachedPath>>,
  package_json: OnceLock<Option<Arc<PackageJson>>>,
  /// Memoized `<self.path>/package.json` `InternedPath` for the
  /// `missing_dependencies` push that fires on every `package_json` cache-hit
  /// `None` (~97% of `package_json` calls in dep-tracking workloads).
  package_json_dep_path: std::sync::OnceLock<InternedPath>,
}

impl From<&CachedPathImpl> for InternedPath {
  /// Reuse the cache-side `FxHash` (already computed in `Cache::value`), so interning the
  /// dependency into the `ResolveContext` sink is a lookup with no rehashing, and allocates
  /// nothing at all for a path that has been seen before.
  fn from(cached: &CachedPathImpl) -> Self {
    Self::from_parts(cached.hash, cached.path.as_std_path())
  }
}

impl CachedPathImpl {
  fn new(hash: u64, path: Box<Utf8Path>, parent: Option<CachedPath>) -> Self {
    Self {
      hash,
      path,
      parent,
      meta: OnceLock::new(),
      canonicalized: OnceLock::new(),
      node_modules: OnceLock::new(),
      package_json: OnceLock::new(),
      package_json_dep_path: std::sync::OnceLock::new(),
    }
  }

  /// Without this cache, each `None` cache-hit on `package_json` would
  /// re-`join` + re-allocate the `Arc<Path>` + re-hash on every push.
  fn package_json_dep_path(&self) -> InternedPath {
    self
      .package_json_dep_path
      .get_or_init(|| self.path.join("package.json").into())
      .clone()
  }

  pub fn path(&self) -> &Utf8Path {
    &self.path
  }

  pub fn to_path_buf(&self) -> Utf8PathBuf {
    self.path.to_path_buf()
  }

  pub fn parent(&self) -> Option<&CachedPath> {
    self.parent.as_ref()
  }

  async fn meta<Fs: Send + Sync + FileSystem>(&self, fs: &Fs) -> Option<FileMetadata> {
    // Skip the Future state-machine + poll on cache hit. `tokio::sync::OnceCell::get`
    // is sync and bypasses constructing the `get_or_init` future entirely.
    if let Some(m) = self.meta.get() {
      return *m;
    }
    *self
      .meta
      .get_or_init(|| async { fs.metadata(self.path.as_std_path()).await.ok() })
      .await
  }

  pub async fn is_file<Fs: Send + Sync + FileSystem>(&self, fs: &Fs, ctx: &mut Ctx) -> bool {
    if let Some(meta) = self.meta(fs).await {
      ctx.add_file_dependency(self);
      meta.is_file
    } else {
      ctx.add_missing_dependency(self);
      false
    }
  }

  pub async fn is_dir<Fs: Send + Sync + FileSystem>(&self, fs: &Fs, ctx: &mut Ctx) -> bool {
    self.meta(fs).await.map_or_else(
      || {
        ctx.add_missing_dependency(self);
        false
      },
      |meta| meta.is_dir,
    )
  }

  pub async fn realpath<Fs: FileSystem + Send + Sync>(&self, fs: &Fs) -> io::Result<Utf8PathBuf> {
    // Cache hit: avoid the heap-allocated `Box::pin` for the cache-miss state machine
    // by returning before delegating to the boxed recursive helper.
    if let Some(cached) = self.canonicalized.get() {
      return Ok(cached.clone().unwrap_or_else(|| self.path.to_path_buf()));
    }
    self.realpath_uncached(fs).await
  }

  fn realpath_uncached<'a, Fs: FileSystem + Send + Sync>(
    &'a self,
    fs: &'a Fs,
  ) -> BoxFuture<'a, io::Result<Utf8PathBuf>> {
    Box::pin(async move {
      self
        .canonicalized
        .get_or_try_init(|| async move {
          if fs
            .symlink_metadata(self.path.as_std_path())
            .await
            .is_ok_and(|m| m.is_symlink)
          {
            return fs
              .canonicalize(self.path.as_std_path())
              .await
              .map(|path| Some(Utf8PathBuf::from_path_buf(path).expect("path should be UTF-8")));
          }
          if let Some(parent) = self.parent() {
            let mut real_path = parent.realpath(fs).await?;
            // Unnormalized paths (e.g. from alias values or absolute specifiers) can
            // end in `..`, where `file_name()` returns `None` and would silently drop
            // the component; POSIX semantics pop it after the parent is resolved.
            match self.path.components().next_back() {
              Some(Utf8Component::Normal(segment)) => real_path.push(segment),
              Some(Utf8Component::ParentDir) => {
                real_path.pop();
              }
              _ => {}
            }
            return Ok(Some(real_path));
          }
          Ok(None)
        })
        .await
        .cloned()
        .map(|r| r.unwrap_or_else(|| self.path.to_path_buf()))
    })
  }

  pub async fn module_directory<Fs: Send + Sync + FileSystem>(
    &self,
    module_name: &str,
    cache: &Cache<Fs>,
    ctx: &mut Ctx,
  ) -> Option<CachedPath> {
    let cached_path = cache.value(&self.path.join(module_name));
    cached_path
      .is_dir(&cache.fs, ctx)
      .await
      .then_some(cached_path)
  }

  pub async fn cached_node_modules<Fs: Send + Sync + FileSystem>(
    &self,
    cache: &Cache<Fs>,
    ctx: &mut Ctx,
  ) -> Option<CachedPath> {
    if let Some(nm) = self.node_modules.get() {
      // Replay ctx tracking from the cold path: module_directory -> is_dir calls
      // ctx.add_missing_dependency when node_modules doesn't exist on disk.
      if nm.is_none() {
        ctx.add_missing_dependency(self.path.join("node_modules"));
      }
      return nm.clone();
    }
    self
      .node_modules
      .get_or_init(|| self.module_directory("node_modules", cache, ctx))
      .await
      .clone()
  }

  /// Find package.json of a path by traversing parent directories.
  ///
  /// # Errors
  ///
  /// * [ResolveError::JSON]
  #[cfg_attr(feature="enable_instrument", tracing::instrument(level=tracing::Level::DEBUG, skip_all, fields(path = %self.path)))]
  pub async fn find_package_json<Fs: FileSystem + Send + Sync>(
    &self,
    fs: &Fs,
    options: &ResolveOptions,
    ctx: &mut Ctx,
  ) -> Result<Option<Arc<PackageJson>>, ResolveError> {
    let mut cache_value = self;
    // Go up directories when the querying path is not a directory
    while !cache_value.is_dir(fs, ctx).await {
      if let Some(cv) = &cache_value.parent {
        cache_value = cv.as_ref();
      } else {
        break;
      }
    }
    let mut cache_value = Some(cache_value);
    while let Some(cv) = cache_value {
      if let Some(package_json) = cv.package_json(fs, options, ctx).await? {
        return Ok(Some(Arc::clone(&package_json)));
      }
      cache_value = cv.parent.as_deref();
    }
    Ok(None)
  }

  /// Get package.json of the given path.
  ///
  /// # Errors
  ///
  /// * [ResolveError::JSON]
  #[cfg_attr(feature="enable_instrument", tracing::instrument(level=tracing::Level::DEBUG, skip_all, fields(path = %self.path)))]
  pub async fn package_json<Fs: FileSystem + Send + Sync>(
    &self,
    fs: &Fs,
    options: &ResolveOptions,
    ctx: &mut Ctx,
  ) -> Result<Option<Arc<PackageJson>>, ResolveError> {
    if let Some(pkg) = self.package_json.get() {
      // Preserve ctx dependency tracking on cache hit.
      match pkg {
        Some(package_json) => ctx.add_file_dependency(&package_json.path),
        None => {
          if ctx.missing_dependencies.is_some() {
            ctx.add_missing_dependency(self.package_json_dep_path());
          }
        }
      }
      return Ok(pkg.clone());
    }
    // Change to `std::sync::OnceLock::get_or_try_init` when it is stable.
    let result = self
      .package_json
      .get_or_try_init(|| async {
        let package_json_path = self.path.join("package.json");
        let Ok(package_json_string) = fs.read(package_json_path.as_std_path()).await else {
          return Ok(None);
        };
        let real_path = if options.symlinks {
          self.realpath(fs).await?.join("package.json")
        } else {
          package_json_path.clone()
        };
        match PackageJson::parse(
          package_json_path.clone().into(),
          real_path.into(),
          package_json_string,
        ) {
          Ok(v) => Ok(Some(Arc::new(v))),
          Err(parse_err) => {
            let package_json_path = self.path.join("package.json");
            let package_json_string = match fs.read_to_string(package_json_path.as_std_path()).await
            {
              Ok(c) => c,
              Err(io_err) => {
                return Err(ResolveError::from(io_err));
              }
            };
            let serde_err = serde_json::from_str::<serde_json::Value>(&package_json_string).err();

            if let Some(err) = serde_err {
              Err(ResolveError::from_serde_json_error(
                package_json_path.into(),
                &err,
                Some(package_json_string),
              ))
            } else {
              let (line, column) = off_to_location(&package_json_string, parse_err.index());

              Err(ResolveError::JSON(JSONError {
                path: package_json_path.into(),
                message: parse_err.error().to_string(),
                line,
                column,
                content: Some(package_json_string),
              }))
            }
          }
        }
      })
      .await
      .cloned();

    // https://github.com/webpack/enhanced-resolve/blob/58464fc7cb56673c9aa849e68e6300239601e615/lib/DescriptionFileUtils.js#L68-L82
    match &result {
      Ok(Some(package_json)) => {
        ctx.add_file_dependency(&package_json.path);
      }
      Ok(None) => {
        // Avoid an allocation by making this lazy
        if ctx.missing_dependencies.is_some() {
          ctx.add_missing_dependency(self.package_json_dep_path());
        }
      }
      Err(_) => {
        if ctx.file_dependencies.is_some() {
          ctx.add_file_dependency(self.path.join("package.json"));
        }
      }
    }
    result
  }
}

/// Memoized cache key, code adapted from <https://stackoverflow.com/a/50478038>.
trait CacheKey {
  fn tuple(&self) -> (u64, &Utf8Path);
}

impl Hash for dyn CacheKey + '_ {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.tuple().0.hash(state);
  }
}

impl PartialEq for dyn CacheKey + '_ {
  fn eq(&self, other: &Self) -> bool {
    // std `Path` equality (memcmp fast path + per-platform semantics matching
    // `hash_path`); see `CachedPath`'s `PartialEq` for the full rationale.
    self.tuple().1.as_std_path() == other.tuple().1.as_std_path()
  }
}

impl Eq for dyn CacheKey + '_ {}

impl CacheKey for (u64, &Utf8Path) {
  fn tuple(&self) -> (u64, &Utf8Path) {
    (self.0, self.1)
  }
}

impl<'a> Borrow<dyn CacheKey + 'a> for (u64, &'a Utf8Path) {
  fn borrow(&self) -> &(dyn CacheKey + 'a) {
    self
  }
}

/// Since the cache key is memoized, use an identity hasher
/// to avoid double cache.
#[derive(Default)]
struct IdentityHasher(u64);

impl Hasher for IdentityHasher {
  fn write(&mut self, _: &[u8]) {
    unreachable!("Invalid use of IdentityHasher")
  }
  fn write_u64(&mut self, n: u64) {
    self.0 = n;
  }
  fn finish(&self) -> u64 {
    self.0
  }
}
