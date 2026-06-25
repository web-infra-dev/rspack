use std::borrow::Cow;

use derive_more::Debug;
use rayon::prelude::*;
use rspack_collections::IdentifierMap;
use rspack_core::{
  ChunkGraph, Compilation, CompilationModuleIds, ModuleIdsArtifact, Plugin,
  incremental::IncrementalPasses,
};
use rspack_error::{Diagnostic, Result, error};
use rspack_hook::{plugin, plugin_hook};

use crate::id_helpers::{
  ModuleFilterFn, assign_deterministic_ids, compare_modules_by_pre_order_index_or_identifier,
  get_full_module_name, get_used_module_ids_and_modules_with_artifact,
  get_used_module_ids_and_modules_with_async_filter,
};

#[derive(Debug, Clone, Default)]
pub struct DeterministicModuleIdsPluginOptions {
  pub context: Option<String>,
  #[debug(skip)]
  pub test: Option<ModuleFilterFn>,
  pub max_length: Option<usize>,
  pub salt: Option<usize>,
  pub fixed_length: Option<bool>,
  pub fail_on_conflict: Option<bool>,
}

#[plugin]
#[derive(Debug)]
pub struct DeterministicModuleIdsPlugin {
  context: Option<String>,
  #[debug(skip)]
  test: Option<ModuleFilterFn>,
  max_length: usize,
  salt: usize,
  fixed_length: bool,
  fail_on_conflict: bool,
}

impl Default for DeterministicModuleIdsPlugin {
  fn default() -> Self {
    Self::new(Default::default())
  }
}

impl DeterministicModuleIdsPlugin {
  pub fn new(options: DeterministicModuleIdsPluginOptions) -> Self {
    Self::new_inner(
      options.context,
      options.test,
      options
        .max_length
        .filter(|max_length| *max_length != 0)
        .unwrap_or(3),
      options.salt.unwrap_or_default(),
      options.fixed_length.unwrap_or_default(),
      options.fail_on_conflict.unwrap_or_default(),
    )
  }
}

#[plugin_hook(CompilationModuleIds for DeterministicModuleIdsPlugin)]
async fn module_ids(
  &self,
  compilation: &Compilation,
  module_ids: &mut ModuleIdsArtifact,
  diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
  if let Some(diagnostic) = compilation.incremental.disable_passes(
    IncrementalPasses::MODULE_IDS,
    "DeterministicModuleIdsPlugin (optimization.moduleIds = \"deterministic\")",
    "it requires calculating the id of all the modules, which is a global effect",
  ) {
    if let Some(diagnostic) = diagnostic {
      diagnostics.push(diagnostic);
    }
    module_ids.clear();
  }

  // Use the sync path when no async test filter is provided (the common case),
  // avoiding unnecessary async overhead on the hot path.
  let (mut used_ids, modules) = if self.test.is_some() {
    get_used_module_ids_and_modules_with_async_filter(compilation, module_ids, self.test.as_ref())
      .await?
  } else {
    get_used_module_ids_and_modules_with_artifact(compilation, module_ids, None)
  };

  let mut module_ids_map = std::mem::take(module_ids);
  let context = self
    .context
    .as_deref()
    .unwrap_or(compilation.options.context.as_ref());
  let mut conflicts = 0;

  let module_graph = compilation.get_module_graph();
  let modules = modules
    .into_iter()
    .filter_map(|i| module_graph.module_by_identifier(&i))
    .collect::<Vec<_>>();
  let used_ids_len = used_ids.len();

  let module_names = modules
    .par_iter()
    .map(|m| (m.identifier(), get_full_module_name(m, context)))
    .collect::<IdentifierMap<String>>();

  assign_deterministic_ids(
    modules,
    |m| {
      Cow::Borrowed(
        module_names
          .get(&m.identifier())
          .expect("should have generated full module name")
          .as_str(),
      )
    },
    |a, b| {
      compare_modules_by_pre_order_index_or_identifier(
        module_graph,
        &a.identifier(),
        &b.identifier(),
      )
    },
    |module, id| {
      if !used_ids.insert(id.to_string()) {
        conflicts += 1;
        return false;
      }
      ChunkGraph::set_module_id(
        &mut module_ids_map,
        module.identifier(),
        id.to_string().into(),
      );
      true
    },
    &[10usize
      .checked_pow(self.max_length as u32)
      .unwrap_or(usize::MAX)],
    if self.fixed_length { 0 } else { 10 },
    used_ids_len,
    self.salt,
  );
  *module_ids = module_ids_map;
  if self.fail_on_conflict && conflicts > 0 {
    return Err(error!(
      "Assigning deterministic module ids has lead to {conflicts} conflict{}.\nIncrease the 'maxLength' to increase the id space and make conflicts less likely (recommended when there are many conflicts or application is expected to grow), or add an 'salt' number to try another hash starting value in the same id space (recommended when there is only a single conflict).",
      if conflicts > 1 { "s" } else { "" }
    ));
  }
  Ok(())
}

impl Plugin for DeterministicModuleIdsPlugin {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compilation_hooks.module_ids.tap(module_ids::new(self));
    Ok(())
  }
}
