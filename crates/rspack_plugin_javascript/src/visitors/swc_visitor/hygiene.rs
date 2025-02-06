use swc_core::common::Mark;
use swc_core::ecma::ast::Pass;
use swc_core::ecma::transforms::base::hygiene::{hygiene_with_config, Config};
use swc_core::ecma::visit::VisitMut;
#[allow(deprecated)]
pub fn hygiene(keep_class_names: bool, top_level_mark: Mark) -> impl 'static + Pass + VisitMut {
  hygiene_with_config(Config {
    keep_class_names,
    top_level_mark,
    ignore_eval: false,
    // FIXME: support user passing preserved_symbols in the future
    preserved_symbols: Default::default(),
  })
}
