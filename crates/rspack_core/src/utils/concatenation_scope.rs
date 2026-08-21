use std::sync::{Arc, LazyLock};

use anymap::CloneAny;
use rspack_cacheable::{
  cacheable,
  with::{AsCacheable, AsMap, AsOption, AsPreset, AsVec},
};
use rspack_collections::IdentifierIndexMap;
use rspack_util::{fx_hash::FxIndexMap, itoa};
use rustc_hash::FxHashMap as HashMap;
use swc_core::atoms::Atom;
use swc_experimental_ecma_ast::{is_valid_continue, is_valid_start};

use crate::{
  ModuleIdentifier,
  concatenated_module::{ConcatenatedImportMap, ConcatenatedModuleInfo, ModuleInfo},
};

pub static DEFAULT_EXPORT_ATOM: LazyLock<Atom> = LazyLock::new(|| "__rspack_default_export".into());
pub const NAMESPACE_OBJECT_EXPORT: &str = "__rspack_ns_object";
pub const DEFAULT_EXPORT: &str = "__rspack_default_export";
const MODULE_REFERENCE_PREFIX: &str = "__rspack_module_ref";
const MODULE_REFERENCE_PROPERTY_ACCESS_SUFFIX: &str = "._";

#[inline]
fn is_ascii_identifier_continue(byte: u8) -> bool {
  byte == b'_' || byte == b'$' || byte.is_ascii_alphanumeric()
}

#[inline]
fn is_ascii_identifier_start(byte: u8) -> bool {
  byte == b'_' || byte == b'$' || byte.is_ascii_alphabetic()
}

#[inline]
fn hex_value(byte: u8) -> Option<u32> {
  match byte {
    b'0'..=b'9' => Some((byte - b'0') as u32),
    b'a'..=b'f' => Some((byte - b'a' + 10) as u32),
    b'A'..=b'F' => Some((byte - b'A' + 10) as u32),
    _ => None,
  }
}

fn parse_unicode_escape(code: &[u8], start: usize) -> Option<(char, usize)> {
  if code.get(start..start + 2)? != b"\\u" {
    return None;
  }

  let mut cursor = start + 2;
  let mut value = 0u32;
  if code.get(cursor) == Some(&b'{') {
    cursor += 1;
    let digits_start = cursor;
    while let Some(digit) = code.get(cursor).and_then(|byte| hex_value(*byte)) {
      if cursor - digits_start == 6 {
        return None;
      }
      value = value.checked_mul(16)?.checked_add(digit)?;
      cursor += 1;
    }
    if cursor == digits_start || code.get(cursor) != Some(&b'}') {
      return None;
    }
    cursor += 1;
  } else {
    for _ in 0..4 {
      value = value
        .checked_mul(16)?
        .checked_add(hex_value(*code.get(cursor)?)?)?;
      cursor += 1;
    }
  }

  char::from_u32(value).map(|character| (character, cursor))
}

/// Returns the end offset and, only when escapes were present, the decoded
/// canonical identifier name.
fn scan_generated_identifier(code: &str, start: usize) -> Option<(usize, Option<String>)> {
  let bytes = code.as_bytes();
  let mut cursor = start;
  let mut canonical_name: Option<String> = None;
  let mut is_start = true;

  while cursor < bytes.len() {
    let (character, end, escaped) = if bytes[cursor] == b'\\' {
      let Some((character, end)) = parse_unicode_escape(bytes, cursor) else {
        if is_start {
          return None;
        }
        break;
      };
      (character, end, true)
    } else {
      let character = code[cursor..].chars().next()?;
      (character, cursor + character.len_utf8(), false)
    };
    if if is_start {
      !is_valid_start(character)
    } else {
      !is_valid_continue(character)
    } {
      break;
    }

    if escaped {
      canonical_name
        .get_or_insert_with(|| {
          let mut name = String::with_capacity(end - start);
          name.push_str(&code[start..cursor]);
          name
        })
        .push(character);
    } else if let Some(name) = &mut canonical_name {
      name.push(character);
    }
    cursor = end;
    is_start = false;
  }

  (cursor != start).then_some((cursor, canonical_name))
}

fn visit_generated_identifiers(code: &str, mut visitor: impl FnMut(&str) -> bool) -> bool {
  // Conservatively inspect every identifier-shaped token in generated
  // replacement code. False positives only reduce name reuse; missing a name
  // could capture a generated global reference.
  let bytes = code.as_bytes();
  let mut cursor = 0;
  while cursor < bytes.len() {
    while cursor < bytes.len()
      && bytes[cursor].is_ascii()
      && !is_ascii_identifier_continue(bytes[cursor])
      && bytes[cursor] != b'\\'
    {
      cursor += 1;
    }
    if cursor == bytes.len() {
      break;
    }

    if !bytes[cursor].is_ascii() || bytes[cursor] == b'\\' {
      if let Some((end, canonical_name)) = scan_generated_identifier(code, cursor) {
        let name = canonical_name.as_deref().unwrap_or(&code[cursor..end]);
        if visitor(name) {
          return true;
        }
        cursor = end;
      } else {
        cursor += if bytes[cursor].is_ascii() {
          1
        } else {
          code[cursor..]
            .chars()
            .next()
            .expect("cursor should be on a character boundary")
            .len_utf8()
        };
      }
      continue;
    }

    let start = cursor;
    while cursor < bytes.len() && is_ascii_identifier_continue(bytes[cursor]) {
      cursor += 1;
    }
    if !is_ascii_identifier_start(bytes[start]) {
      continue;
    }
    if cursor < bytes.len() && (!bytes[cursor].is_ascii() || bytes[cursor] == b'\\') {
      let (end, canonical_name) = scan_generated_identifier(code, start)
        .expect("ASCII identifier start should produce an identifier");
      let name = canonical_name.as_deref().unwrap_or(&code[start..end]);
      if visitor(name) {
        return true;
      }
      cursor = end;
    } else {
      if visitor(&code[start..cursor]) {
        return true;
      }
    }
  }

  false
}

/// Checks whether generated JavaScript contains an identifier after decoding
/// Unicode escapes in its spelling.
pub fn generated_code_contains_identifier(code: &str, identifier: &str) -> bool {
  visit_generated_identifiers(code, |name| name == identifier)
}
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

/// Cacheable mutations produced while generating a module with a
/// [`ConcatenationScope`]. The ESM library plugin opts into collecting this
/// output because it consumes the mutations after the module codegen pass.
#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct CodeGenerationDataConcatenationScopeOutput {
  #[cacheable(with=AsOption<AsPreset>)]
  namespace_export_symbol: Option<Atom>,
  #[cacheable(with=AsOption<AsMap<AsPreset, AsCacheable>>)]
  export_map: Option<HashMap<Atom, String>>,
  #[cacheable(with=AsOption<AsMap<AsPreset, AsCacheable>>)]
  raw_export_map: Option<HashMap<Atom, String>>,
  #[cacheable(with=AsOption<AsMap<AsCacheable, AsCacheable>>)]
  import_map: ConcatenatedImportMap,
  #[cacheable(with=AsMap<AsCacheable, AsMap<AsCacheable, AsCacheable>>)]
  refs: IdentifierIndexMap<FxIndexMap<String, ModuleReferenceOptions>>,
}

impl CodeGenerationDataConcatenationScopeOutput {
  pub fn apply_to(&self, current_module: &mut ConcatenatedModuleInfo) {
    current_module
      .namespace_export_symbol
      .clone_from(&self.namespace_export_symbol);
    current_module.export_map.clone_from(&self.export_map);
    current_module
      .raw_export_map
      .clone_from(&self.raw_export_map);
    current_module.import_map.clone_from(&self.import_map);
  }

  pub fn refs(&self) -> &IdentifierIndexMap<FxIndexMap<String, ModuleReferenceOptions>> {
    &self.refs
  }
}

#[derive(Debug, Clone)]
pub struct ConcatenationScope {
  pub concat_module_id: ModuleIdentifier,
  pub current_module: ConcatenatedModuleInfo,
  pub modules_map: Arc<IdentifierIndexMap<ModuleInfo>>,
  pub data: anymap::Map<dyn CloneAny + Send + Sync>,
  pub refs: IdentifierIndexMap<FxIndexMap<String, ModuleReferenceOptions>>,
  collect_codegen_data: bool,
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
      collect_codegen_data: false,
    }
  }

  /// Persist the codegen mutations that are consumed after module code
  /// generation. This is opt-in so regular module concatenation does not pay
  /// the collection or cache-storage cost.
  pub fn enable_codegen_data_collection(&mut self) {
    self.collect_codegen_data = true;
  }

  pub fn is_codegen_data_collection_enabled(&self) -> bool {
    self.collect_codegen_data
  }

  pub(crate) fn take_codegen_data_output(&mut self) -> CodeGenerationDataConcatenationScopeOutput {
    assert!(self.collect_codegen_data);
    CodeGenerationDataConcatenationScopeOutput {
      namespace_export_symbol: self.current_module.namespace_export_symbol.take(),
      export_map: self.current_module.export_map.take(),
      raw_export_map: self.current_module.raw_export_map.take(),
      import_map: self.current_module.import_map.take(),
      refs: std::mem::take(&mut self.refs),
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
