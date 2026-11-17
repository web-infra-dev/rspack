//! Experimental [Node.js package map] resolution, selected through
//! [`ResolveOptions::package_map`](crate::ResolveOptions::package_map).
//!
//! A package map contains a `packages` object whose opaque package IDs map to a required `url` and
//! an optional `dependencies` object. Entry URLs are resolved from the location of the map.
//!
//! The resolver API does not propagate a package ID between resolutions, so it uses the
//! specification's path-based fallback to identify the importer. Importers inside a location
//! shared by several package IDs therefore stay ambiguous, as described for [multiple packages
//! sharing one URL]. Selecting a dependency follows one map edge before regular resolution
//! resumes; package-map dependency cycles are not detected, matching the specification's
//! limitation.
//!
//! A map is loaded lazily, because constructing a resolver cannot fail. A resolver and the
//! resolvers cloned from it share the loaded maps, which are reloaded after
//! [`Resolver::clear_cache`](crate::Resolver::clear_cache).
//!
//! [Node.js package map]: https://nodejs.org/api/packages.html#package-maps
//! [multiple packages sharing one URL]: https://nodejs.org/api/packages.html#multiple-packages-for-the-same-url

mod map;

use std::{
  io,
  path::{Path, PathBuf},
  sync::{
    Arc, RwLock,
    atomic::{AtomicU64, Ordering},
  },
};

use camino::{Utf8Path, Utf8PathBuf};
use futures::future::BoxFuture;
use rspack_paths::InternedPath;
use tokio::sync::Mutex;

use self::map::PackageMap;
use crate::{FileSystem, ResolveError, context::ResolveContext as Ctx, path::PathUtil};

/// The outcome of loading a package map, kept until the cache is cleared.
pub(crate) struct LoadedPackageMap {
  /// The map, or the reason why it is invalid.
  result: Result<PackageMap, String>,
  /// The map file, a dependency of every resolution that consults the map.
  path: InternedPath,
  /// Whether the map file was missing, so that creating it invalidates the failed resolutions.
  missing: bool,
}

impl LoadedPackageMap {
  /// Finds the location of the package that `package_name` refers to in the package that contains
  /// the directory `importer`, see [PackageMap::target].
  ///
  /// Returns `None` when the importing package does not depend on `package_name`.
  pub(crate) fn target(
    &self,
    importer: &Utf8Path,
    specifier: &str,
    package_name: &str,
    ctx: &mut Ctx,
  ) -> Result<Option<&Utf8Path>, ResolveError> {
    let dependencies = if self.missing {
      &mut ctx.missing_dependencies
    } else {
      &mut ctx.file_dependencies
    };
    if let Some(dependencies) = dependencies {
      dependencies.push(self.path.clone());
    }
    match &self.result {
      Ok(package_map) => package_map.target(importer, specifier, package_name),
      Err(reason) => Err(ResolveError::PackageMapInvalid {
        package_map_path: self.path.to_path_buf(),
        reason: reason.clone(),
      }),
    }
  }
}

/// The package maps of a resolver and of the resolvers cloned from it, by configured path.
///
/// Nothing here depends on the type of the file system of the resolver, and the functions that
/// resolvers call are not inlined, so that the code is not duplicated for each file system.
#[derive(Default)]
pub(crate) struct PackageMapCache {
  /// Resolvers rarely use more than one map.
  maps: RwLock<Vec<(PathBuf, Arc<LoadedPackageMap>)>>,
  /// Incremented by [Self::clear], so that a map read before the cache was cleared is not kept.
  generation: AtomicU64,
  /// Serializes loading so that concurrent resolutions read and parse a map once.
  loading: Mutex<()>,
}

impl PackageMapCache {
  /// The map at `path`, if it is loaded.
  #[inline(never)]
  pub(crate) fn get(&self, path: &Path) -> Option<Arc<LoadedPackageMap>> {
    self
      .maps
      .read()
      .expect("package map cache was poisoned")
      .iter()
      .find(|(map_path, _)| map_path == path)
      .map(|(_, loaded)| Arc::clone(loaded))
  }

  /// Reads and parses the map at the configured path `key`, and keeps it unless the cache is
  /// cleared meanwhile.
  #[inline(never)]
  pub(crate) fn load<'a>(
    &'a self,
    key: &'a Path,
    fs: &'a (dyn FileSystem + Send + Sync),
  ) -> BoxFuture<'a, Arc<LoadedPackageMap>> {
    Box::pin(async move {
      let _guard = self.loading.lock().await;
      if let Some(loaded) = self.get(key) {
        return loaded;
      }
      let generation = self.generation.load(Ordering::Acquire);
      let mut missing = false;
      let (path, result) = match absolute_path(key) {
        Ok(location) => {
          tracing::debug!(path = ?location, "load_package_map");
          let result = match fs.read_to_string(location.as_std_path()).await {
            Ok(json) => PackageMap::parse(&location, &json),
            Err(error) => {
              missing = error.kind() == io::ErrorKind::NotFound;
              Err(format!("the file cannot be read: {error}"))
            }
          };
          (location.into_std_path_buf(), result)
        }
        Err(reason) => (key.to_path_buf(), Err(reason.to_string())),
      };
      let loaded = LoadedPackageMap {
        result,
        path: InternedPath::from(path.as_path()),
        missing,
      };
      let loaded = Arc::new(loaded);
      let mut maps = self.maps.write().expect("package map cache was poisoned");
      if generation == self.generation.load(Ordering::Acquire) {
        maps.push((key.to_path_buf(), Arc::clone(&loaded)));
      }
      loaded
    })
  }

  pub(crate) fn clear(&self) {
    self.generation.fetch_add(1, Ordering::AcqRel);
    self
      .maps
      .write()
      .expect("package map cache was poisoned")
      .clear();
  }
}

/// Returns the absolute, normalized location of the package map at `path`.
///
/// A relative path is resolved from the current working directory, like the path of Node's
/// `--experimental-package-map` option.
fn absolute_path(path: &Path) -> Result<Utf8PathBuf, &'static str> {
  let path = Utf8Path::from_path(path).ok_or("the path must be valid UTF-8")?;
  if path.is_absolute() {
    return Ok(path.normalize());
  }
  let cwd = std::env::current_dir()
    .ok()
    .and_then(|cwd| Utf8PathBuf::from_path_buf(cwd).ok())
    .ok_or("the path is relative and the working directory is not available")?;
  Ok(cwd.normalize_with(path.as_str()))
}
