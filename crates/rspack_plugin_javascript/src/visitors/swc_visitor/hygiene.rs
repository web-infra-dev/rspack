use swc_core::{
  common::Mark,
  ecma::{
    ast::Pass,
    transforms::base::hygiene::{hygiene_with_config, Config},
    visit::VisitMut,
  },
};
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
