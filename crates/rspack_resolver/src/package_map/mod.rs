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
use rspack_paths::InternedPath;
use rustc_hash::FxHashMap;
use tokio::sync::Mutex;

pub(crate) use self::map::{FindPackageIdError, PackageMap};
use crate::{FileSystem, ResolveError, context::ResolveContext as Ctx, path::PathUtil};

/// The outcome of loading a package map, kept until the cache is cleared.
#[derive(Clone)]
struct LoadedPackageMap {
  result: Result<Arc<PackageMap>, ResolveError>,
  /// The map file, a dependency of every resolution that consults the map.
  path: InternedPath,
  /// Whether the map file was missing, so that creating it invalidates the failed resolutions.
  missing: bool,
}

/// The package maps of a resolver and of the resolvers cloned from it, by configured path.
#[derive(Default)]
pub(crate) struct PackageMapCache {
  maps: RwLock<FxHashMap<PathBuf, LoadedPackageMap>>,
  /// Incremented by [Self::clear], so that a map read before the cache was cleared is not kept.
  generation: AtomicU64,
  /// Serializes loading so that concurrent resolutions read and parse a map once.
  loading: Mutex<()>,
}

impl PackageMapCache {
  pub(crate) async fn get_or_load<Fs: FileSystem>(
    &self,
    path: &Path,
    fs: &Fs,
    ctx: &mut Ctx,
  ) -> Result<Arc<PackageMap>, ResolveError> {
    let loaded = match self.get(path) {
      Some(loaded) => loaded,
      None => {
        let _guard = self.loading.lock().await;
        match self.get(path) {
          Some(loaded) => loaded,
          None => {
            let generation = self.generation.load(Ordering::Acquire);
            let loaded = load(path, fs).await;
            let mut maps = self.maps.write().expect("package map cache was poisoned");
            if generation == self.generation.load(Ordering::Acquire) {
              maps.insert(path.to_path_buf(), loaded.clone());
            }
            loaded
          }
        }
      }
    };
    if loaded.missing {
      ctx.add_missing_dependency(&loaded.path);
    } else {
      ctx.add_file_dependency(&loaded.path);
    }
    loaded.result
  }

  pub(crate) fn clear(&self) {
    self.generation.fetch_add(1, Ordering::AcqRel);
    self
      .maps
      .write()
      .expect("package map cache was poisoned")
      .clear();
  }

  fn get(&self, path: &Path) -> Option<LoadedPackageMap> {
    self
      .maps
      .read()
      .expect("package map cache was poisoned")
      .get(path)
      .cloned()
  }
}

async fn load<Fs: FileSystem>(path: &Path, fs: &Fs) -> LoadedPackageMap {
  let path = match absolute_path(path) {
    Ok(path) => path,
    Err(reason) => {
      return LoadedPackageMap {
        result: Err(ResolveError::PackageMapInvalid {
          package_map_path: path.to_path_buf(),
          reason: reason.to_string(),
        }),
        path: InternedPath::from(path),
        missing: false,
      };
    }
  };
  tracing::debug!(path = ?path, "load_package_map");
  let (result, missing) = match fs.read_to_string(path.as_std_path()).await {
    Ok(json) => (PackageMap::parse(&path, &json).map(Arc::new), false),
    Err(error) => (
      Err(PackageMap::invalid(
        &path,
        format!("the file cannot be read: {error}"),
      )),
      error.kind() == io::ErrorKind::NotFound,
    ),
  };
  LoadedPackageMap {
    result,
    path: InternedPath::from(path.as_path()),
    missing,
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
  Ok(cwd.normalize_with(path))
}
