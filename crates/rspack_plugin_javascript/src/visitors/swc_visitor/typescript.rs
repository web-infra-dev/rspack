use std::sync::Arc;

use swc_core::common::{comments::Comments, Mark, SourceMap};
use swc_core::ecma::transforms::{
  react::{default_pragma, default_pragma_frag},
  typescript,
};
use swc_core::ecma::visit::Fold;

pub fn typescript<'a>(
  top_level_mark: Mark,
  comments: Option<&'a dyn Comments>,
  cm: &Arc<SourceMap>,
) -> impl Fold + 'a {
  /*  let import_export_assign_config = match module {
    Some(ModuleConfig::Es6) => TsImportExportAssignConfig::EsNext,
    Some(ModuleConfig::CommonJs(..))
    | Some(ModuleConfig::Amd(..))
    | Some(ModuleConfig::Umd(..)) => TsImportExportAssignConfig::Preserve,
    Some(ModuleConfig::NodeNext) => TsImportExportAssignConfig::NodeNext,
    // TODO: should Preserve for SystemJS
    _ => TsImportExportAssignConfig::Classic,
  };*/

  typescript::tsx(
    cm.clone(),
    typescript::Config::default(),
    typescript::TsxConfig {
      pragma: Some(default_pragma()),
      pragma_frag: Some(default_pragma_frag()),
    },
    comments,
    top_level_mark,
  )
}
