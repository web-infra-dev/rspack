use rspack_collections::Identifier;
use rspack_util::{
  asset_condition::{AssetCondition, AssetConditions},
  atom::Atom,
};

use crate::{RspackHash, RspackHashable};

impl RspackHashable for Atom {
  fn hash(&self, state: &mut RspackHash) {
    self.as_str().hash(state);
  }
}

impl RspackHashable for Identifier {
  fn hash(&self, state: &mut RspackHash) {
    self.as_str().hash(state);
  }
}

impl RspackHashable for AssetCondition {
  fn hash(&self, state: &mut RspackHash) {
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

impl RspackHashable for AssetConditions {
  fn hash(&self, state: &mut RspackHash) {
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
