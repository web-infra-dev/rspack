use derive_more::Debug;
use rayon::prelude::*;
use rspack_core::{
  ChunkGraph, Compilation, CompilationModuleIds, ModuleIdsArtifact, Plugin,
  incremental::IncrementalPasses,
};
use rspack_error::{Diagnostic, Result, error};
use rspack_hook::{plugin, plugin_hook};

use crate::{
  compat_hashed_id::{
    CompatHashedIdAssigner, FULL_IDENTIFIER_LENGTH, hash_identifier, normalize_min_length,
    validate_min_length,
  },
  id_helpers::{
    compare_modules_by_pre_order_index_or_identifier, get_full_module_name,
    get_used_module_ids_and_modules_with_artifact,
  },
};

#[derive(Debug, Clone, Default)]
pub struct CompatHashedModuleIdsPluginOptions {
  pub min_length: Option<usize>,
}

#[plugin]
#[derive(Debug)]
pub struct CompatHashedModuleIdsPlugin {
  min_length: usize,
}

impl Default for CompatHashedModuleIdsPlugin {
  fn default() -> Self {
    Self::new(Default::default())
  }
}

impl CompatHashedModuleIdsPlugin {
  pub fn new(options: CompatHashedModuleIdsPluginOptions) -> Self {
    Self::new_inner(normalize_min_length(options.min_length))
  }
}

#[plugin_hook(CompilationModuleIds for CompatHashedModuleIdsPlugin)]
async fn module_ids(
  &self,
  compilation: &Compilation,
  module_ids: &mut ModuleIdsArtifact,
  preserved_module_ids: &ModuleIdsArtifact,
  diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
  if let Some(diagnostic) = compilation.incremental.disable_passes(
    IncrementalPasses::MODULE_IDS | IncrementalPasses::MODULES_HASHES,
    "CompatHashedModuleIdsPlugin (optimization.moduleIds = \"compat-hashed\")",
    "it requires calculating the id of all the modules, which is a global effect",
  ) {
    if let Some(diagnostic) = diagnostic {
      diagnostics.push(diagnostic);
    }
    module_ids.retain(|module, _| preserved_module_ids.contains_key(module));
  }

  validate_min_length(self.min_length, "CompatHashedModuleIdsPlugin")?;

  let (used_ids, modules) =
    get_used_module_ids_and_modules_with_artifact(compilation, module_ids, None);

  let mut module_ids_map = std::mem::take(module_ids);
  let context = compilation.options.context.as_ref();
  let module_graph = compilation.get_module_graph();
  let modules = modules
    .into_iter()
    .filter_map(|identifier| module_graph.module_by_identifier(&identifier))
    .collect::<Vec<_>>();

  let mut modules_with_hashes = modules
    .into_par_iter()
    .map(|module| {
      let name = get_full_module_name(module, context);
      (module, hash_identifier(&name))
    })
    .collect::<Vec<_>>();

  modules_with_hashes.sort_unstable_by(|(a, _), (b, _)| {
    compare_modules_by_pre_order_index_or_identifier(module_graph, &a.identifier(), &b.identifier())
  });

  let mut id_assigner = CompatHashedIdAssigner::new(self.min_length, used_ids);
  for (module, hash) in modules_with_hashes {
    let Some(module_id) = id_assigner.assign(&hash) else {
      return Err(error!(
        "Unable to assign a unique compat-hashed id to module '{}' after using all {FULL_IDENTIFIER_LENGTH} hash characters",
        module.identifier()
      ));
    };

    ChunkGraph::set_module_id(&mut module_ids_map, module.identifier(), module_id.into());
  }

  *module_ids = module_ids_map;
  Ok(())
}

impl Plugin for CompatHashedModuleIdsPlugin {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compilation_hooks.module_ids.tap(module_ids::new(self));
    Ok(())
  }
}
