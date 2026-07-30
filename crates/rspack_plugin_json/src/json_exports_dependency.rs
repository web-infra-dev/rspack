use json::JsonValue;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsModuleDependency, Compilation, Dependency, DependencyCodeGeneration,
  DependencyId, ExportNameOrSpec, ExportSpec, ExportsInfoArtifact, ExportsOfExportsSpec,
  ExportsSpec, ModuleGraph, ModuleGraphCacheArtifact, RuntimeSpec,
};
use rspack_hash::{RspackHash, RspackHasher};
use rspack_util::itoa;

#[cacheable]
#[derive(Debug, Clone)]
pub struct JsonExportsDependency {
  id: DependencyId,
  exports_depth: u32,
}

impl JsonExportsDependency {
  pub fn new(exports_depth: u32) -> Self {
    Self {
      id: DependencyId::new(),
      exports_depth,
    }
  }

  fn data<'a>(&self, module_graph: &'a ModuleGraph) -> &'a JsonValue {
    module_graph
      .get_parent_module(&self.id)
      .and_then(|identifier| module_graph.module_by_identifier(identifier))
      .and_then(|module| module.build_info().json_data.as_ref())
      .expect("JSON export dependency should have parent JSON module data")
  }
}

#[cacheable_dyn]
impl Dependency for JsonExportsDependency {
  fn id(&self) -> &rspack_core::DependencyId {
    &self.id
  }

  fn get_exports(
    &self,
    module_graph: &ModuleGraph,
    _mg_cache: &ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
  ) -> Option<ExportsSpec> {
    Some(ExportsSpec {
      exports: get_exports_from_data(self.data(module_graph), self.exports_depth, 1)
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
    hasher: &mut RspackHasher,
    compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    let module_graph = compilation.get_module_graph();
    self.data(module_graph).to_string().hash(hasher);
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
          exports: get_exports_from_data(v, exports_depth, cur_depth + 1),
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
            exports: get_exports_from_data(item, exports_depth, cur_depth + 1),
            ..Default::default()
          })
        })
        .collect::<Vec<_>>()
    }
  };
  Some(ret)
}

#[cfg(test)]
mod tests {
  use json::JsonValue;
  use rspack_cacheable::{cacheable, to_bytes, with::AsPreset};
  use rspack_core::{
    DependencyId, DependencyParents, ExportNameOrSpec, Module, ModuleGraph, ModuleIdentifier,
    RawModule, RuntimeGlobals,
  };
  use rspack_hash::{HashFunction, RspackHash, RspackHasher};

  use super::{JsonExportsDependency, get_exports_from_data};

  #[cacheable]
  #[derive(Debug)]
  struct LegacyJsonExportsDependency {
    id: DependencyId,
    #[cacheable(with=AsPreset)]
    data: JsonValue,
    exports_depth: u32,
  }

  fn export_names(exports: &[ExportNameOrSpec]) -> Vec<&str> {
    exports
      .iter()
      .map(|export| match export {
        ExportNameOrSpec::String(name) => name.as_str(),
        ExportNameOrSpec::ExportSpec(spec) => spec.name.as_str(),
      })
      .collect()
  }

  #[test]
  fn preserves_named_json_exports_at_each_configured_depth() {
    let data = json::parse(r#"{"named":{"nested":1},"other":true}"#).unwrap();
    assert!(get_exports_from_data(&data, 0, 1).is_none());

    let depth_one = get_exports_from_data(&data, 1, 1).unwrap();
    assert_eq!(export_names(&depth_one), ["named", "other"]);
    let ExportNameOrSpec::ExportSpec(named) = &depth_one[0] else {
      panic!("named JSON export should include its export specification");
    };
    assert!(named.exports.is_none());

    let depth_two = get_exports_from_data(&data, 2, 1).unwrap();
    let ExportNameOrSpec::ExportSpec(named) = &depth_two[0] else {
      panic!("named JSON export should include its export specification");
    };
    assert_eq!(export_names(named.exports.as_ref().unwrap()), ["nested"]);
  }

  #[test]
  fn preserves_array_exports_and_primitive_default_only_behavior() {
    let array = json::parse(r#"[{"nested":true},4]"#).unwrap();
    assert_eq!(
      export_names(&get_exports_from_data(&array, 2, 1).unwrap()),
      ["0", "1"]
    );
    for source in ["null", "true", "7", r#""value""#] {
      let primitive = json::parse(source).unwrap();
      assert!(get_exports_from_data(&primitive, 2, 1).is_none());
    }
  }

  #[test]
  fn reads_custom_parser_data_from_the_attached_parent_module() {
    let dependency = JsonExportsDependency::new(2);
    let parsed_data = json::parse(r#"{"custom":{"nested":true}}"#).unwrap();
    let identifier: ModuleIdentifier = "synthetic.json".into();
    let mut parent = RawModule::new(
      "original source".to_string(),
      identifier,
      "synthetic.json".to_string(),
      RuntimeGlobals::empty(),
    );
    parent.build_info_mut().json_data = Some(parsed_data.clone());

    let mut module_graph = ModuleGraph::default();
    module_graph.set_parents(
      dependency.id,
      DependencyParents {
        module: identifier,
        ..Default::default()
      },
    );
    module_graph.add_module(Box::new(parent));

    assert_eq!(dependency.data(&module_graph), &parsed_data);

    let mut legacy_hash = RspackHasher::new(&HashFunction::Xxhash64);
    parsed_data.to_string().hash(&mut legacy_hash);
    let mut canonical_hash = RspackHasher::new(&HashFunction::Xxhash64);
    dependency
      .data(&module_graph)
      .to_string()
      .hash(&mut canonical_hash);
    assert_eq!(legacy_hash.finish(), canonical_hash.finish());
  }

  #[test]
  #[should_panic(expected = "JSON export dependency should have parent JSON module data")]
  fn requires_an_attached_parent_before_reading_module_data() {
    JsonExportsDependency::new(2).data(&ModuleGraph::default());
  }

  #[test]
  fn archives_only_json_export_metadata_instead_of_cloning_the_json_value() {
    let dependency = JsonExportsDependency::new(2);
    let legacy = LegacyJsonExportsDependency {
      id: dependency.id,
      data: json::object! { payload: "x".repeat(16 * 1024) },
      exports_depth: 2,
    };

    let legacy_bytes = to_bytes(&legacy, &()).unwrap();
    let metadata_bytes = to_bytes(&dependency, &()).unwrap();
    eprintln!(
      "synthetic JSON exports dependency archive: previous={} bytes, single-source={} bytes",
      legacy_bytes.len(),
      metadata_bytes.len()
    );
    assert!(legacy_bytes.len() > 16 * 1024);
    assert!(metadata_bytes.len() < 128);
    assert!(legacy_bytes.len() > metadata_bytes.len() * 100);
  }
}
