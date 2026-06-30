use lightningcss::targets::Browsers;
use swc_config::types::{BoolOr, BoolOrDataConfig};

use crate::{RspackHash, RspackHashable, hash_by_json};

impl<T: RspackHashable> RspackHashable for BoolOr<T> {
  fn hash(&self, state: &mut RspackHash) {
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

impl<T: RspackHashable> RspackHashable for BoolOrDataConfig<T> {
  fn hash(&self, state: &mut RspackHash) {
    if let Some(value) = self.inner() {
      value.hash(state);
    }
  }
}

impl RspackHashable for Browsers {
  fn hash(&self, state: &mut RspackHash) {
    hash_by_json(self, state);
  }
}
