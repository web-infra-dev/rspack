use rspack_core::{CollectedTypeScriptInfo, EvaluatedInlinableValue, TSEnumValue};
use rspack_swc_plugin_ts_collector::{
  EnumMemberValue, ExportedEnumCollector, TypeExportsCollector,
};
use rustc_hash::FxHashMap;
use swc::atoms::Atom;
use swc_core::ecma::{ast::Program, utils::number::ToJsString, visit::VisitWith};

use crate::options::{CollectTypeScriptInfoOptions, CollectingEnumKind};

pub fn collect_typescript_info(
  program: &Program,
  options: &CollectTypeScriptInfoOptions,
) -> CollectedTypeScriptInfo {
  let mut type_exports = Default::default();
  if options.type_exports.unwrap_or_default() {
    program.visit_with(&mut TypeExportsCollector::new(&mut type_exports));
  }
  let mut exported_enums: FxHashMap<Atom, FxHashMap<Atom, EnumMemberValue>> = Default::default();
  if let Some(kind) = &options.exported_enum {
    program.visit_with(&mut ExportedEnumCollector::new(
      matches!(kind, CollectingEnumKind::ConstOnly),
      &mut exported_enums,
    ));
  }
  CollectedTypeScriptInfo {
    type_exports,
    exported_enums: exported_enums
      .into_iter()
      .map(|(k, members)| {
        let value = TSEnumValue::new(
          members
            .into_iter()
            .map(|(id, v)| {
              let value = match v {
                EnumMemberValue::Number(n) => {
                  Some(EvaluatedInlinableValue::new_number(n.to_js_string().into()))
                }
                EnumMemberValue::String(s) => Some(EvaluatedInlinableValue::new_string(s)),
                EnumMemberValue::Unknown => None,
              };
              (id, value)
            })
            .collect(),
        );
        (k, value)
      })
      .collect(),
  }
}
