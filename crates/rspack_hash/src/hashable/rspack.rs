use rspack_collections::Identifier;
use rspack_util::{
  asset_condition::{AssetCondition, AssetConditions},
  atom::Atom,
};

use crate::{HashAlgorithm, RspackHash};

impl RspackHash for Atom {
  fn hash(&self, state: &mut HashAlgorithm) {
    self.as_str().hash(state);
  }
}

impl RspackHash for Identifier {
  fn hash(&self, state: &mut HashAlgorithm) {
    self.as_str().hash(state);
  }
}

impl RspackHash for AssetCondition {
  fn hash(&self, state: &mut HashAlgorithm) {
    match self {
      AssetCondition::String(value) => {
        "string".hash(state);
        value.hash(state);
      }
      AssetCondition::Regexp(value) => {
        "regexp".hash(state);
        value.source.hash(state);
        value.flags.hash(state);
      }
    }
  }
}

impl RspackHash for AssetConditions {
  fn hash(&self, state: &mut HashAlgorithm) {
    match self {
      AssetConditions::Single(value) => {
        "single".hash(state);
        value.hash(state);
      }
      AssetConditions::Multiple(value) => {
        "multiple".hash(state);
        value.hash(state);
      }
    }
  }
}
