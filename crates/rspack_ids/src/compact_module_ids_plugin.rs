use derive_more::Debug;
use rayon::prelude::*;
use rspack_core::{
  ChunkGraph, Compilation, CompilationModuleIds, ModuleIdsArtifact, Plugin,
  incremental::IncrementalPasses,
};
use rspack_error::{Diagnostic, Result, error};
use rspack_hook::{plugin, plugin_hook};

use crate::{
  DeterministicModuleIdsPluginOptions,
  id_helpers::{
    ModuleFilterFn, assign_deterministic_ids, compare_modules_by_pre_order_index_or_identifier,
    get_full_module_name, get_used_module_ids_and_modules_with_artifact,
    get_used_module_ids_and_modules_with_async_filter,
  },
};

const IDENTIFIER_START_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const IDENTIFIER_CONTINUE_CHARS: &[u8] =
  b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

pub type CompactModuleIdsPluginOptions = DeterministicModuleIdsPluginOptions;

// Keep ids valid as unquoted JavaScript property names while using the full
// alphanumeric alphabet after the first character.
fn identifier_space(max_length: usize) -> usize {
  let mut space = 0usize;
  let mut block = IDENTIFIER_START_CHARS.len();
  for _ in 0..max_length {
    space = space.saturating_add(block);
    block = block.saturating_mul(IDENTIFIER_CONTINUE_CHARS.len());
  }
  space
}

fn to_identifier(mut id: usize) -> String {
  let mut length = 1usize;
  let mut block = IDENTIFIER_START_CHARS.len();
  while id >= block {
    id -= block;
    length += 1;
    block = block.saturating_mul(IDENTIFIER_CONTINUE_CHARS.len());
  }

  let mut divisor = IDENTIFIER_CONTINUE_CHARS
    .len()
    .saturating_pow((length - 1) as u32);
  let mut result = String::with_capacity(length);
  result.push(IDENTIFIER_START_CHARS[id / divisor] as char);
  id %= divisor;
  while divisor > 1 {
    divisor /= IDENTIFIER_CONTINUE_CHARS.len();
    result.push(IDENTIFIER_CONTINUE_CHARS[id / divisor] as char);
    id %= divisor;
  }
  result
}

#[plugin]
#[derive(Debug)]
pub struct CompactModuleIdsPlugin {
  context: Option<String>,
  #[debug(skip)]
  test: Option<ModuleFilterFn>,
  max_length: usize,
  salt: usize,
  fixed_length: bool,
  fail_on_conflict: bool,
}

impl Default for CompactModuleIdsPlugin {
  fn default() -> Self {
    Self::new(Default::default())
  }
}

impl CompactModuleIdsPlugin {
  pub fn new(options: CompactModuleIdsPluginOptions) -> Self {
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

  let modules_with_names = modules
    .into_par_iter()
    .map(|m| (m, get_full_module_name(m, context)))
    .collect::<Vec<_>>();

  assign_deterministic_ids(
    modules_with_names,
    |(_, name)| name.as_str(),
    |(a, _), (b, _)| {
      compare_modules_by_pre_order_index_or_identifier(
        module_graph,
        &a.identifier(),
        &b.identifier(),
      )
    },
    |(module, _), id| {
      let id = to_identifier(id);
      if !used_ids.insert(id.clone()) {
        conflicts += 1;
        return false;
      }
      ChunkGraph::set_module_id(&mut module_ids_map, module.identifier(), id.into());
      true
    },
    &[identifier_space(self.max_length)],
    if self.fixed_length {
      0
    } else {
      IDENTIFIER_CONTINUE_CHARS.len()
    },
    used_ids_len,
    self.salt,
  );
  *module_ids = module_ids_map;
  if self.fail_on_conflict && conflicts > 0 {
    return Err(error!(
      "Assigning compact module ids has lead to {conflicts} conflict{}.\nIncrease the 'maxLength' to increase the id space and make conflicts less likely (recommended when there are many conflicts or application is expected to grow), or add an 'salt' number to try another hash starting value in the same id space (recommended when there is only a single conflict).",
      if conflicts > 1 { "s" } else { "" }
    ));
  }
  Ok(())
}

impl Plugin for CompactModuleIdsPlugin {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compilation_hooks.module_ids.tap(module_ids::new(self));
    Ok(())
  }
}
