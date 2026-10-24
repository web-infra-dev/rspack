use derive_more::Debug;
use rayon::prelude::*;
use rspack_core::{
  ChunkGraph, Compilation, CompilationModuleIds, DeterministicModuleIdsInputs, ModuleId,
  ModuleIdsArtifact, Plugin, incremental::IncrementalPasses,
};
use rspack_error::{Diagnostic, Result, error};
use rspack_hook::{plugin, plugin_hook};
use rspack_util::number_hash::{get_number_hash_combined_from_state, get_number_hash_state};
use rustc_hash::FxHashSet;

use crate::id_helpers::{
  ModuleFilterFn, assign_deterministic_ids_with_hash, compare_by_pre_order_index_or_id,
  get_deterministic_id_range, get_full_module_name, get_used_module_ids_and_modules_with_artifact,
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
  preserved_module_ids: &ModuleIdsArtifact,
  diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
  let hooks = &compilation.plugin_driver.compilation_hooks;
  // Only skip when the inputs below capture everything that affects the ids:
  // incremental module ids on, this plugin the only moduleIds tap, no `test`
  // function, and no ids assigned earlier by other hooks.
  let can_reuse = self.test.is_none()
    && preserved_module_ids.is_empty()
    && hooks.module_ids.tap_stages().len() == 1
    && !hooks.module_ids.has_interceptors()
    && compilation
      .incremental
      .passes_enabled(IncrementalPasses::MODULE_IDS);
  let inputs = can_reuse.then(|| {
    let context = self
      .context
      .as_deref()
      .unwrap_or(compilation.options.context.as_ref());
    // Collect candidates as if no module had an id yet, so the inputs describe
    // the whole assignment, not just modules added since the last build.
    let (used_ids, modules) =
      get_used_module_ids_and_modules_with_artifact(compilation, preserved_module_ids, None);
    let mg = compilation.get_module_graph();
    let mut candidates = modules
      .into_par_iter()
      .map(|identifier| {
        let module = mg
          .module_by_identifier(&identifier)
          .expect("should have module");
        (
          identifier,
          get_full_module_name(module, context),
          mg.get_pre_order_index(&identifier),
        )
      })
      .collect::<Vec<_>>();
    candidates.sort_unstable_by(|(a, _, a_index), (b, _, b_index)| {
      compare_by_pre_order_index_or_id(*a_index, a, *b_index, b)
    });
    let ranges = [10usize
      .checked_pow(self.max_length as u32)
      .unwrap_or(usize::MAX)];
    let range = get_deterministic_id_range(
      candidates.len(),
      &ranges,
      if self.fixed_length { 0 } else { 10 },
      used_ids.len(),
    );
    let mut reserved_ids = used_ids.into_iter().map(ModuleId::from).collect::<Vec<_>>();
    reserved_ids.sort_unstable();
    DeterministicModuleIdsInputs {
      candidates,
      reserved_ids,
      context: context.to_owned(),
      range,
      max_length: self.max_length,
      salt: self.salt,
      fixed_length: self.fixed_length,
      fail_on_conflict: self.fail_on_conflict,
    }
  });
  {
    let mut previous = compilation
      .module_ids_diff_artifact
      .deterministic_inputs
      .lock()
      .expect("Mutex poisoned: deterministic module ID inputs");
    // Same inputs, and every candidate still has its id: nothing to do.
    if let Some(inputs) = &inputs
      && compilation
        .incremental
        .mutations_readable(IncrementalPasses::MODULE_IDS)
      && previous.as_ref() == Some(inputs)
      && module_ids.len() == inputs.candidates.len()
      && inputs
        .candidates
        .iter()
        .all(|(identifier, _, _)| module_ids.contains_key(identifier))
    {
      return Ok(());
    }
    // Cleared until the assignment succeeds, so a failed one is never skipped.
    *previous = None;
  }

  if let Some(diagnostic) = compilation.incremental.disable_passes(
    IncrementalPasses::MODULE_IDS,
    "DeterministicModuleIdsPlugin (optimization.moduleIds = \"deterministic\")",
    "it requires calculating the id of all the modules, which is a global effect",
  ) {
    if let Some(diagnostic) = diagnostic {
      diagnostics.push(diagnostic);
    }
    module_ids.retain(|module, _| preserved_module_ids.contains_key(module));
  }

  let mut conflicts = 0;
  let ranges = [10usize
    .checked_pow(self.max_length as u32)
    .unwrap_or(usize::MAX)];
  let expand_factor = if self.fixed_length { 0 } else { 10 };

  let (mut used_module_ids, modules_with_hashes) = if let Some(inputs) = &inputs {
    // Reuse the candidates and reserved ids collected above.
    debug_assert_eq!(
      inputs.range,
      get_deterministic_id_range(
        inputs.candidates.len(),
        &ranges,
        expand_factor,
        inputs.reserved_ids.len(),
      )
    );
    let mut used_module_ids = FxHashSet::with_capacity_and_hasher(
      inputs.reserved_ids.len() + inputs.candidates.len(),
      Default::default(),
    );
    used_module_ids.extend(inputs.reserved_ids.iter().cloned());
    let modules_with_hashes = inputs
      .candidates
      .par_iter()
      .map(|(identifier, full_name, pre_order_index)| {
        (
          *identifier,
          *pre_order_index,
          get_number_hash_state(full_name, inputs.range),
        )
      })
      .collect::<Vec<_>>();
    (used_module_ids, modules_with_hashes)
  } else {
    // Setups that can't skip collect candidates as before.
    let (used_ids, modules) = if self.test.is_some() {
      get_used_module_ids_and_modules_with_async_filter(compilation, module_ids, self.test.as_ref())
        .await?
    } else {
      get_used_module_ids_and_modules_with_artifact(compilation, module_ids, None)
    };
    let context = self
      .context
      .as_deref()
      .unwrap_or(compilation.options.context.as_ref());
    let module_graph = compilation.get_module_graph();
    let modules = modules
      .into_iter()
      .filter_map(|identifier| {
        module_graph
          .module_by_identifier(&identifier)
          .map(|module| {
            (
              identifier,
              module,
              module_graph.get_pre_order_index(&identifier),
            )
          })
      })
      .collect::<Vec<_>>();
    let range = get_deterministic_id_range(modules.len(), &ranges, expand_factor, used_ids.len());
    let mut used_module_ids =
      FxHashSet::with_capacity_and_hasher(used_ids.len() + modules.len(), Default::default());
    used_module_ids.extend(used_ids.into_iter().map(ModuleId::from));
    let modules_with_hashes = modules
      .into_par_iter()
      .map(|(identifier, module, pre_order_index)| {
        let full_name = get_full_module_name(module, context);
        (
          identifier,
          pre_order_index,
          get_number_hash_state(&full_name, range),
        )
      })
      .collect::<Vec<_>>();
    (used_module_ids, modules_with_hashes)
  };
  let mut module_ids_map = std::mem::take(module_ids);
  module_ids_map.reserve(modules_with_hashes.len());

  assign_deterministic_ids_with_hash(
    modules_with_hashes,
    |(a_identifier, a_pre_order_index, _), (b_identifier, b_pre_order_index, _)| {
      compare_by_pre_order_index_or_id(
        *a_pre_order_index,
        a_identifier,
        *b_pre_order_index,
        b_identifier,
      )
    },
    |(module_identifier, _, _), id| {
      let module_id: ModuleId = id.to_string().into();
      if !used_module_ids.insert(module_id.clone()) {
        conflicts += 1;
        return false;
      }
      ChunkGraph::set_module_id(&mut module_ids_map, *module_identifier, module_id);
      true
    },
    |(_, _, hash_state), suffix| get_number_hash_combined_from_state(*hash_state, suffix),
    self.salt,
  );
  *module_ids = module_ids_map;
  if self.fail_on_conflict && conflicts > 0 {
    return Err(error!(
      "Assigning deterministic module ids has lead to {conflicts} conflict{}.\nIncrease the 'maxLength' to increase the id space and make conflicts less likely (recommended when there are many conflicts or application is expected to grow), or add an 'salt' number to try another hash starting value in the same id space (recommended when there is only a single conflict).",
      if conflicts > 1 { "s" } else { "" }
    ));
  }
  if let Some(inputs) = inputs {
    *compilation
      .module_ids_diff_artifact
      .deterministic_inputs
      .lock()
      .expect("Mutex poisoned: deterministic module ID inputs") = Some(inputs);
  }
  Ok(())
}

impl Plugin for DeterministicModuleIdsPlugin {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compilation_hooks.module_ids.tap(module_ids::new(self));
    Ok(())
  }
}
