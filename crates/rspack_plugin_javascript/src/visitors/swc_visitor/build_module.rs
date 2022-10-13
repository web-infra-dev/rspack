use std::sync::Arc;
use swc::config::ModuleConfig;
use swc_common::{comments::SingleThreadedComments, Mark, SourceMap};
use swc_ecma_ast::EsVersion;
use swc_ecma_transforms::feature::{enable_available_feature_from_es_version, FeatureFlag};
use swc_ecma_visit::Fold;

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
    &swc_common::FileName::Custom("".to_string()),
    unresolved_mark,
    module,
    feature_flag,
  )
}
