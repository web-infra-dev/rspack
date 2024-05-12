use swc_core::common::Mark;
use swc_core::ecma::transforms::base::hygiene::{hygiene_with_config, Config};
use swc_core::ecma::visit::Fold;

#[allow(deprecated)]
pub fn hygiene(keep_class_names: bool, top_level_mark: Mark) -> impl Fold {
  hygiene_with_config(Config {
    keep_class_names,
    safari_10: true,
    top_level_mark,
    ignore_eval: false,
  })
}
