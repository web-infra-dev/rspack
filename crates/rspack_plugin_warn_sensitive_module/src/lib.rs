// https://github.com/webpack/webpack/blob/main/lib/WarnCaseSensitiveModulesPlugin.js

use std::{collections::HashMap, hash::BuildHasherDefault};

use cow_utils::CowUtils;
use rspack_collections::{Identifier, IdentifierSet};
use rspack_core::{
  ApplyContext, Compilation, CompilationSeal, CompilerOptions, Logger, ModuleGraph, Plugin,
  PluginContext,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::{FxHashMap, FxHasher};

#[plugin]
#[derive(Debug, Default)]
pub struct WarnCaseSensitiveModulesPlugin;

impl WarnCaseSensitiveModulesPlugin {
  pub fn create_sensitive_modules_warning(
    &self,
    modules: Vec<Identifier>,
    graph: &ModuleGraph,
  ) -> String {
    let mut message =
      String::from("There are multiple modules with names that only differ in casing.\n");

    for m in modules {
      if let Some(boxed_m) = graph.module_by_identifier(&m) {
        let mut module_msg = format!("  - {m}\n");
        graph
          .get_incoming_connections(&boxed_m.identifier())
          .iter()
          .for_each(|c| {
            if let Some(original_identifier) = c.original_module_identifier {
              module_msg.push_str(&format!("    - used by {original_identifier}\n"));
            }
          });

        message.push_str(&module_msg);
      }
    }

    message
  }
}

#[plugin_hook(CompilationSeal for WarnCaseSensitiveModulesPlugin)]
async fn seal(&self, compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger(self.name());
  let start = logger.time("check case sensitive modules");
  let mut diagnostics: Vec<Diagnostic> = vec![];
  let module_graph = compilation.get_module_graph();
  let mut not_conflect: FxHashMap<String, Identifier> = HashMap::with_capacity_and_hasher(
    module_graph.modules().len(),
    BuildHasherDefault::<FxHasher>::default(),
  );
  let mut conflict: FxHashMap<String, IdentifierSet> = FxHashMap::default();

  for module in module_graph.modules().values() {
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

    let identifier = module.identifier();
    let lower_identifier = identifier.cow_to_lowercase();
    if let Some(prev_identifier) = not_conflect.remove(lower_identifier.as_ref()) {
      conflict.insert(
        lower_identifier.into_owned(),
        IdentifierSet::from_iter([prev_identifier, identifier]),
      );
    } else if let Some(set) = conflict.get_mut(lower_identifier.as_ref()) {
      set.insert(identifier);
    } else {
      not_conflect.insert(lower_identifier.into_owned(), identifier);
    }
  }

  // sort by module identifier, guarantee the warning order
  let mut case_map_vec = conflict.into_iter().collect::<Vec<_>>();
  case_map_vec.sort_unstable_by(|a, b| a.0.cmp(&b.0));

  for (_, set) in case_map_vec {
    let mut case_modules = set.iter().copied().collect::<Vec<_>>();
    case_modules.sort_unstable();
    diagnostics.push(Diagnostic::warn(
      "Sensitive Modules Warn".to_string(),
      self.create_sensitive_modules_warning(case_modules, &compilation.get_module_graph()),
    ));
  }

  compilation.extend_diagnostics(diagnostics);
  logger.time_end(start);
  Ok(())
}

// This Plugin warns when there are case sensitive modules in the compilation
// which can cause unexpected behavior when deployed on a case-insensitive environment
// it is executed in hook `compilation.seal`
impl Plugin for WarnCaseSensitiveModulesPlugin {
  fn name(&self) -> &'static str {
    "rspack.WarnCaseSensitiveModulesPlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx.context.compilation_hooks.seal.tap(seal::new(self));
    Ok(())
  }
}
