use std::sync::{Arc, LazyLock};

use anymap::CloneAny;
use rspack_collections::IdentifierIndexMap;
use rspack_util::{
  fx_hash::{FxIndexMap, FxIndexSet},
  itoa,
};
use swc_core::atoms::Atom;

use crate::{
  DependencyRange, ExportMode, ModuleIdentifier, PendingConcatenationScopeInfo,
  concatenated_module::{
    ConcatenatedModuleInfo, FasterModuleConcatenationInfo, GENERATED_TOP_LEVEL_SYMBOL_PREFIX,
    GeneratedTopLevelSymbolTarget, MODULE_REFERENCE_PLACEHOLDER_PREFIX, MODULE_REFERENCE_PREFIX,
    MODULE_REFERENCE_SUFFIX, ModuleInfo, OriginalScopeIdentUpdate,
  },
};

pub static DEFAULT_EXPORT_ATOM: LazyLock<Atom> = LazyLock::new(|| "__rspack_default_export".into());
pub const NAMESPACE_OBJECT_EXPORT: &str = "__rspack_ns_object";
pub const DEFAULT_EXPORT: &str = "__rspack_default_export";
const MODULE_REFERENCE_PROPERTY_ACCESS_SUFFIX: &str = "._";

#[inline]
fn is_ascii_identifier_continue(byte: u8) -> bool {
  byte == b'_' || byte == b'$' || byte.is_ascii_alphanumeric()
}

fn add_used_names_from_generated_code(info: &mut FasterModuleConcatenationInfo, code: &str) {
  // The legacy codegen parse reserves names referenced by replacement code.
  // Faster concatenation skips that parse, so conservatively collect every
  // ASCII identifier-shaped token. False positives only reduce name reuse;
  // missing a name could capture a generated global reference.
  let bytes = code.as_bytes();
  let mut cursor = 0;
  while cursor < bytes.len() {
    while cursor < bytes.len() && !is_ascii_identifier_continue(bytes[cursor]) {
      cursor += 1;
    }
    let start = cursor;
    while cursor < bytes.len() && is_ascii_identifier_continue(bytes[cursor]) {
      cursor += 1;
    }
    if start < cursor
      && (bytes[start] == b'_' || bytes[start] == b'$' || bytes[start].is_ascii_alphabetic())
    {
      info.added_used_names.push(code[start..cursor].into());
    }
  }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ModuleReferenceOptions {
  pub ids: Vec<Atom>,
  pub call: bool,
  pub direct_import: bool,
  pub deferred_import: bool,
  pub asi_safe: Option<bool>,
  pub index: usize,
}

#[derive(Debug, Clone)]
pub struct ConcatenatedModuleReference {
  pub module: ModuleIdentifier,
  pub options: ModuleReferenceOptions,
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
  faster_module_concatenation_info: Option<Box<FasterModuleConcatenationInfo>>,
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
      faster_module_concatenation_info: None,
    }
  }

  pub fn enable_faster_module_concatenation(&mut self) {
    self
      .faster_module_concatenation_info
      .get_or_insert_default();
  }

  pub(crate) fn take_faster_module_concatenation_info(
    &mut self,
  ) -> Option<Box<FasterModuleConcatenationInfo>> {
    self.faster_module_concatenation_info.take()
  }

  pub fn is_faster_module_concatenation(&self) -> bool {
    self.faster_module_concatenation_info.is_some()
  }

  pub fn current_module_with_scope_info(
    &self,
    pending: &PendingConcatenationScopeInfo,
    original_source: &str,
  ) -> ConcatenatedModuleInfo {
    let mut current_module = self.current_module.clone();
    if let Some(faster_info) = self.faster_module_concatenation_info.as_deref() {
      let mut faster_info = faster_info.clone();
      crate::concatenated_module::populate_info_from_pending(
        pending,
        original_source,
        &mut current_module,
        &mut faster_info,
      );
    }
    current_module
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

  pub fn register_generated_export(&mut self, export_name: Atom, preferred_name: &str) -> Atom {
    let symbol = self.ensure_generated_top_level_symbol(preferred_name);
    self.register_export(export_name, symbol.to_string());
    symbol
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

  pub fn register_generated_namespace_export(&mut self, preferred_name: &str) -> Atom {
    let symbol = self.ensure_generated_top_level_symbol(preferred_name);
    self.register_namespace_export(symbol.as_ref());
    symbol
  }

  pub fn remove_original_range(&mut self, range: DependencyRange) {
    self.record_source_edit(Some(range), None);
  }

  pub fn set_original_range_non_shorthand(&mut self, range: DependencyRange) {
    self.record_non_shorthand_source_edit(range, "");
  }

  #[inline]
  pub(crate) fn record_source_edit(
    &mut self,
    removed_range: Option<DependencyRange>,
    generated_code: Option<&str>,
  ) {
    let Some(info) = self.faster_module_concatenation_info.as_deref_mut() else {
      return;
    };
    if let Some(range) = removed_range {
      info
        .original_scope_ident_updates
        .push(OriginalScopeIdentUpdate::Remove(range));
    }
    if let Some(code) = generated_code
      && !code.is_empty()
    {
      add_used_names_from_generated_code(info, code);
    }
  }

  #[inline]
  pub(crate) fn record_non_shorthand_source_edit(
    &mut self,
    range: DependencyRange,
    generated_code: &str,
  ) {
    let Some(info) = self.faster_module_concatenation_info.as_deref_mut() else {
      return;
    };
    info
      .original_scope_ident_updates
      .push(OriginalScopeIdentUpdate::NonShorthand(range));
    if !generated_code.is_empty() {
      add_used_names_from_generated_code(info, generated_code);
    }
  }

  pub fn add_scope_ident(&mut self, symbol: Atom, range: DependencyRange) {
    let Some(info) = self.faster_module_concatenation_info.as_deref_mut() else {
      return;
    };
    info.added_scope_idents.push(crate::AddedScopeIdent {
      symbol,
      range,
      shorthand: false,
      is_class_expr_with_ident: false,
    });
  }

  pub fn ensure_generated_top_level_symbol(&mut self, preferred_name: &str) -> Atom {
    let preferred_name = Atom::from(preferred_name);
    if !self.is_faster_module_concatenation() {
      return preferred_name;
    }
    if let Some(existing) = self
      .current_module
      .generated_top_level_symbols
      .iter()
      .find(|symbol| {
        symbol.target == GeneratedTopLevelSymbolTarget::New
          && symbol.preferred_name == preferred_name
      })
    {
      return existing.placeholder.clone();
    }

    // Use a verbose internal prefix to make collisions with user identifiers and
    // generated data unlikely.
    let placeholder = Atom::from(format!(
      "{GENERATED_TOP_LEVEL_SYMBOL_PREFIX}{}__",
      self.current_module.generated_top_level_symbols.len()
    ));
    self
      .current_module
      .generated_top_level_symbols
      .push(crate::GeneratedTopLevelSymbol {
        preferred_name,
        placeholder: placeholder.clone(),
        target: GeneratedTopLevelSymbolTarget::New,
        resolved_binding: None,
      });
    placeholder
  }

  pub fn rebind_generated_top_level_symbol(
    &mut self,
    preferred_name: &str,
    original_range: DependencyRange,
  ) -> Atom {
    let preferred_name = Atom::from(preferred_name);
    if !self.is_faster_module_concatenation() {
      return preferred_name;
    }
    if let Some(existing) = self
      .current_module
      .generated_top_level_symbols
      .iter()
      .find(|symbol| {
        symbol.target == GeneratedTopLevelSymbolTarget::Rebind { original_range }
          && symbol.preferred_name == preferred_name
      })
    {
      return existing.placeholder.clone();
    }

    let placeholder = Atom::from(format!(
      "{GENERATED_TOP_LEVEL_SYMBOL_PREFIX}{}__",
      self.current_module.generated_top_level_symbols.len()
    ));
    self
      .current_module
      .generated_top_level_symbols
      .push(crate::GeneratedTopLevelSymbol {
        preferred_name,
        placeholder: placeholder.clone(),
        target: GeneratedTopLevelSymbolTarget::Rebind { original_range },
        resolved_binding: None,
      });
    placeholder
  }

  fn build_module_reference(
    &mut self,
    module: &ModuleIdentifier,
    options: &ModuleReferenceOptions,
  ) -> String {
    if self.is_faster_module_concatenation() {
      if let Some((placeholder, _)) = self
        .current_module
        .module_references
        .iter()
        .find(|(_, reference)| reference.module == *module && reference.options == *options)
      {
        return placeholder.clone();
      }

      let mut index_buffer = itoa::Buffer::new();
      let index_str = index_buffer.format(self.current_module.module_references.len());
      let mut placeholder = String::with_capacity(
        MODULE_REFERENCE_PLACEHOLDER_PREFIX.len() + index_str.len() + MODULE_REFERENCE_SUFFIX.len(),
      );
      placeholder.push_str(MODULE_REFERENCE_PLACEHOLDER_PREFIX);
      placeholder.push_str(index_str);
      placeholder.push_str(MODULE_REFERENCE_SUFFIX);
      self.current_module.module_references.insert(
        placeholder.clone(),
        ConcatenatedModuleReference {
          module: *module,
          options: options.clone(),
        },
      );
      return placeholder;
    }

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
    module_ref.push_str(MODULE_REFERENCE_PREFIX);
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
    module_ref.push_str(MODULE_REFERENCE_SUFFIX);
    module_ref
  }

  pub fn create_module_reference(
    &mut self,
    module: &ModuleIdentifier,
    options: ModuleReferenceOptions,
  ) -> String {
    let module_ref = self.build_module_reference(module, &options);
    self
      .refs
      .entry(*module)
      .or_default()
      .insert(module_ref.clone(), options);
    module_ref
  }

  pub fn create_export_reference(
    &mut self,
    module: &ModuleIdentifier,
    options: &ModuleReferenceOptions,
  ) -> String {
    self.build_module_reference(module, options)
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
  fn create_module_reference_tracks_legacy_and_faster_references() {
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
      ..options.clone()
    };
    assert_module_reference_options_eq(&parsed, &expected);

    let (mut faster_scope, referenced_module_id) = create_test_scope(7);
    faster_scope.enable_faster_module_concatenation();
    let module_ref = faster_scope.create_module_reference(&referenced_module_id, options.clone());
    assert_eq!(module_ref, "__rspack_module_reference_placeholder_0__._");

    let reference = faster_scope
      .current_module
      .module_references
      .get(&module_ref)
      .expect("should store structured module reference");
    assert_eq!(reference.module, referenced_module_id);
    assert_module_reference_options_eq(&reference.options, &options);
    assert!(ConcatenationScope::match_module_reference(&module_ref).is_none());
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
