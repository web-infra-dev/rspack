use json::JsonValue;
use rspack_core::{
  AsContextDependency, AsModuleDependency, ConnectionState, Dependency, DependencyId,
  DependencyTemplate, ExportNameOrSpec, ExportSpec, ExportsOfExportsSpec, ExportsSpec, ModuleGraph,
  ModuleIdentifier, TemplateContext, TemplateReplaceSource, UsageState, UsedByExports, UsedName,
};
#[derive(Debug, Clone)]
pub struct JsonExportsDependency {
  id: DependencyId,
  data: JsonValue,
}

impl JsonExportsDependency {
  pub fn new(data: JsonValue) -> Self {
    Self {
      data,
      id: DependencyId::new(),
    }
  }
}

impl Dependency for JsonExportsDependency {
  fn id(&self) -> &rspack_core::DependencyId {
    &self.id
  }

  fn dependency_debug_name(&self) -> &'static str {
    "JsonExportsDependency"
  }

  fn get_exports(&self, _mg: &ModuleGraph) -> Option<ExportsSpec> {
    Some(ExportsSpec {
      exports: get_exports_from_data(&self.data).unwrap_or(ExportsOfExportsSpec::Null),
      ..Default::default()
    })
  }
}

impl AsModuleDependency for JsonExportsDependency {}
impl AsContextDependency for JsonExportsDependency {}

impl DependencyTemplate for JsonExportsDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
  }
}

fn get_exports_from_data(data: &JsonValue) -> Option<ExportsOfExportsSpec> {
  let data = data;
  let ret = match data {
    JsonValue::Null
    | JsonValue::Short(_)
    | JsonValue::String(_)
    | JsonValue::Number(_)
    | JsonValue::Boolean(_) => {
      return None;
    }
    JsonValue::Object(obj) => ExportsOfExportsSpec::Array(
      obj
        .iter()
        .map(|(k, v)| {
          ExportNameOrSpec::ExportSpec(ExportSpec {
            name: k.into(),
            can_mangle: Some(true),
            exports: get_exports_from_data(v).map(|item| match item {
              ExportsOfExportsSpec::True => unreachable!(),
              ExportsOfExportsSpec::Null => unreachable!(),
              ExportsOfExportsSpec::Array(arr) => arr,
            }),
            ..Default::default()
          })
        })
        .collect::<Vec<_>>(),
    ),
    JsonValue::Array(arr) => {
      if arr.len() > 100 {
        return None;
      }
      ExportsOfExportsSpec::Array(
        arr
          .iter()
          .enumerate()
          .map(|(i, item)| {
            ExportNameOrSpec::ExportSpec(ExportSpec {
              name: format!("{i}").into(),
              can_mangle: Some(true),
              exports: get_exports_from_data(item).map(|item| match item {
                ExportsOfExportsSpec::True | ExportsOfExportsSpec::Null => unreachable!(),
                ExportsOfExportsSpec::Array(arr) => arr,
              }),
              ..Default::default()
            })
          })
          .collect::<Vec<_>>(),
      )
    }
  };
  Some(ret)
}

// const getExportsFromData = data => {
// 	if (data && typeof data === "object") {
// 		if (Array.isArray(data)) {
// 			return data.length < 100
// 				? data.map((item, idx) => {
// 						return {
// 							name: `${idx}`,
// 							canMangle: true,
// 							exports: getExportsFromData(item)
// 						};
// 				  })
// 				: undefined;
// 		} else {
// 			const exports = [];
// 			for (const key of Object.keys(data)) {
// 				exports.push({
// 					name: key,
// 					canMangle: true,
// 					exports: getExportsFromData(data[key])
// 				});
// 			}
// 			return exports;
// 		}
// 	}
// 	return undefined;
// };
