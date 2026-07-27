use derive_more::Debug;
use rayon::prelude::*;
use rspack_core::{
  ChunkGraph, Compilation, CompilationModuleIds, ModuleIdsArtifact, Plugin,
  incremental::IncrementalPasses,
};
use rspack_error::{Diagnostic, Result, error};
use rspack_hash::{HashFunction, RspackHasher};
use rspack_hook::{plugin, plugin_hook};

use crate::{
  compact_id::{FULL_IDENTIFIER_LENGTH, encode_identifier_hash},
  id_helpers::{
    compare_modules_by_pre_order_index_or_identifier, get_full_module_name,
    get_used_module_ids_and_modules_with_artifact,
  },
};

#[derive(Debug, Clone, Default)]
pub struct CompactModuleIdsPluginOptions {
  pub min_length: Option<usize>,
}

#[plugin]
#[derive(Debug)]
pub struct CompactModuleIdsPlugin {
  min_length: usize,
}

impl Default for CompactModuleIdsPlugin {
  fn default() -> Self {
    Self::new(Default::default())
  }
}

impl CompactModuleIdsPlugin {
  pub fn new(options: CompactModuleIdsPluginOptions) -> Self {
    Self::new_inner(
      options
        .min_length
        .filter(|min_length| *min_length != 0)
        .unwrap_or(1),
    )
  }
}

#[plugin_hook(CompilationModuleIds for CompactModuleIdsPlugin)]
async fn module_ids(
  &self,
  compilation: &Compilation,
  module_ids: &mut ModuleIdsArtifact,
  preserved_module_ids: &ModuleIdsArtifact,
  diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
  if let Some(diagnostic) = compilation.incremental.disable_passes(
    IncrementalPasses::MODULE_IDS | IncrementalPasses::MODULES_HASHES,
    "CompactModuleIdsPlugin (optimization.moduleIds = \"compact\")",
    "it requires calculating the id of all the modules, which is a global effect",
  ) {
    if let Some(diagnostic) = diagnostic {
      diagnostics.push(diagnostic);
    }
    module_ids.retain(|module, _| preserved_module_ids.contains_key(module));
  }

  if self.min_length > FULL_IDENTIFIER_LENGTH {
    return Err(error!(
      "'minLength' must not exceed {FULL_IDENTIFIER_LENGTH} for CompactModuleIdsPlugin"
    ));
  }

  let (mut used_ids, modules) =
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
      let mut hasher = RspackHasher::new(&HashFunction::Xxhash64);
      hasher.write(name.as_bytes());
      (module, encode_identifier_hash(hasher.finish()))
    })
    .collect::<Vec<_>>();

  modules_with_hashes.sort_unstable_by(|(a, _), (b, _)| {
    compare_modules_by_pre_order_index_or_identifier(module_graph, &a.identifier(), &b.identifier())
  });

  for (module, hash) in modules_with_hashes {
    // SAFETY: `encode_identifier_hash` only emits ASCII characters.
    let hash = unsafe { std::str::from_utf8_unchecked(&hash) };
    let Some(module_id) = (self.min_length..=hash.len()).find_map(|length| {
      let candidate = &hash[..length];
      if used_ids.contains(candidate) {
        None
      } else {
        Some(candidate.to_string())
      }
    }) else {
      return Err(error!(
        "Unable to assign a unique compact id to module '{}' after using all {FULL_IDENTIFIER_LENGTH} hash characters",
        module.identifier()
      ));
    };

    used_ids.insert(module_id.clone());
    ChunkGraph::set_module_id(&mut module_ids_map, module.identifier(), module_id.into());
  }

  *module_ids = module_ids_map;
  Ok(())
}

impl Plugin for CompactModuleIdsPlugin {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compilation_hooks.module_ids.tap(module_ids::new(self));
    Ok(())
  }
}
