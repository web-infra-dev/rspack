use napi::Either;
use rspack_regex::RspackRegex;
use rspack_util::asset_condition::{AssetCondition, AssetConditions};

pub type RawAssetCondition = Either<String, RspackRegex>;
pub type RawAssetConditions = Either<RawAssetCondition, Vec<RawAssetCondition>>;

struct RawAssetConditionWrapper(RawAssetCondition);
struct RawAssetConditionsWrapper(RawAssetConditions);

impl From<RawAssetConditionWrapper> for AssetCondition {
  fn from(x: RawAssetConditionWrapper) -> Self {
    match x.0 {
      Either::A(v) => Self::String(v),
      Either::B(v) => Self::Regexp(v),
    }
  }
}

impl From<RawAssetConditionsWrapper> for AssetConditions {
  fn from(value: RawAssetConditionsWrapper) -> Self {
    match value.0 {
      Either::A(v) => Self::Single(RawAssetConditionWrapper(v).into()),
      Either::B(v) => Self::Multiple(
        v.into_iter()
          .map(|v| RawAssetConditionWrapper(v).into())
          .collect(),
      ),
    }
  }
}

pub fn into_asset_condition(r: RawAssetCondition) -> AssetCondition {
  RawAssetConditionWrapper(r).into()
}

pub fn into_asset_conditions(r: RawAssetConditions) -> AssetConditions {
  RawAssetConditionsWrapper(r).into()
}
