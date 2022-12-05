use rspack_core::Define;
use std::sync::Arc;
use swc_core::base::config::GlobalPassOption;
use swc_core::common::{errors::Handler, SourceMap};
use swc_core::ecma::atoms::JsWord;
use swc_core::ecma::visit::Fold;

pub fn define(opts: &Define, handler: &Handler, cm: &Arc<SourceMap>) -> impl Fold {
  let mut global_opts: GlobalPassOption = Default::default();
  for (key, value) in opts {
    global_opts
      .vars
      .insert(JsWord::from(key.as_str()), JsWord::from(value.as_str()));
  }
  global_opts.build(cm, handler)
}
