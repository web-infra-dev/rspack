use std::sync::Arc;

use futures::Future;
use rspack_error::Result;

use crate::{
  cache::snapshot::{Snapshot, SnapshotManager},
  cache::storage,
  ModuleIdentifier, ResolveArgs, ResolveResult,
};

type Storage = dyn storage::Storage<(Snapshot, ResolveResult)>;

#[derive(Debug)]
pub struct ResolveModuleOccasion {
  storage: Option<Box<Storage>>,
  snapshot_manager: Arc<SnapshotManager>,
}

impl ResolveModuleOccasion {
  pub fn new(storage: Option<Box<Storage>>, snapshot_manager: Arc<SnapshotManager>) -> Self {
    Self {
      storage,
      snapshot_manager,
    }
  }

  pub async fn use_cache<'a, G, F>(
    &self,
    args: ResolveArgs<'a>,
    generator: G,
  ) -> Result<ResolveResult>
  where
    G: Fn(ResolveArgs<'a>) -> F,
    F: Future<Output = Result<ResolveResult>>,
  {
    let storage = match &self.storage {
      Some(s) => s,
      // no cache return directly
      None => return generator(args).await,
    };

    let id = ModuleIdentifier::from(format!(
      "{}|{}",
      args.importer.unwrap_or(""),
      args.specifier
    ));
    {
      // read
      if let Some((snapshot, data)) = storage.get(&id) {
        let valid = self
          .snapshot_manager
          .check_snapshot_valid(&snapshot)
          .await
          .unwrap_or(false);

        if valid {
          return Ok(data);
        }
      };
    }

    // run generator and save to cache
    let data = generator(args).await?;
    let mut paths = Vec::new();
    if let ResolveResult::Info(info) = &data {
      paths.push(info.path.as_path());
    }

    let snapshot = self
      .snapshot_manager
      .create_snapshot(&paths, |option| &option.resolve)
      .await?;
    storage.set(id, (snapshot, data.clone()));
    Ok(data)
  }
}
