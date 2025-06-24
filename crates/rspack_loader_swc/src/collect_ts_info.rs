use rspack_core::{CollectedTypeScriptInfo, EvaluatedInlinableValue};
use rspack_swc_plugin_ts_collector::{
  EnumMemberValue, TopLevelEnumCollector, TypeExportsCollector,
};
use rustc_hash::FxHashMap;
use swc::atoms::Atom;
use swc_core::ecma::{ast::Program, utils::number::ToJsString, visit::VisitWith};

use crate::options::{CollectTypeScriptInfoOptions, CrossModuleEnumKind};

pub fn collect_typescript_info(
  program: &Program,
  options: &CollectTypeScriptInfoOptions,
) -> CollectedTypeScriptInfo {
  let mut type_exports = Default::default();
  if options.type_exports.unwrap_or_default() {
    program.visit_with(&mut TypeExportsCollector::new(&mut type_exports));
  }
  let mut top_level_enums: FxHashMap<
    (Atom, Atom),
    rspack_swc_plugin_ts_collector::EnumMemberValue,
  > = Default::default();
  if let Some(kind) = &options.cross_module_enums {
    program.visit_with(&mut TopLevelEnumCollector::new(
      matches!(kind, CrossModuleEnumKind::ConstOnly),
      &mut top_level_enums,
    ));
  }
  CollectedTypeScriptInfo {
    type_exports,
    top_level_enums: top_level_enums
      .into_iter()
      .map(|(k, v)| {
        let value = match v {
          EnumMemberValue::Number(n) => {
            EvaluatedInlinableValue::new_number(n.to_js_string().into())
          }
          EnumMemberValue::String(s) => EvaluatedInlinableValue::new_string(s),
        };
        (k, value)
      })
      .collect(),
  }
}
