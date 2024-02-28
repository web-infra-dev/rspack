use swc_core::atoms::Atom;

use crate::visitors::TagInfoData;

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug, Hash)]
pub struct TopLevelSymbol(pub(super) Atom);

impl TagInfoData for TopLevelSymbol {
  fn serialize(data: &Self) -> serde_json::Value {
    serde_json::to_value(data).expect("serialize failed for `TopLevelSymbol`")
  }

  fn deserialize(value: serde_json::Value) -> Self {
    serde_json::from_value(value).expect("deserialize failed for `TopLevelSymbol`")
  }
}
