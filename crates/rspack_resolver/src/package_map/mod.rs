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
//! The map is loaded lazily, because constructing a resolver cannot fail, and it is reloaded after
//! [`Resolver::clear_cache`](crate::Resolver::clear_cache).
//!
//! [Node.js package map]: https://nodejs.org/api/packages.html#package-maps
//! [multiple packages sharing one URL]: https://nodejs.org/api/packages.html#multiple-packages-for-the-same-url

mod map;

use std::{
  io,
  path::Path,
  sync::{Arc, RwLock},
};

use camino::{Utf8Path, Utf8PathBuf};
use rspack_paths::InternedPath;
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

/// The package map of a resolver, shared by its clones that use the same map.
#[derive(Default)]
pub(crate) struct PackageMapCache {
  value: RwLock<Option<LoadedPackageMap>>,
  /// Serializes loading so that concurrent resolutions read and parse the map once.
  loading: Mutex<()>,
}

impl PackageMapCache {
  pub(crate) async fn get_or_load<Fs: FileSystem>(
    &self,
    path: &Path,
    fs: &Fs,
    ctx: &mut Ctx,
  ) -> Result<Arc<PackageMap>, ResolveError> {
    let loaded = match self.get() {
      Some(loaded) => loaded,
      None => {
        let _guard = self.loading.lock().await;
        match self.get() {
          Some(loaded) => loaded,
          None => {
            let loaded = load(path, fs).await;
            *self.value.write().expect("package map cache was poisoned") = Some(loaded.clone());
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
    *self.value.write().expect("package map cache was poisoned") = None;
  }

  fn get(&self) -> Option<LoadedPackageMap> {
    self
      .value
      .read()
      .expect("package map cache was poisoned")
      .clone()
  }
}

async fn load<Fs: FileSystem>(path: &Path, fs: &Fs) -> LoadedPackageMap {
  let Some(path) = absolute_path(path) else {
    return LoadedPackageMap {
      result: Err(ResolveError::PackageMapInvalid {
        package_map_path: path.to_path_buf(),
        reason: "the path must be valid UTF-8".to_string(),
      }),
      path: InternedPath::from(path),
      missing: false,
    };
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
fn absolute_path(path: &Path) -> Option<Utf8PathBuf> {
  let path = Utf8Path::from_path(path)?;
  if path.is_absolute() {
    return Some(path.normalize());
  }
  let cwd = Utf8PathBuf::from_path_buf(std::env::current_dir().ok()?).ok()?;
  Some(cwd.normalize_with(path))
}
