use std::collections::hash_map::Entry;
use std::sync::Arc;

use indexmap::IndexMap;
use once_cell::sync::Lazy;
use regex::Regex;
use swc_core::atoms::Atom;

use crate::concatenated_module::{ConcatenatedModuleInfo, ModuleInfo};
use crate::ModuleIdentifier;

pub const DEFAULT_EXPORT: &str = "__WEBPACK_DEFAULT_EXPORT__";
pub const NAMESPACE_OBJECT_EXPORT: &str = "__WEBPACK_NAMESPACE_OBJECT__";

static MODULE_REFERENCE_REGEXP: Lazy<Regex> = once_cell::sync::Lazy::new(|| {
  Regex::new(
    r"^__WEBPACK_MODULE_REFERENCE__(\d+)_([\da-f]+|ns)(_call)?(_directImport)?(?:_asiSafe(\d))?__$",
  )
  .expect("should initialized regex")
});

#[derive(Default, Debug)]
pub struct ModuleReferenceOptions {
  pub ids: Vec<Atom>,
  pub call: bool,
  pub direct_import: bool,
  pub asi_safe: Option<bool>,
  pub index: usize,
}

#[derive(Debug, Clone)]
pub struct ConcatenationScope {
  pub current_module: ConcatenatedModuleInfo,
  pub modules_map: Arc<IndexMap<ModuleIdentifier, ModuleInfo>>,
}

#[allow(unused)]
impl ConcatenationScope {
  pub fn new(
    modules_map: Arc<IndexMap<ModuleIdentifier, ModuleInfo>>,
    current_module: ConcatenatedModuleInfo,
  ) -> Self {
    ConcatenationScope {
      current_module,
      modules_map,
    }
  }

  pub fn is_module_in_scope(&self, module: &ModuleIdentifier) -> bool {
    self.modules_map.contains_key(module)
  }

  pub fn register_export(&mut self, export_name: Atom, symbol: String) {
    let export_map = self.current_module.export_map.get_or_insert_default();
    match export_map.entry(export_name) {
      Entry::Occupied(mut occ) => {
        occ.insert(symbol);
      }
      Entry::Vacant(vac) => {
        vac.insert(symbol);
      }
    }
  }

  pub fn register_raw_export(&mut self, export_name: Atom, symbol: String) {
    let raw_export_map = self.current_module.raw_export_map.get_or_insert_default();
    match raw_export_map.entry(export_name) {
      Entry::Occupied(mut occ) => {
        occ.insert(symbol);
      }
      Entry::Vacant(vac) => {
        vac.insert(symbol);
      }
    }
  }

  pub fn register_namespace_export(&mut self, symbol: &str) {
    self.current_module.namespace_export_symbol = Some(symbol.into());
  }

  pub fn create_module_reference(
    &self,
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

    format!(
      "__WEBPACK_MODULE_REFERENCE__{}_{}{}{}{}__._",
      info.index(),
      export_data,
      call_flag,
      direct_import_flag,
      asi_safe_flag
    )
  }

  pub fn is_module_reference(name: &str) -> bool {
    MODULE_REFERENCE_REGEXP.is_match(name)
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
      let asi_safe = captures.get(5).map(|s| s.as_str() == "1");
      Some(ModuleReferenceOptions {
        ids,
        call,
        direct_import,
        asi_safe,
        index,
      })
    } else {
      None
    }
  }
}
