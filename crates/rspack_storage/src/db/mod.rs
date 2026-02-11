mod transaction;

use std::path::PathBuf;

struct KVDB {
  root: PathBuf,
}

impl KVDB {
  fn new(root: PathBuf) -> Self {
    Self { root }
  }

  fn bucket() {}

  fn buckets() {}

  fn transaction() {}

  //    fn
}
