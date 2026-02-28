use std::sync::{Arc, LazyLock};

use anymap::CloneAny;
use regex::Regex;
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

static MODULE_REFERENCE_REGEXP: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(
    r"^__rspack_module_ref(\d+)_([\da-f]+|ns)(_call)?(_directImport)?(_deferredImport)?(?:_asiSafe(\d))?__$",
  )
  .expect("should initialized regex")
});

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
    options: &ModuleReferenceOptions,
  ) -> String {
    let info = self
      .modules_map
      .get(module)
      .expect("should have module info");

    let call_flag = match options.call {
      true => "_call",
      _ => "",
    };
    let direct_import_flag = match options.direct_import {
      true => "_directImport",
      _ => "",
    };
    let deferred_import_flag = match options.deferred_import {
      true => "_deferredImport",
      _ => "",
    };
    let asi_safe_flag = match options.asi_safe {
      Some(true) => "_asiSafe1",
      Some(false) => "_asiSafe0",
      None => "",
    };

    let export_data = if !options.ids.is_empty() {
      hex::encode(serde_json::to_string(&options.ids).expect("should serialize to json string"))
    } else {
      "ns".to_string()
    };

    let mut index_buffer = itoa::Buffer::new();
    let index_str = index_buffer.format(info.index());
    let module_ref = format!(
      "__rspack_module_ref{index_str}_{export_data}{call_flag}{direct_import_flag}{deferred_import_flag}{asi_safe_flag}__._"
    );
    let entry = self.refs.entry(*module).or_default();
    entry.insert(module_ref.clone(), options.clone());

    module_ref
  }

  pub fn match_module_reference(name: &str) -> Option<ModuleReferenceOptions> {
    if let Some(captures) = MODULE_REFERENCE_REGEXP.captures(name) {
      let index: usize = captures[1].parse().expect("");
      let ids: Vec<Atom> = if &captures[2] == "ns" {
        vec![]
      } else {
        serde_json::from_slice(&hex::decode(&captures[2]).expect("should decode hex"))
          .expect("should have deserialize")
      };
      let call = captures.get(3).is_some();
      let direct_import = captures.get(4).is_some();
      let deferred_import = captures.get(5).is_some();
      let asi_safe = captures.get(6).map(|s| s.as_str() == "1");
      Some(ModuleReferenceOptions {
        ids,
        call,
        direct_import,
        deferred_import,
        asi_safe,
        index,
      })
    } else {
      None
    }
  }

  pub fn is_module_concatenated(&self, module: &ModuleIdentifier) -> bool {
    matches!(
      self.modules_map.get(module).expect("should have module"),
      ModuleInfo::Concatenated(_)
    )
  }
}
