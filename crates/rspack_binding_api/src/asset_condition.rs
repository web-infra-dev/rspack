use napi::Either;
use rspack_regex::RspackRegex;
use rspack_util::asset_condition::{AssetCondition, AssetConditions};

use crate::js_regex::JsRegExp;

pub type RawAssetCondition = Either<String, JsRegExp>;
pub type RawAssetConditions = Either<RawAssetCondition, Vec<RawAssetCondition>>;

struct RawAssetConditionWrapper(RawAssetCondition);
struct RawAssetConditionsWrapper(RawAssetConditions);

impl TryFrom<RawAssetConditionWrapper> for AssetCondition {
  type Error = rspack_error::Error;

  fn try_from(x: RawAssetConditionWrapper) -> Result<Self, Self::Error> {
    match x.0 {
      Either::A(v) => Ok(Self::String(v)),
      Either::B(v) => Ok(Self::Regexp(v.try_into()?)),
    }
  }
}

impl TryFrom<RawAssetConditionsWrapper> for AssetConditions {
  type Error = rspack_error::Error;

  fn try_from(value: RawAssetConditionsWrapper) -> Result<Self, Self::Error> {
    match value.0 {
      Either::A(v) => Ok(Self::Single(RawAssetConditionWrapper(v).try_into()?)),
      Either::B(v) => Ok(Self::Multiple(
        v.into_iter()
          .map(|v| RawAssetConditionWrapper(v).try_into())
          .collect::<Result<_, _>>()?,
      )),
    }
  }
}

pub fn try_into_asset_conditions(r: RawAssetConditions) -> rspack_error::Result<AssetConditions> {
  RawAssetConditionsWrapper(r).try_into()
}
