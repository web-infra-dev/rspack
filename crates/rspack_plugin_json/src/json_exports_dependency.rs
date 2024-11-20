use json::JsonValue;
use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  AsContextDependency, AsModuleDependency, Compilation, Dependency, DependencyId,
  DependencyTemplate, ExportNameOrSpec, ExportSpec, ExportsOfExportsSpec, ExportsSpec, ModuleGraph,
  RuntimeSpec, TemplateContext, TemplateReplaceSource,
};
use rspack_util::{ext::DynHash, itoa};

#[cacheable]
#[derive(Debug, Clone)]
pub struct JsonExportsDependency {
  id: DependencyId,
  #[cacheable(with=AsPreset)]
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

#[cacheable_dyn]
impl Dependency for JsonExportsDependency {
  fn id(&self) -> &rspack_core::DependencyId {
    &self.id
  }

  fn get_exports(&self, _mg: &ModuleGraph) -> Option<ExportsSpec> {
    Some(ExportsSpec {
      exports: get_exports_from_data(&self.data).unwrap_or(ExportsOfExportsSpec::Null),
      ..Default::default()
    })
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
  }
}

impl AsModuleDependency for JsonExportsDependency {}
impl AsContextDependency for JsonExportsDependency {}

#[cacheable_dyn]
impl DependencyTemplate for JsonExportsDependency {
  fn apply(
    &self,
    _source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }

  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    self.data.to_string().dyn_hash(hasher);
  }
}

fn get_exports_from_data(data: &JsonValue) -> Option<ExportsOfExportsSpec> {
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
              name: itoa!(i).into(),
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
