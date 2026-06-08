use std::sync::{Arc, LazyLock};

use anymap::CloneAny;
use rspack_collections::IdentifierIndexMap;
use rspack_util::{
  fx_hash::{FxIndexMap, FxIndexSet},
  itoa,
};
use swc_core::atoms::Atom;

use crate::{
  ExportMode, ModuleIdentifier,
  concatenated_module::{ConcatenatedModuleInfo, ModuleInfo},
};

pub static DEFAULT_EXPORT_ATOM: LazyLock<Atom> = LazyLock::new(|| "__rspack_default_export".into());
pub const NAMESPACE_OBJECT_EXPORT: &str = "__rspack_ns_object";
pub const DEFAULT_EXPORT: &str = "__rspack_default_export";
const MODULE_REFERENCE_PREFIX: &str = "__rspack_module_ref";
const MODULE_REFERENCE_PROPERTY_ACCESS_SUFFIX: &str = "._";

#[derive(Default, Debug, Clone)]
pub struct ModuleReferenceOptions {
  pub ids: Vec<Atom>,
  pub call: bool,
  pub direct_import: bool,
  pub deferred_import: bool,
  pub asi_safe: Option<bool>,
  pub index: usize,
}

#[derive(Debug, Clone)]
pub struct ConcatenationScope {
  pub concat_module_id: ModuleIdentifier,
  pub current_module: ConcatenatedModuleInfo,
  pub modules_map: Arc<IdentifierIndexMap<ModuleInfo>>,
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
      data: Default::default(),
      refs: IdentifierIndexMap::default(),
      dyn_refs: Default::default(),
      re_exports: Default::default(),
    }
  }

  pub fn is_module_in_scope(&self, module: &ModuleIdentifier) -> bool {
    self.modules_map.contains_key(module)
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
    mut options: ModuleReferenceOptions,
  ) -> String {
    let info = self
      .modules_map
      .get(module)
      .expect("should have module info");

    options.index = info.index();

    let mut index_buffer = itoa::Buffer::new();
    let index_str = index_buffer.format(options.index);
    let mut reference_index_buffer = itoa::Buffer::new();
    let reference_index_str =
      reference_index_buffer.format(self.current_module.module_references.len());
    let mut module_ref_identifier =
      String::with_capacity(index_str.len() + reference_index_str.len() + 32);
    module_ref_identifier.push_str(MODULE_REFERENCE_PREFIX);
    module_ref_identifier.push_str(index_str);
    module_ref_identifier.push('_');
    module_ref_identifier.push_str(reference_index_str);
    module_ref_identifier.push_str("__");

    self
      .current_module
      .module_references
      .insert(Atom::from(module_ref_identifier.as_str()), options.clone());

    let mut module_ref = String::with_capacity(
      module_ref_identifier.len() + MODULE_REFERENCE_PROPERTY_ACCESS_SUFFIX.len(),
    );
    module_ref.push_str(&module_ref_identifier);
    module_ref.push_str(MODULE_REFERENCE_PROPERTY_ACCESS_SUFFIX);

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
  fn create_module_reference_stores_lookup_and_legacy_refs() {
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
    assert!(module_ref.ends_with(MODULE_REFERENCE_PROPERTY_ACCESS_SUFFIX));
    let module_ref_identifier = module_ref
      .strip_suffix(MODULE_REFERENCE_PROPERTY_ACCESS_SUFFIX)
      .expect("should include module reference property access suffix");
    let expected = ModuleReferenceOptions {
      index: 7,
      ..options
    };

    let stored = scope
      .refs
      .get(&referenced_module_id)
      .and_then(|refs| refs.get(&module_ref))
      .expect("should store created module reference");
    assert_module_reference_options_eq(stored, &expected);

    let lookup_key = Atom::from(module_ref_identifier);
    let lookup = scope
      .current_module
      .module_references
      .get(&lookup_key)
      .expect("should store created module reference lookup");
    assert_module_reference_options_eq(lookup, &expected);
  }

  #[test]
  fn create_module_reference_preserves_option_branches() {
    let (mut scope, referenced_module_id) = create_test_scope(7);
    let options_cases = vec![
      ModuleReferenceOptions::default(),
      ModuleReferenceOptions {
        ids: vec![Atom::from("named")],
        direct_import: true,
        asi_safe: Some(true),
        ..Default::default()
      },
      ModuleReferenceOptions {
        ids: vec![Atom::from("default"), Atom::from("named")],
        call: true,
        deferred_import: true,
        asi_safe: Some(false),
        ..Default::default()
      },
    ];

    for options in options_cases {
      let module_ref = scope.create_module_reference(&referenced_module_id, options.clone());
      let module_ref_identifier = module_ref
        .strip_suffix(MODULE_REFERENCE_PROPERTY_ACCESS_SUFFIX)
        .expect("should include module reference property access suffix");
      let expected = ModuleReferenceOptions {
        index: 7,
        ..options
      };

      let stored = scope
        .refs
        .get(&referenced_module_id)
        .and_then(|refs| refs.get(&module_ref))
        .expect("should store created module reference");
      assert_module_reference_options_eq(stored, &expected);

      let lookup_key = Atom::from(module_ref_identifier);
      let lookup = scope
        .current_module
        .module_references
        .get(&lookup_key)
        .expect("should store created module reference lookup");
      assert_module_reference_options_eq(lookup, &expected);
    }
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
