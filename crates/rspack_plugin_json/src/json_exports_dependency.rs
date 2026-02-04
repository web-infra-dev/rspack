use json::JsonValue;
use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  AsContextDependency, AsModuleDependency, Compilation, Dependency, DependencyCodeGeneration,
  DependencyId, ExportNameOrSpec, ExportSpec, ExportSpecExports, ExportsOfExportsSpec, ExportsSpec,
  ModuleGraph, ModuleGraphCacheArtifact, RuntimeSpec,
};
use rspack_util::{ext::DynHash, itoa};

#[cacheable]
#[derive(Debug, Clone)]
pub struct JsonExportsDependency {
  id: DependencyId,
  #[cacheable(with=AsPreset)]
  data: JsonValue,
  exports_depth: u32,
}

impl JsonExportsDependency {
  pub fn new(data: JsonValue, exports_depth: u32) -> Self {
    Self {
      data,
      id: DependencyId::new(),
      exports_depth,
    }
  }
}

#[cacheable_dyn]
impl Dependency for JsonExportsDependency {
  fn id(&self) -> &rspack_core::DependencyId {
    &self.id
  }

  fn get_exports(
    &self,
    _mg: &ModuleGraph,
    _mg_cache: &ModuleGraphCacheArtifact,
  ) -> Option<ExportsSpec> {
    Some(ExportsSpec {
      exports: get_exports_from_data(&self.data, self.exports_depth, 1)
        .map_or(ExportsOfExportsSpec::NoExports, ExportsOfExportsSpec::Names),
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
impl DependencyCodeGeneration for JsonExportsDependency {
  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    self.data.to_string().dyn_hash(hasher);
  }
}

fn get_exports_from_data(
  data: &JsonValue,
  exports_depth: u32,
  cur_depth: u32,
) -> Option<Vec<ExportNameOrSpec>> {
  if cur_depth > exports_depth {
    return None;
  }
  let ret = match data {
    JsonValue::Null
    | JsonValue::Short(_)
    | JsonValue::String(_)
    | JsonValue::Number(_)
    | JsonValue::Boolean(_) => {
      return None;
    }
    JsonValue::Object(obj) => obj
      .iter()
      .map(|(k, v)| {
        ExportNameOrSpec::ExportSpec(ExportSpec {
          name: k.into(),
          can_mangle: Some(true),
          exports: get_exports_from_data(v, exports_depth, cur_depth + 1)
            .map(ExportSpecExports::new),
          ..Default::default()
        })
      })
      .collect::<Vec<_>>(),
    JsonValue::Array(arr) => {
      if arr.len() > 100 {
        return None;
      }
      arr
        .iter()
        .enumerate()
        .map(|(i, item)| {
          let mut i_buffer = itoa::Buffer::new();
          let i_str = i_buffer.format(i);
          ExportNameOrSpec::ExportSpec(ExportSpec {
            name: i_str.into(),
            can_mangle: Some(true),
            exports: get_exports_from_data(item, exports_depth, cur_depth + 1)
              .map(ExportSpecExports::new),
            ..Default::default()
          })
        })
        .collect::<Vec<_>>()
    }
  };
  Some(ret)
}
