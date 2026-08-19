use std::sync::{Arc, LazyLock};

use anymap::CloneAny;
use rspack_cacheable::{
  cacheable,
  with::{AsCacheable, AsMap, AsOption, AsPreset, AsVec},
};
use rspack_collections::IdentifierIndexMap;
use rspack_hash::RspackHashDigest;
use rspack_util::{
  fx_hash::{FxIndexMap, FxIndexSet},
  itoa,
};
use rustc_hash::FxHashMap as HashMap;
use swc_core::atoms::Atom;

use crate::{
  ExportMode, ModuleIdentifier,
  concatenated_module::{ConcatenatedImportMap, ConcatenatedModuleInfo, ModuleInfo},
};

pub static DEFAULT_EXPORT_ATOM: LazyLock<Atom> = LazyLock::new(|| "__rspack_default_export".into());
pub const NAMESPACE_OBJECT_EXPORT: &str = "__rspack_ns_object";
pub const DEFAULT_EXPORT: &str = "__rspack_default_export";
const MODULE_REFERENCE_PREFIX: &str = "__rspack_module_ref";
const MODULE_REFERENCE_PROPERTY_ACCESS_SUFFIX: &str = "._";

#[cacheable]
#[derive(Default, Debug, Clone)]
pub struct ModuleReferenceOptions {
  #[cacheable(with=AsVec<AsPreset>)]
  pub ids: Vec<Atom>,
  pub call: bool,
  pub direct_import: bool,
  pub deferred_import: bool,
  pub asi_safe: Option<bool>,
  pub index: usize,
}

/// Module-local data produced while generating code in a concatenation scope.
///
/// The scope also contains compilation-local inputs such as `modules_map` and
/// type-erased scratch data. Only this output is needed after code generation
/// and is safe to persist with the generated source.
#[cacheable]
#[derive(Debug, Default, Clone)]
pub struct ConcatenationCodeGenerationData {
  #[cacheable(with=AsOption<AsPreset>)]
  pub namespace_export_symbol: Option<Atom>,
  #[cacheable(with=AsOption<AsMap<AsPreset>>)]
  pub export_map: Option<HashMap<Atom, String>>,
  #[cacheable(with=AsOption<AsMap<AsPreset>>)]
  pub raw_export_map: Option<HashMap<Atom, String>>,
  #[cacheable(with=AsOption<AsMap>)]
  pub import_map: ConcatenatedImportMap,
  #[cacheable(with=AsMap<AsCacheable, AsMap>)]
  pub refs: IdentifierIndexMap<FxIndexMap<String, ModuleReferenceOptions>>,
}

impl ConcatenationCodeGenerationData {
  pub fn apply_to(&self, info: &mut ConcatenatedModuleInfo) {
    info.namespace_export_symbol = self.namespace_export_symbol.clone();
    info.export_map = self.export_map.clone();
    info.raw_export_map = self.raw_export_map.clone();
    info.import_map = self.import_map.clone();
  }

  pub fn into_module_info(self, mut info: ConcatenatedModuleInfo) -> ConcatenatedModuleInfo {
    info.namespace_export_symbol = self.namespace_export_symbol;
    info.export_map = self.export_map;
    info.raw_export_map = self.raw_export_map;
    info.import_map = self.import_map;
    info
  }
}

#[derive(Debug, Clone)]
pub struct ConcatenationScope {
  pub concat_module_id: ModuleIdentifier,
  pub current_module: ConcatenatedModuleInfo,
  pub modules_map: Arc<IdentifierIndexMap<ModuleInfo>>,
  /// A stable fingerprint of the compilation-local inputs that can affect code generation.
  ///
  /// Scope providers may set this when their scope contains inputs that are not already covered by
  /// the module runtime hash. The fingerprint is persisted alongside module hashes, while the scope
  /// itself remains compilation-local.
  pub code_generation_hash: Option<RspackHashDigest>,
  pub data: anymap::Map<dyn CloneAny + Send + Sync>,
  pub refs: IdentifierIndexMap<FxIndexMap<String, ModuleReferenceOptions>>,
  pub dyn_refs: IdentifierIndexMap<FxIndexSet<(String, Atom)>>,
  pub re_exports: IdentifierIndexMap<Vec<ExportMode>>,
}

#[allow(unused)]
impl ConcatenationScope {
  pub fn new(
    concat_module_id: ModuleIdentifier,
    modules_map: Arc<IdentifierIndexMap<ModuleInfo>>,
    current_module: ConcatenatedModuleInfo,
  ) -> Self {
    ConcatenationScope {
      concat_module_id,
      current_module,
      modules_map,
      code_generation_hash: None,
      data: Default::default(),
      refs: IdentifierIndexMap::default(),
      dyn_refs: Default::default(),
      re_exports: Default::default(),
    }
  }

  pub fn is_module_in_scope(&self, module: &ModuleIdentifier) -> bool {
    self.modules_map.contains_key(module)
  }

  pub fn with_code_generation_hash(mut self, hash: RspackHashDigest) -> Self {
    self.code_generation_hash = Some(hash);
    self
  }

  pub fn into_code_generation_data(self) -> ConcatenationCodeGenerationData {
    ConcatenationCodeGenerationData {
      namespace_export_symbol: self.current_module.namespace_export_symbol,
      export_map: self.current_module.export_map,
      raw_export_map: self.current_module.raw_export_map,
      import_map: self.current_module.import_map,
      refs: self.refs,
    }
  }

  /**
  export { symbol as export_name }
  */
  pub fn register_export(&mut self, export_name: Atom, symbol: String) {
    let export_map = self.current_module.export_map.get_or_insert_default();
    export_map.insert(export_name, symbol);
  }

  pub fn register_raw_export(&mut self, export_name: Atom, symbol: String) {
    let raw_export_map = self.current_module.raw_export_map.get_or_insert_default();
    raw_export_map.insert(export_name, symbol);
  }

  pub fn register_namespace_import(
    &mut self,
    import_source: String,
    attributes: Option<String>,
    import_symbol: Atom,
  ) -> &Atom {
    let raw_import_map = self.current_module.import_map.get_or_insert_default();
    let entry = raw_import_map
      .entry((import_source, attributes))
      .or_default();

    if entry.namespace.is_none() {
      entry.namespace = Some(import_symbol)
    }

    entry
      .namespace
      .as_ref()
      .expect("should have namespace symbol")
  }

  pub fn register_import(
    &mut self,
    import_source: String,
    attributes: Option<String>,
    import_symbol: Option<Atom>,
  ) {
    let raw_import_map = self.current_module.import_map.get_or_insert_default();
    let entry = raw_import_map
      .entry((import_source, attributes))
      .or_default();

    let Some(import_symbol) = import_symbol else {
      return;
    };

    entry.specifiers.insert(import_symbol);
  }

  pub fn register_namespace_export(&mut self, symbol: &str) {
    self.current_module.namespace_export_symbol = Some(symbol.into());
  }

  pub fn create_module_reference(
    &mut self,
    module: &ModuleIdentifier,
    options: ModuleReferenceOptions,
  ) -> String {
    let info = self
      .modules_map
      .get(module)
      .expect("should have module info");

    let export_data = if !options.ids.is_empty() {
      hex::encode(simd_json::to_string(&options.ids).expect("should serialize to json string"))
    } else {
      "ns".to_string()
    };

    let mut index_buffer = itoa::Buffer::new();
    let index_str = index_buffer.format(info.index());
    let mut module_ref = String::with_capacity(index_str.len() + export_data.len() + 64);
    module_ref.push_str("__rspack_module_ref");
    module_ref.push_str(index_str);
    module_ref.push('_');
    module_ref.push_str(&export_data);
    if options.call {
      module_ref.push_str("_call");
    }
    if options.direct_import {
      module_ref.push_str("_directImport");
    }
    if options.deferred_import {
      module_ref.push_str("_deferredImport");
    }
    if let Some(asi_safe) = options.asi_safe {
      module_ref.push_str(if asi_safe { "_asiSafe1" } else { "_asiSafe0" });
    }
    module_ref.push_str("__._");
    let entry = self.refs.entry(*module).or_default();
    entry.insert(module_ref.clone(), options);

    module_ref
  }

  pub fn match_module_reference(name: &str) -> Option<ModuleReferenceOptions> {
    let name = name
      .strip_suffix(MODULE_REFERENCE_PROPERTY_ACCESS_SUFFIX)
      .unwrap_or(name);
    let encoded = name
      .strip_prefix(MODULE_REFERENCE_PREFIX)?
      .strip_suffix("__")?;
    let (index, encoded) = encoded.split_once('_')?;
    let index = index.parse().ok()?;

    let export_data_len = encoded.find('_').unwrap_or(encoded.len());
    let (export_data, mut flags) = encoded.split_at(export_data_len);
    let ids = if export_data == "ns" {
      vec![]
    } else {
      serde_json::from_slice(&hex::decode(export_data).ok()?).ok()?
    };

    let call = if let Some(stripped) = flags.strip_prefix("_call") {
      flags = stripped;
      true
    } else {
      false
    };
    let direct_import = if let Some(stripped) = flags.strip_prefix("_directImport") {
      flags = stripped;
      true
    } else {
      false
    };
    let deferred_import = if let Some(stripped) = flags.strip_prefix("_deferredImport") {
      flags = stripped;
      true
    } else {
      false
    };
    let asi_safe = if let Some(stripped) = flags.strip_prefix("_asiSafe") {
      let (flag, rest) = stripped.split_at(1);
      flags = rest;
      match flag {
        "0" => Some(false),
        "1" => Some(true),
        _ => return None,
      }
    } else {
      None
    };

    if !flags.is_empty() {
      return None;
    }

    Some(ModuleReferenceOptions {
      ids,
      call,
      direct_import,
      deferred_import,
      asi_safe,
      index,
    })
  }

  pub fn is_module_concatenated(&self, module: &ModuleIdentifier) -> bool {
    matches!(
      self.modules_map.get(module).expect("should have module"),
      ModuleInfo::Concatenated(_)
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::concatenated_module::ExternalModuleInfo;

  fn create_test_scope(index: usize) -> (ConcatenationScope, ModuleIdentifier) {
    let concat_module_id: ModuleIdentifier = "concat-module".into();
    let referenced_module_id: ModuleIdentifier = "referenced-module".into();

    let current_module = ConcatenatedModuleInfo {
      module: concat_module_id,
      ..Default::default()
    };

    let mut modules_map = IdentifierIndexMap::default();
    modules_map.insert(
      referenced_module_id,
      ModuleInfo::External(ExternalModuleInfo::new(index, referenced_module_id)),
    );

    (
      ConcatenationScope::new(concat_module_id, Arc::new(modules_map), current_module),
      referenced_module_id,
    )
  }

  fn assert_module_reference_options_eq(
    actual: &ModuleReferenceOptions,
    expected: &ModuleReferenceOptions,
  ) {
    assert_eq!(actual.ids, expected.ids);
    assert_eq!(actual.call, expected.call);
    assert_eq!(actual.direct_import, expected.direct_import);
    assert_eq!(actual.deferred_import, expected.deferred_import);
    assert_eq!(actual.asi_safe, expected.asi_safe);
    assert_eq!(actual.index, expected.index);
  }

  #[test]
  fn create_module_reference_round_trips_through_matcher() {
    let (mut scope, referenced_module_id) = create_test_scope(7);
    let options = ModuleReferenceOptions {
      ids: vec![Atom::from("default"), Atom::from("named")],
      call: true,
      direct_import: true,
      deferred_import: true,
      asi_safe: Some(false),
      ..Default::default()
    };

    let module_ref = scope.create_module_reference(&referenced_module_id, options.clone());

    let stored = scope
      .refs
      .get(&referenced_module_id)
      .and_then(|refs| refs.get(&module_ref))
      .expect("should store created module reference");
    assert_module_reference_options_eq(stored, &options);

    let parsed = ConcatenationScope::match_module_reference(&module_ref)
      .expect("should parse full module reference");
    let expected = ModuleReferenceOptions {
      index: 7,
      ..options
    };
    assert_module_reference_options_eq(&parsed, &expected);
  }

  #[test]
  fn match_module_reference_accepts_identifier_without_property_access_suffix() {
    let parsed = ConcatenationScope::match_module_reference(
      "__rspack_module_ref3_ns_call_directImport_deferredImport_asiSafe1__",
    )
    .expect("should parse identifier-only module reference");

    assert!(parsed.ids.is_empty());
    assert!(parsed.call);
    assert!(parsed.direct_import);
    assert!(parsed.deferred_import);
    assert_eq!(parsed.asi_safe, Some(true));
    assert_eq!(parsed.index, 3);
  }

  #[test]
  fn match_module_reference_rejects_invalid_suffix() {
    assert!(
      ConcatenationScope::match_module_reference("__rspack_module_ref1_ns_asiSafe2__").is_none()
    );
  }
}
