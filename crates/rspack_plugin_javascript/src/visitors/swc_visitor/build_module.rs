use std::sync::Arc;

use swc_core::base::config::ModuleConfig;
use swc_core::common::FileName;
use swc_core::common::{comments::SingleThreadedComments, Mark, SourceMap};
use swc_core::ecma::ast::EsVersion;
use swc_core::ecma::transforms::base::feature::{
  enable_available_feature_from_es_version, FeatureFlag,
};
use swc_core::ecma::visit::Fold;

pub fn build_module<'a>(
  cm: &Arc<SourceMap>,
  unresolved_mark: Mark,
  module: Option<ModuleConfig>,
  comments: Option<&'a SingleThreadedComments>,
  target: Option<EsVersion>,
) -> impl Fold + 'a {
  let feature_flag = if let Some(version) = target {
    enable_available_feature_from_es_version(version)
  } else {
    FeatureFlag::empty()
  };
  ModuleConfig::build(
    cm.clone(),
    comments,
    Default::default(),
    Default::default(),
    &FileName::Custom("".to_string()),
    unresolved_mark,
    module,
    feature_flag,
  )
}
