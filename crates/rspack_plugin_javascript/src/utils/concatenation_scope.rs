use std::collections::hash_map::Entry;

use rspack_core::ModuleIdentifier;
use rustc_hash::FxHashMap as HashMap;
use swc_core::atoms::Atom;

#[derive(Debug)]
struct ExternalModuleInfo {
  index: usize,
  module: ModuleIdentifier,
}

#[derive(Debug)]
struct ConcatenatedModuleInfo {
  index: usize,
  module: ModuleIdentifier,
  export_map: HashMap<String, String>,
  raw_export_map: HashMap<String, String>,
  namespace_export_symbol: Option<String>,
}

// Assuming Module is a type that needs to be defined
type Module = (); // Placeholder, replace with the actual definition

enum ModuleInfo {
  External(ExternalModuleInfo),
  Concatenated(ConcatenatedModuleInfo),
}

struct ModuleReferenceOptions {
  ids: Option<Vec<String>>,
  call: bool,
  direct_import: bool,
  asi_safe: Option<bool>,
}

struct ConcatenationScope {
  current_module: ConcatenatedModuleInfo,
  modules_map: HashMap<ModuleIdentifier, ModuleInfo>,
}

impl ConcatenationScope {
  fn new(
    modules_map: HashMap<ModuleIdentifier, ModuleInfo>,
    current_module: ConcatenatedModuleInfo,
  ) -> Self {
    ConcatenationScope {
      current_module,
      modules_map,
    }
  }

  fn is_module_in_scope(&self, module: &ModuleIdentifier) -> bool {
    self.modules_map.contains_key(module)
  }

  fn register_export(&mut self, export_name: Atom, symbol: Atom) {
    match self.current_module.export_map.entry(export_name) {
      Entry::Occupied(occ) => {
        occ.insert(symbol);
      }
      Entry::Vacant(vac) => {
        vac.insert(symbol);
      }
    }
  }

  fn register_raw_export(&mut self, export_name: Atom, expression: Atom) {
    match self.current_module.export_map.entry(export_name) {
      Entry::Occupied(occ) => {
        occ.insert(symbol);
      }
      Entry::Vacant(vac) => {
        vac.insert(symbol);
      }
    }
  }

  fn register_namespace_export(&mut self, symbol: &str) {
    if let ModuleInfo::Concatenated(concatenated_module) = &mut self
      .modules_map
      .get_mut(&self.current_module.module)
      .unwrap()
    {
      concatenated_module.namespace_export_symbol = Some(symbol.to_string());
    }
  }

  fn create_module_reference(
    &self,
    module: &'a Module,
    options: &ModuleReferenceOptions,
  ) -> String {
    let info = match self.modules_map.get(module).unwrap() {
      ModuleInfo::External(external_info) => external_info,
      ModuleInfo::Concatenated(concatenated_info) => concatenated_info,
    };

    let call_flag = if options.call { "_call" } else { "" };
    let direct_import_flag = if options.direct_import {
      "_directImport"
    } else {
      ""
    };
    let asi_safe_flag = match options.asi_safe {
      Some(true) => "_asiSafe1",
      Some(false) => "_asiSafe0",
      None => "",
    };

    let export_data = if let Some(ids) = &options.ids {
      hex::encode(ids.join(",").as_bytes())
    } else {
      "ns".to_string()
    };

    format!(
      "__WEBPACK_MODULE_REFERENCE__{}{}{}{}__._",
      info.index, export_data, call_flag, direct_import_flag, asi_safe_flag
    )
  }

  fn is_module_reference(name: &str) -> bool {
    lazy_static! {
        static ref MODULE_REFERENCE_REGEXP: Regex =
            Regex::new(r"^__WEBPACK_MODULE_REFERENCE__(\d+)_([\da-f]+|ns)(_call)?(_directImport)?(?:_asiSafe(\d))?__$").unwrap();
    }
    MODULE_REFERENCE_REGEXP.is_match(name)
  }

  fn match_module_reference(name: &str) -> Option<ModuleReferenceOptions> {
    lazy_static! {
        static ref MODULE_REFERENCE_REGEXP: Regex =
            Regex::new(r"^__WEBPACK_MODULE_REFERENCE__(\d+)_([\da-f]+|ns)(_call)?(_directImport)?(?:_asiSafe(\d))?__$").unwrap();
    }
    if let Some(captures) = MODULE_REFERENCE_REGEXP.captures(name) {
      let index: usize = captures[1].parse().unwrap();
      let ids: Option<Vec<String>> = if &captures[2] == "ns" {
        None
      } else {
        Some(
          hex::decode(&captures[2])
            .unwrap()
            .split(',')
            .map(String::from)
            .collect(),
        )
      };
      let call = captures.get(3).is_some();
      let direct_import = captures.get(4).is_some();
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
