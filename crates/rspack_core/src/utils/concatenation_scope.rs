use std::collections::hash_map::Entry;
use std::sync::Arc;

use once_cell::sync::Lazy;
use regex::Regex;
use rustc_hash::FxHashMap as HashMap;
use swc_core::atoms::Atom;

use crate::concatenated_module::{ConcatenatedModuleInfo, ModuleInfo};
use crate::ModuleIdentifier;

pub const DEFAULT_EXPORT: &'static str = "__WEBPACK_DEFAULT_EXPORT__";
pub const NAMESPACE_OBJECT_EXPORT: &'static str = "__WEBPACK_NAMESPACE_OBJECT__";

const MODULE_REFERENCE_REGEXP: Lazy<Regex> = once_cell::sync::Lazy::new(|| {
  Regex::new(
    r"^__WEBPACK_MODULE_REFERENCE__(\d+)_([\da-f]+|ns)(_call)?(_directImport)?(?:_asiSafe(\d))?__$",
  )
  .unwrap()
});

struct ModuleReferenceOptions {
  ids: Option<Vec<String>>,
  call: Option<bool>,
  direct_import: Option<bool>,
  asi_safe: Option<bool>,
}

#[allow(unused)]
#[derive(Debug)]
pub struct ConcatenationScope {
  current_module: ConcatenatedModuleInfo,
  modules_map: Arc<HashMap<ModuleIdentifier, ModuleInfo>>,
  /// Noticed that, this field is rspack only, because defer mutable operation could work around
  /// rustc borrow checker and avoid clone overhead
  pub concated_module_namespace_export_symbol: Option<Atom>,
}

#[allow(unused)]
impl ConcatenationScope {
  pub fn new(
    modules_map: Arc<HashMap<ModuleIdentifier, ModuleInfo>>,
    current_module: ConcatenatedModuleInfo,
  ) -> Self {
    ConcatenationScope {
      current_module,
      modules_map,
      concated_module_namespace_export_symbol: None,
    }
  }

  pub fn is_module_in_scope(&self, module: &ModuleIdentifier) -> bool {
    self.modules_map.contains_key(module)
  }

  pub fn register_export(&mut self, export_name: Atom, symbol: String) {
    match self.current_module.export_map.entry(export_name) {
      Entry::Occupied(mut occ) => {
        occ.insert(symbol);
      }
      Entry::Vacant(vac) => {
        vac.insert(symbol);
      }
    }
  }

  pub fn register_raw_export(&mut self, export_name: Atom, symbol: String) {
    match self.current_module.raw_export_map.entry(export_name) {
      Entry::Occupied(mut occ) => {
        occ.insert(symbol);
      }
      Entry::Vacant(vac) => {
        vac.insert(symbol);
      }
    }
  }

  pub fn register_namespace_export(&mut self, symbol: &str) {
    if let Some(ModuleInfo::Concatenated(concatenated_module)) =
      &mut self.modules_map.get(&self.current_module.module)
    {
      self.concated_module_namespace_export_symbol = Some(symbol.into());
    }
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
      Some(true) => "_call",
      _ => "",
    };
    let direct_import_flag = match options.direct_import {
      Some(true) => "_directImport",
      _ => "",
    };
    let asi_safe_flag = match options.asi_safe {
      Some(true) => "_asiSafe1",
      Some(false) => "_asiSafe0",
      None => "",
    };

    let export_data = if let Some(ids) = &options.ids {
      hex::encode(serde_json::to_string(ids).expect("should serialize to json string"))
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
      let index: usize = captures[1].parse().unwrap();
      let ids: Option<Vec<String>> = if &captures[2] == "ns" {
        Some(vec![])
      } else {
        Some(
          serde_json::from_slice(&hex::decode(&captures[2]).expect("should decode hex"))
            .expect("should have deserialize"),
        )
      };
      let call = Some(captures.get(3).is_some());
      let direct_import = Some(captures.get(4).is_some());
      let asi_safe = captures.get(5).map(|s| s.as_str() == "1");
      Some(ModuleReferenceOptions {
        ids,
        call,
        direct_import,
        asi_safe,
      })
    } else {
      None
    }
  }
}
