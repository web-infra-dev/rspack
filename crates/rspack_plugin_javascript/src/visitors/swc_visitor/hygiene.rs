use swc_ecma_transforms::hygiene::{hygiene_with_config, Config};
use swc_ecma_visit::Fold;

pub fn hygiene(keep_class_names: bool) -> impl Fold {
  hygiene_with_config(Config { keep_class_names })
}
