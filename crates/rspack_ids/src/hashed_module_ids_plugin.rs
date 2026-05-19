use std::hash::Hasher;

use rspack_core::{
  ChunkGraph, Compilation, CompilationModuleIds, ModuleIdsArtifact, Plugin,
  incremental::IncrementalPasses,
};
use rspack_error::{Diagnostic, Result};
use rspack_hash::{HashDigest, HashFunction, RspackHash};
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::FxHashSet;

use crate::id_helpers::{
  compare_modules_by_pre_order_index_or_identifier, get_full_module_name,
  get_used_module_ids_and_modules_with_artifact,
};

#[derive(Debug, Clone)]
pub struct HashedModuleIdsPluginOptions {
  pub context: Option<String>,
  pub hash_function: HashFunction,
  pub hash_digest: HashDigest,
  pub hash_digest_length: usize,
}

impl Default for HashedModuleIdsPluginOptions {
  fn default() -> Self {
    Self {
      context: None,
      hash_function: HashFunction::MD4,
      hash_digest: HashDigest::Base64,
      hash_digest_length: 4,
    }
  }
}

#[plugin]
#[derive(Debug)]
pub struct HashedModuleIdsPlugin {
  context: Option<String>,
  hash_function: HashFunction,
  hash_digest: HashDigest,
  hash_digest_length: usize,
}

impl HashedModuleIdsPlugin {
  pub fn new(options: HashedModuleIdsPluginOptions) -> Self {
    Self::new_inner(
      options.context,
      options.hash_function,
      options.hash_digest,
      options.hash_digest_length,
    )
  }
}

#[plugin_hook(CompilationModuleIds for HashedModuleIdsPlugin)]
async fn module_ids(
  &self,
  compilation: &Compilation,
  module_ids: &mut ModuleIdsArtifact,
  diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
  if let Some(diagnostic) = compilation.incremental.disable_passes(
    IncrementalPasses::MODULE_IDS,
    "HashedModuleIdsPlugin",
    "it requires calculating the id of all the modules, which is a global effect",
  ) {
    if let Some(diagnostic) = diagnostic {
      diagnostics.push(diagnostic);
    }
    module_ids.clear();
  }

  let context = self
    .context
    .as_deref()
    .unwrap_or(compilation.options.context.as_ref());

  let (used_ids, mut modules) =
    get_used_module_ids_and_modules_with_artifact(compilation, module_ids, None);
  let mut used_ids: FxHashSet<String> = used_ids;

  let mut module_ids_map = std::mem::take(module_ids);
  let module_graph = compilation.get_module_graph();

  modules
    .sort_unstable_by(|a, b| compare_modules_by_pre_order_index_or_identifier(module_graph, a, b));

  for module_identifier in modules {
    let Some(module) = module_graph.module_by_identifier(&module_identifier) else {
      continue;
    };
    let ident = get_full_module_name(module, context);

    let mut hasher = RspackHash::new(&self.hash_function);
    hasher.write(ident.as_bytes());
    let hash_digest = hasher.digest(&self.hash_digest);
    let hash_id = hash_digest.encoded();

    let mut len = self.hash_digest_length;
    while used_ids.contains(&hash_id[..len.min(hash_id.len())]) && len < hash_id.len() {
      len += 1;
    }

    let module_id = &hash_id[..len.min(hash_id.len())];
    ChunkGraph::set_module_id(&mut module_ids_map, module_identifier, module_id.into());
    used_ids.insert(module_id.to_string());
  }

  *module_ids = module_ids_map;

  Ok(())
}

impl Plugin for HashedModuleIdsPlugin {
  fn name(&self) -> &'static str {
    "HashedModuleIdsPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compilation_hooks.module_ids.tap(module_ids::new(self));
    Ok(())
  }
}
