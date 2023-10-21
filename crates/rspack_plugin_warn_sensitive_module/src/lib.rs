// https://github.com/webpack/webpack/blob/main/lib/WarnCaseSensitiveModulesPlugin.js

use std::collections::HashMap;

use dashmap::DashSet;
use rspack_core::{Compilation, Logger, Module, ModuleGraph, Plugin};
use rspack_error::Diagnostic;

#[derive(Debug, Default)]
pub struct WarnCaseSensitiveModulesPlugin;

impl WarnCaseSensitiveModulesPlugin {
  pub fn new() -> Self {
    Self
  }

  #[allow(clippy::borrowed_box)]
  pub fn create_sensitive_modules_warning(
    &self,
    modules: &Vec<&&Box<dyn Module>>,
    graph: &ModuleGraph,
  ) -> String {
    let mut message =
      String::from("There are multiple modules with names that only differ in casing.\n");

    for m in modules {
      let mut module_msg = format!("  - {}\n", m.identifier().to_string());
      graph.get_incoming_connections(m).iter().for_each(|c| {
        if let Some(original_identifier) = c.original_module_identifier {
          module_msg.push_str(&format!("    - used by {}\n", original_identifier));
        }
      });
      message.push_str(&module_msg);
    }

    message
  }
}

// This Plugin warns when there are case sensitive modules in the compilation
// which can cause unexpected behavior when deployed on a case-insensitive environment
// it is executed in hook `compilation.seal`
impl Plugin for WarnCaseSensitiveModulesPlugin {
  fn name(&self) -> &'static str {
    "rspack.WarnCaseSensitiveModulesPlugin"
  }

  fn module_ids(&self, compilation: &mut Compilation) -> rspack_error::Result<()> {
    let logger = compilation.get_logger(self.name());
    let start = logger.time("check case sensitive modules");
    let diagnostics: DashSet<Diagnostic> = DashSet::default();
    let modules = compilation
      .module_graph
      .modules()
      .values()
      .collect::<Vec<_>>();
    let mut module_without_case_map: HashMap<String, HashMap<String, &Box<dyn Module>>> =
      HashMap::new();

    for module in modules {
      // Ignore `data:` URLs, because it's not a real path
      if let Some(normal_module) = module.as_normal_module() {
        if normal_module
          .resource_resolved_data()
          .encoded_content
          .is_some()
        {
          continue;
        }
      }

      let identifier = module.identifier().to_string();
      let lower_identifier = identifier.to_lowercase();
      let lower_map = module_without_case_map
        .entry(lower_identifier)
        .or_insert(HashMap::new());
      lower_map.insert(identifier, module);
    }

    for lower_map in module_without_case_map.values() {
      if lower_map.len() > 1 {
        let mut modules = lower_map.values().collect::<Vec<_>>();
        modules.sort_by_key(|m| m.identifier());
        diagnostics.insert(Diagnostic::warn(
          "Sensitive Modules Warn".to_string(),
          self.create_sensitive_modules_warning(&modules, &compilation.module_graph),
          0,
          0,
        ));
      }
    }

    compilation.push_batch_diagnostic(diagnostics.into_iter().collect());
    logger.time_end(start);
    Ok(())
  }
}
