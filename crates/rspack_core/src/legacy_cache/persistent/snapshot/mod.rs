mod scope;
mod strategy;

use std::{
  path::{Path, PathBuf},
  sync::Arc,
};

use rspack_error::Result;
use rspack_fs::ReadableFileSystem;
use rspack_parallel::TryFutureConsumer;
use rspack_paths::{InternedPath, InternedPathMap, InternedPathSet};
use rustc_hash::FxHashSet;

pub(crate) use self::strategy::{StrategyHelper, ValidateResult};
pub use self::{scope::SnapshotScope, strategy::Strategy};
use super::{storage::Storage, validation::is_relocated};
use crate::{
  Context, FutureConsumer, SnapshotOptions, SnapshotStrategyOptions,
  cache::{CacheCodec, RootBinding},
};

/// Snapshot is used to check if files have been modified or deleted.
///
/// Snapshot will generate `Strategy` for target file, and check the modification
/// through the generated `Strategy`
#[derive(Debug)]
pub struct Snapshot {
  options: Arc<SnapshotOptions>,
  context: Context,
  fs: Arc<dyn ReadableFileSystem>,
  codec: Arc<CacheCodec>,
}

impl Snapshot {
  pub fn new(
    options: SnapshotOptions,
    context: Context,
    fs: Arc<dyn ReadableFileSystem>,
    codec: Arc<CacheCodec>,
  ) -> Self {
    Self {
      options: Arc::new(options),
      context,
      fs,
      codec,
    }
  }

  async fn calc_strategy(
    options: &Arc<SnapshotOptions>,
    helper: &Arc<StrategyHelper>,
    path: &InternedPath,
    scope: SnapshotScope,
    portable: bool,
  ) -> Option<Strategy> {
    let mut missing = false;
    match helper.is_symlink(path).await {
      // A context lists the linked directory, so it keeps the content hash. Other scopes only
      // need the link target and a shallow identity of it.
      Ok(true) => {
        return Some(match scope {
          SnapshotScope::CONTEXT => helper.dir_hash(path).await,
          _ => {
            helper
              .link_strategy(path, matches!(scope, SnapshotScope::BUILD))
              .await
          }
        });
      }
      Err(rspack_fs::Error::Io(error))
        if matches!(
          error.kind(),
          std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
        ) =>
      {
        missing = true;
      }
      Err(_) => return Some(Strategy::Failed),
      Ok(false) => {}
    }
    let path_str = path.to_string_lossy();
    if options.is_immutable_path(&path_str) {
      return None;
    }
    if options.is_managed_path(&path_str)
      && let Some(v) = helper.package_version(path).await
    {
      return Some(v);
    }
    Some(match scope {
      SnapshotScope::FILE => {
        helper
          .file_strategy(path, SnapshotStrategyOptions::hash_and_timestamp())
          .await
      }
      SnapshotScope::MISSING => Strategy::Missing,
      // A context directory that does not exist stays valid until it appears, like a missing
      // file. Recording it as failed rebuilt its module on every start.
      SnapshotScope::CONTEXT if missing => Strategy::Missing,
      SnapshotScope::CONTEXT => {
        helper
          .dir_strategy(
            path,
            if portable {
              SnapshotStrategyOptions::hash()
            } else {
              SnapshotStrategyOptions::timestamp()
            },
          )
          .await
      }
      SnapshotScope::BUILD => {
        helper
          .dir_strategy(path, SnapshotStrategyOptions::hash())
          .await
      }
    })
  }

  #[tracing::instrument("Cache::Snapshot::reset", skip_all)]
  pub fn reset(&self, storage: &mut dyn Storage) {
    storage.reset(SnapshotScope::FILE.name());
    storage.reset(SnapshotScope::CONTEXT.name());
    storage.reset(SnapshotScope::MISSING.name());
  }

  #[tracing::instrument("Cache::Snapshot::add", skip_all)]
  pub async fn add(
    &self,
    storage: &mut dyn Storage,
    scope: SnapshotScope,
    paths: impl Iterator<Item = InternedPath>,
  ) {
    self.add_paths(storage, scope, paths, None).await;
  }

  /// Adds context snapshots that no validation observed. The build read these directories after
  /// `start_time`, so one modified since then may differ from what was read. Like webpack, its
  /// snapshot is stored as failed and revalidates on the next start.
  #[tracing::instrument("Cache::Snapshot::add_unobserved_contexts", skip_all)]
  pub async fn add_unobserved_contexts(
    &self,
    storage: &mut dyn Storage,
    paths: impl Iterator<Item = InternedPath>,
    start_time: Option<u64>,
  ) {
    self
      .add_paths(storage, SnapshotScope::CONTEXT, paths, start_time)
      .await;
  }

  async fn add_paths(
    &self,
    storage: &mut dyn Storage,
    scope: SnapshotScope,
    paths: impl Iterator<Item = InternedPath>,
    start_time: Option<u64>,
  ) {
    let helper = Arc::new(StrategyHelper::new(
      self.fs.clone(),
      self.options.clone(),
      self.context.clone(),
      self.codec.clone(),
    ));
    let codec = self.codec.clone();
    // TODO merge package version file
    paths
      .map(|path| {
        let helper = helper.clone();
        let options = self.options.clone();
        let codec = codec.clone();
        async move {
          let mut strategy =
            Self::calc_strategy(&options, &helper, &path, scope, codec.is_portable()).await?;
          if let Some(start_time) = start_time
            && matches!(
              strategy,
              Strategy::DirHash { .. }
                | Strategy::DirTimestamp { .. }
                | Strategy::DirTimestampAndHash { .. }
            )
            && helper.dir_modified_since(&path, start_time).await
          {
            strategy = Strategy::Failed;
          }
          Some((path, strategy))
        }
      })
      .fut_consume(|data| {
        if let Some(data) = data {
          self.refresh(storage, scope, std::iter::once(data));
        }
      })
      .await;
  }

  /// Keep the observation made during validation: inputs can change after the build read them.
  pub fn refresh(
    &self,
    storage: &mut dyn Storage,
    scope: SnapshotScope,
    snapshots: impl Iterator<Item = (InternedPath, Strategy)>,
  ) {
    for (path, strategy) in snapshots {
      let key = self
        .codec
        .encode_path_key(&path)
        .expect("should encode path");
      let layout_key = if path.is_fixed() && self.codec.is_portable() {
        self
          .codec
          .encode_path_key(&path.without_fixed())
          .expect("should encode path")
      } else {
        key.clone()
      };
      // Fixed owners are discarded after relocation. Expire both snapshot encodings too:
      // neither the original absolute key nor its layout counterpart has a surviving owner.
      let bound_root = if path.is_fixed() {
        self.codec.bound_root(RootBinding::Bound)
      } else {
        None
      };
      let value = self
        .codec
        .encode(&(bound_root, strategy))
        .expect("should encode strategy");
      self.remove(storage, scope, std::iter::once(path));
      // Counters merge by filesystem identity. A fixed observation may share its key with an
      // issuer-derived group; retain the layout validation too, whichever group arrived first.
      if key != layout_key {
        storage.set(scope.name(), layout_key, value.clone());
      }
      storage.set(scope.name(), key, value);
    }
  }

  pub fn remove(
    &self,
    storage: &mut dyn Storage,
    scope: SnapshotScope,
    paths: impl Iterator<Item = InternedPath>,
  ) {
    for item in paths {
      storage.remove(
        scope.name(),
        &self
          .codec
          .encode_path_key(&item.clone().into_fixed())
          .expect("should encode path"),
      );
      storage.remove(
        scope.name(),
        &self
          .codec
          .encode_path_key(&item.without_fixed())
          .expect("should encode path"),
      )
    }
  }

  /// Replace only expired records whose layout input independently validated as unchanged.
  pub(crate) fn refresh_expired(
    &self,
    storage: &mut dyn Storage,
    scope: SnapshotScope,
    dropped: &[Vec<u8>],
    unchanged: &InternedPathSet,
    observed: &InternedPathMap<Strategy>,
  ) {
    for key in dropped {
      let path = self
        .codec
        .decode::<InternedPath>(key)
        .expect("expired snapshot keys were checked during validation");
      if !path.is_fixed()
        && unchanged.contains(&path)
        && let Some(strategy) = observed.get(&path)
      {
        self.refresh(storage, scope, std::iter::once((path, strategy.clone())));
      }
    }
  }

  /// Directories below the root that are links here, in a cache written under another root that
  /// did not follow them. The same path can reach other files through such a link, so a matching
  /// hash proves nothing: `node_modules` linked to another checkout resolves into that checkout.
  async fn unrecorded_links(
    helper: &Arc<StrategyHelper>,
    codec: &Arc<CacheCodec>,
    data: &[(Vec<u8>, Vec<u8>)],
  ) -> Result<FxHashSet<PathBuf>> {
    let mut links = FxHashSet::default();
    let Some(root) = codec.portable_root() else {
      return Ok(links);
    };
    let recorded = data
      .iter()
      .map(|(key, _)| codec.decode::<InternedPath>(key))
      .collect::<Result<InternedPathSet>>()?;
    let mut directories = FxHashSet::default();
    for path in &recorded {
      let path: &Path = path.as_ref();
      for ancestor in path.ancestors().skip(1) {
        if ancestor == root.root
          || ancestor == root.canonical_root
          || !(ancestor.starts_with(&root.root) || ancestor.starts_with(&root.canonical_root))
          || !directories.insert(ancestor.to_path_buf())
        {
          break;
        }
      }
    }
    directories
      .into_iter()
      .map(|directory| {
        let helper = helper.clone();
        let path = InternedPath::from(directory.as_path());
        let recorded = recorded.contains(&path);
        async move {
          (!recorded && matches!(helper.is_symlink(&path).await, Ok(true))).then_some(directory)
        }
      })
      .fut_consume(|directory| links.extend(directory))
      .await;
    Ok(links)
  }

  #[allow(clippy::type_complexity)]
  #[tracing::instrument("Cache::Snapshot::calc_modified_path", skip_all)]
  pub async fn calc_modified_paths(
    &self,
    storage: &dyn Storage,
    scope: SnapshotScope,
  ) -> Result<(
    bool,
    InternedPathSet,
    InternedPathSet,
    InternedPathSet,
    InternedPathMap<Strategy>,
    Vec<Vec<u8>>,
  )> {
    let mut dropped_keys = Vec::new();
    let mut modified_path = InternedPathSet::default();
    let mut deleted_path = InternedPathSet::default();
    let mut no_change_path = InternedPathSet::default();
    let mut observed_path = InternedPathMap::default();
    let helper = Arc::new(StrategyHelper::new(
      self.fs.clone(),
      self.options.clone(),
      self.context.clone(),
      self.codec.clone(),
    ));
    let codec = self.codec.clone();

    let data = storage.load(scope.name()).await?;
    let is_hot_start = !data.is_empty();
    // Written under another root: every record's timestamps describe other files.
    let relocated = codec.is_portable() && is_relocated(storage, &codec).await?;
    let unrecorded_links = Arc::new(if relocated && matches!(scope, SnapshotScope::FILE) {
      Self::unrecorded_links(&helper, &codec, &data).await?
    } else {
      Default::default()
    });
    data
      .into_iter()
      .map(|(key, value)| {
        let helper = helper.clone();
        let options = self.options.clone();
        let codec = codec.clone();
        let unrecorded_links = unrecorded_links.clone();
        async move {
          let path = codec.decode::<InternedPath>(&key)?;
          // Unix native path payloads round-trip verbatim. Other platforms may intern a
          // component-equivalent spelling, and portable decoding can rebase a key.
          if (codec.is_portable() || !cfg!(unix)) && codec.encode_path_key(&path)? != key {
            return Ok((path, ValidateResult::Modified, None, Some(key)));
          }
          if !unrecorded_links.is_empty()
            && AsRef::<Path>::as_ref(&path)
              .ancestors()
              .any(|ancestor| unrecorded_links.contains(ancestor))
          {
            return Ok((path, ValidateResult::Modified, None, None));
          }
          let (validate, mut observed) = match codec.decode::<(Option<u64>, Strategy)>(&value) {
            Ok((bound_root, strategy)) => {
              if !codec.accepts_bound_root(bound_root) {
                // A fixed factory observation also writes a layout tombstone because path
                // counters merge filesystem identities. Expire that record, but independently
                // validate the input of any layout owner that survived segmented recovery.
                // Fixed build inputs still bind the build segment; fixed factory groups still
                // expire and refactorize. Global build dependencies have no such owner gate.
                if !path.is_fixed() && !matches!(scope, SnapshotScope::BUILD) {
                  let validate = match helper.validate_relocated(&path, &strategy).await {
                    ValidateResult::NoChanged => ValidateResult::NoChangedRefresh(strategy),
                    validate => validate,
                  };
                  return Ok((path, validate, None, Some(key)));
                }
                return Ok((path, ValidateResult::Modified, None, Some(key)));
              }
              if relocated {
                // Ordinary records carry no root binding, yet their timestamps were observed
                // on the writer's files. Equal times on a copy prove nothing about content or
                // about what the path now resolves to: validate the content instead.
                (helper.validate_relocated(&path, &strategy).await, None)
              } else {
                let (validate, observed) = helper.validate(&path, &strategy).await;
                let observed = observed
                  .or_else(|| matches!(validate, ValidateResult::NoChanged).then_some(strategy));
                (validate, observed)
              }
            }
            Err(_) => (ValidateResult::Modified, None),
          };
          // Keep exactly what validation saw, including a changed directory's current hash.
          // A failed or undecodable snapshot has no observation, so recover it once here.
          if matches!(scope, SnapshotScope::CONTEXT) {
            if observed.is_none() {
              observed =
                Self::calc_strategy(&options, &helper, &path, scope, codec.is_portable()).await;
            }
          } else {
            observed = None;
          }
          Ok::<_, rspack_error::Error>((path, validate, observed, None))
        }
      })
      .try_fut_consume(|(path, validate, observed, dropped)| {
        if let Some(key) = dropped {
          dropped_keys.push(key);
        }
        if let Some(strategy) = observed {
          observed_path.insert(path.clone(), strategy);
        }
        match validate {
          ValidateResult::Modified => {
            modified_path.insert(path);
          }
          ValidateResult::Deleted => {
            deleted_path.insert(path);
          }
          ValidateResult::NoChangedRefresh(strategy) => {
            observed_path.insert(path.clone(), strategy);
            no_change_path.insert(path);
          }
          ValidateResult::NoChanged => {
            no_change_path.insert(path);
          }
        }
      })
      .await?;

    Ok((
      is_hot_start,
      modified_path,
      deleted_path,
      no_change_path,
      observed_path,
      dropped_keys,
    ))
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_fs::{MemoryFileSystem, WritableFileSystem};
  use rspack_paths::InternedPath;

  use super::{super::storage::MemoryStorage, Snapshot, SnapshotScope};
  use crate::{PathMatcher, SnapshotOptions, SnapshotStrategyOptions, cache::CacheCodec};

  macro_rules! p {
    ($tt:tt) => {
      InternedPath::from(std::path::Path::new($tt))
    };
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
  async fn should_snapshot_work() {
    let fs = Arc::new(MemoryFileSystem::default());
    let mut storage = MemoryStorage::default();
    let codec = Arc::new(CacheCodec::new(None));
    let options = SnapshotOptions {
      immutable_paths: vec![PathMatcher::String("/constant".into())],
      unmanaged_paths: vec![PathMatcher::String("/node_modules/project".into())],
      managed_paths: vec![PathMatcher::String("/node_modules".into())],
      resolve_build_dependencies: SnapshotStrategyOptions::hash_and_timestamp(),
      build_dependencies: SnapshotStrategyOptions::hash_and_timestamp(),
      resolve: SnapshotStrategyOptions::hash_and_timestamp(),
      module: SnapshotStrategyOptions::hash_and_timestamp(),
      context_module: SnapshotStrategyOptions::timestamp(),
    };

    fs.create_dir_all("/node_modules/project".into())
      .await
      .unwrap();
    fs.create_dir_all("/node_modules/lib".into()).await.unwrap();
    fs.write("/file1".into(), "abc".as_bytes()).await.unwrap();
    fs.write("/constant".into(), "abc".as_bytes())
      .await
      .unwrap();
    fs.write(
      "/node_modules/project/package.json".into(),
      r#"{"name":"project","version":"1.0.0"}"#.as_bytes(),
    )
    .await
    .unwrap();
    fs.write("/node_modules/project/file1".into(), "abc".as_bytes())
      .await
      .unwrap();
    fs.write(
      "/node_modules/lib/package.json".into(),
      r#"{"name":"lib","version":"1.1.0"}"#.as_bytes(),
    )
    .await
    .unwrap();
    fs.write("/node_modules/lib/file1".into(), "abc".as_bytes())
      .await
      .unwrap();

    let snapshot = Snapshot::new(options, "/".into(), fs.clone(), codec);

    snapshot
      .add(
        &mut storage,
        SnapshotScope::FILE,
        [
          p!("/file1"),
          p!("/constant"),
          p!("/node_modules/project/file1"),
          p!("/node_modules/lib/file1"),
        ]
        .into_iter(),
      )
      .await;
    std::thread::sleep(std::time::Duration::from_millis(100));
    fs.write("/file1".into(), "abcd".as_bytes()).await.unwrap();
    fs.write("/constant".into(), "abcd".as_bytes())
      .await
      .unwrap();
    fs.write("/node_modules/project/file1".into(), "abcd".as_bytes())
      .await
      .unwrap();
    fs.write("/node_modules/lib/file1".into(), "abcd".as_bytes())
      .await
      .unwrap();

    let (is_hot_start, modified_paths, deleted_paths, no_change_paths, _, _) = snapshot
      .calc_modified_paths(&storage, SnapshotScope::FILE)
      .await
      .unwrap();
    assert!(is_hot_start);
    assert!(deleted_paths.is_empty());
    assert!(!modified_paths.contains(&p!("/constant")));
    assert!(modified_paths.contains(&p!("/file1")));
    assert!(modified_paths.contains(&p!("/node_modules/project/file1")));
    assert!(!modified_paths.contains(&p!("/node_modules/lib/file1")));
    assert_eq!(no_change_paths.len(), 1);

    fs.write(
      "/node_modules/lib/package.json".into(),
      r#"{"name":"lib","version":"1.3.0"}"#.as_bytes(),
    )
    .await
    .unwrap();
    snapshot
      .add(
        &mut storage,
        SnapshotScope::FILE,
        [p!("/file1")].into_iter(),
      )
      .await;
    let (is_hot_start, modified_paths, deleted_paths, no_change_paths, _, _) = snapshot
      .calc_modified_paths(&storage, SnapshotScope::FILE)
      .await
      .unwrap();
    assert!(is_hot_start);
    assert!(deleted_paths.is_empty());
    assert!(!modified_paths.contains(&p!("/constant")));
    assert!(!modified_paths.contains(&p!("/file1")));
    assert!(modified_paths.contains(&p!("/node_modules/project/file1")));
    assert!(modified_paths.contains(&p!("/node_modules/lib/file1")));
    assert_eq!(no_change_paths.len(), 1);
  }
}
