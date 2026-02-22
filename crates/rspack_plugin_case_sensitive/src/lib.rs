// https://github.com/webpack/webpack/blob/main/lib/WarnCaseSensitiveModulesPlugin.js

use cow_utils::CowUtils;
use itertools::Itertools;
use rspack_collections::{Identifier, IdentifierSet};
use rspack_core::{Compilation, CompilationSeal, CompilerEmit, Logger, ModuleGraph, Plugin};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::{FxBuildHasher, FxHashMap as HashMap, FxHashSet as HashSet};

#[plugin]
#[derive(Debug, Default)]
pub struct CaseSensitivePlugin;

impl CaseSensitivePlugin {
  pub fn create_sensitive_modules_warning(
    &self,
    modules: Vec<Identifier>,
    graph: &ModuleGraph,
  ) -> String {
    let mut message =
      String::from("There are multiple modules with names that only differ in casing.\n");

    for m in modules {
      if let Some(boxed_m) = graph.module_by_identifier(&m) {
        message.push_str("  - ");
        message.push_str(&m);
        message.push('\n');
        graph
          .get_incoming_connections(&boxed_m.identifier())
          .for_each(|c| {
            if let Some(original_identifier) = c.original_module_identifier {
              message.push_str("    - used by ");
              message.push_str(&original_identifier);
              message.push('\n');
            }
          });
      }
    }

    message
  }

  pub fn create_sensitive_assets_warning(&self, filenames: &HashSet<String>) -> String {
    let filenames_str = filenames.iter().map(|f| format!("  - {f}")).join("\n");
    format!(
      r#"Prevent writing to file that only differs in casing or query string from already written file.
This will lead to a race-condition and corrupted files on case-insensitive file systems.
{filenames_str}"#
    )
  }
}

#[plugin_hook(CompilationSeal for CaseSensitivePlugin)]
async fn seal(&self, compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger(self.name());
  let start = logger.time("check case sensitive modules");
  let mut diagnostics: Vec<Diagnostic> = vec![];
  let module_graph = compilation.get_module_graph();
  let mut not_conflect: HashMap<String, Identifier> =
    HashMap::with_capacity_and_hasher(module_graph.modules_len(), FxBuildHasher);
  let mut conflict: HashMap<String, IdentifierSet> = HashMap::default();

  for (_, module) in module_graph.modules() {
    // Ignore `data:` URLs, because it's not a real path
    if let Some(normal_module) = module.as_normal_module()
      && normal_module
        .resource_resolved_data()
        .encoded_content()
        .is_some()
    {
      continue;
    }

    let identifier = module.identifier();
    let lower_identifier = identifier.cow_to_ascii_lowercase();
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
      "Sensitive Warn".to_string(),
      self.create_sensitive_modules_warning(case_modules, compilation.get_module_graph()),
    ));
  }

  compilation.extend_diagnostics(diagnostics);
  logger.time_end(start);
  Ok(())
}

#[plugin_hook(CompilerEmit for CaseSensitivePlugin)]
async fn emit(&self, compilation: &mut Compilation) -> Result<()> {
  let mut diagnostics: Vec<Diagnostic> = vec![];

  // Check for case-sensitive conflicts before emitting assets
  // Only check for filenames that differ in casing (not query strings)
  // Only report conflict if filenames have same lowercase but different casing
  let mut case_map: HashMap<String, HashSet<String>> = HashMap::default();
  for filename in compilation.assets().keys() {
    let (target_file, _query) = filename.split_once('?').unwrap_or((filename, ""));
    let lower_key = cow_utils::CowUtils::cow_to_lowercase(target_file);
    case_map
      .entry(lower_key.to_string())
      .or_default()
      .insert(target_file.to_string());
  }

  // Found conflict: multiple filenames with same lowercase representation but different casing
  for (_lower_key, filenames) in case_map.iter() {
    // Only report conflict if there are multiple unique filenames (different casing)
    if filenames.len() > 1 {
      diagnostics.push(Diagnostic::warn(
        "Sensitive Warn".to_string(),
        self.create_sensitive_assets_warning(filenames),
      ));
    }
  }

  compilation.extend_diagnostics(diagnostics);
  Ok(())
}

// This Plugin warns when there are case sensitive modules in the compilation
// which can cause unexpected behavior when deployed on a case-insensitive environment
// it is executed in hook `compilation.seal`
impl Plugin for CaseSensitivePlugin {
  fn name(&self) -> &'static str {
    "rspack.CaseSensitivePlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compilation_hooks.seal.tap(seal::new(self));
    ctx.compiler_hooks.emit.tap(emit::new(self));
    Ok(())
  }
}
