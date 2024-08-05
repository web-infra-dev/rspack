use napi::Either;
use rspack_napi::regexp::{JsRegExp, JsRegExpExt};
use rspack_util::asset_rule::{AssetRule, AssetRules};

pub type RawAssetRule = Either<String, JsRegExp>;
pub type RawAssetRules = Either<RawAssetRule, Vec<RawAssetRule>>;

struct RawAssetRuleWrapper(RawAssetRule);
struct RawAssetRulesWrapper(RawAssetRules);

impl From<RawAssetRuleWrapper> for AssetRule {
  fn from(x: RawAssetRuleWrapper) -> Self {
    match x.0 {
      Either::A(v) => Self::String(v),
      Either::B(v) => Self::Regexp(v.to_rspack_regex()),
    }
  }
}

impl From<RawAssetRulesWrapper> for AssetRules {
  fn from(value: RawAssetRulesWrapper) -> Self {
    match value.0 {
      Either::A(v) => Self::Single(RawAssetRuleWrapper(v).into()),
      Either::B(v) => Self::Multiple(
        v.into_iter()
          .map(|v| RawAssetRuleWrapper(v).into())
          .collect(),
      ),
    }
  }
}

pub fn into_asset_rule(r: RawAssetRule) -> AssetRule {
  RawAssetRuleWrapper(r).into()
}

pub fn into_asset_rules(r: RawAssetRules) -> AssetRules {
  RawAssetRulesWrapper(r).into()
}
