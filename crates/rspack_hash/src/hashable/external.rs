use lightningcss::targets::Browsers;
use swc_config::types::{BoolOr, BoolOrDataConfig};

use crate::{RspackHash, RspackHasher, hash_by_json};

impl<T: RspackHash> RspackHash for BoolOr<T> {
  fn hash(&self, state: &mut RspackHasher) {
    match self {
      BoolOr::Bool(value) => {
        "bool".hash(state);
        value.hash(state);
      }
      BoolOr::Data(value) => {
        "data".hash(state);
        value.hash(state);
      }
    }
  }
}

impl<T: RspackHash> RspackHash for BoolOrDataConfig<T> {
  fn hash(&self, state: &mut RspackHasher) {
    if let Some(value) = self.inner() {
      value.hash(state);
    }
  }
}

impl RspackHash for Browsers {
  fn hash(&self, state: &mut RspackHasher) {
    hash_by_json(self, state);
  }
}
