mod cacheable;
mod occasion;
mod snapshot;
mod storage;

use std::{path::PathBuf, sync::Arc};

use occasion::MakeOccasion;

use self::cacheable::CacheContext;
pub use self::cacheable::FromContext;
pub use self::snapshot::SnapshotOption;
use self::{
  snapshot::Snapshot,
  storage::{ArcStorage, FsStorage},
};
use crate::CompilerOptions;

// TODO call write storage only build success
#[derive(Debug)]
pub struct Cache {
  pub make_occasion: MakeOccasion,
  pub storage: ArcStorage,
  pub snapshot: Snapshot,
}

// TODO conside multi compiler
impl Cache {
  pub fn new(compiler_option: Arc<CompilerOptions>) -> Self {
    let context = Arc::new(CacheContext {
      options: compiler_option.clone(),
    });
    let storage = Arc::new(FsStorage::new(
      PathBuf::from(compiler_option.context.as_str())
        .join("node_modules/.cache/rspack/compiler-id-version"),
    ));
    Self {
      make_occasion: MakeOccasion::new(storage.clone(), context),
      snapshot: Snapshot::new(storage.clone(), Default::default()),
      storage,
    }
  }

  pub fn idle(&self) {
    self.storage.idle();
  }
}
