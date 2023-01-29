use swc_core::ecma::transforms::base::hygiene::{hygiene_with_config, Config};
use swc_core::ecma::visit::Fold;

pub fn hygiene(keep_class_names: bool) -> impl Fold {
  hygiene_with_config(Config {
    keep_class_names,
    safari_10: true,
    top_level_mark: Default::default(),
  })
}
