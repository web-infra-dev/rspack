use itertools::Itertools;
use rayon::prelude::*;
use rspack_core::{
  Compilation, CompilationOptimizeDependencies, ExportProvided, ExportsInfo, ExportsInfoGetter,
  Plugin, PrefetchExportsInfoMode, SideEffectsOptimizeArtifact, UsageState, UsedNameItem,
  incremental::IncrementalPasses,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::{FxHashMap, FxHashSet};

#[plugin]
#[derive(Debug, Default)]
pub struct InlineExportsPlugin;

// We put it to optimize_dependencies hook instead of optimize_code_generation hook like MangleExportsPlugin
// because inline can affect side effects optimization (not the SideEffectsFlagPlugin does, the buildChunkGraph
// does), buildChunkGraph can use dependency condition to determine if a dependency still active, if the dependency
// imported export is inlined, then the dependency is inactive and will not be processed by buildChunkGraph, if a
// module's all exports are all being inlined, then the module can be eliminated by buildChunkGraph
#[plugin_hook(CompilationOptimizeDependencies for InlineExportsPlugin, stage = 100)]
async fn optimize_dependencies(
  &self,
  compilation: &mut Compilation,
  _side_effect_optimize_artifact: &mut SideEffectsOptimizeArtifact,
  diagnostics: &mut Vec<Diagnostic>,
) -> Result<Option<bool>> {
  if let Some(diagnostic) = compilation.incremental.disable_passes(
    IncrementalPasses::MODULES_HASHES,
    "InlineExportsPlugin (optimization.inlineExports = true)",
    "it requires calculating the export names of all the modules, which is a global effect",
  ) {
    diagnostics.extend(diagnostic);
    compilation.cgm_hash_artifact.clear();
  }

  let mg = compilation.get_module_graph_mut();
  let modules = mg.modules();

  let mut visited: FxHashSet<ExportsInfo> = FxHashSet::default();

  let mut q = modules
    .keys()
    .filter_map(|mid| {
      let mgm = mg.module_graph_module_by_identifier(mid)?;
      Some(mgm.exports)
    })
    .collect_vec();

  while !q.is_empty() {
    let items = std::mem::take(&mut q);
    let batch = items
      .par_iter()
      .filter_map(|exports_info| {
        let exports_info_data =
          ExportsInfoGetter::prefetch(exports_info, mg, PrefetchExportsInfoMode::Default);
        let export_list = {
          // If there are other usage (e.g. `import { Kind } from './enum'; Kind;`) in any runtime,
          // then we cannot inline this export.
          if exports_info_data.other_exports_info().get_used(None) != UsageState::Unused {
            return None;
          }
          exports_info_data
            .exports()
            .map(|(_, export_info_data)| {
              let do_inline = !export_info_data.has_used_name()
                && export_info_data.can_inline() == Some(true)
                && matches!(export_info_data.provided(), Some(ExportProvided::Provided));

              let nested_exports_info = if export_info_data.exports_info_owned() {
                let used = export_info_data.get_used(None);
                if used == UsageState::OnlyPropertiesUsed || used == UsageState::Unused {
                  export_info_data.exports_info()
                } else {
                  None
                }
              } else {
                None
              };
              (export_info_data.id(), nested_exports_info, do_inline)
            })
            .collect::<Vec<_>>()
        };

        Some((*exports_info, export_list))
      })
      .collect::<FxHashMap<_, _>>();

    visited.extend(batch.keys());
    for (_, export_list) in batch {
      q.extend(
        export_list
          .into_iter()
          .filter_map(|(export_info, nested_exports_info, do_inline)| {
            if do_inline {
              let data = export_info.as_data_mut(mg);
              data.set_used_name(UsedNameItem::Inlined(
                data
                  .can_inline_provide()
                  .expect("should have provided inline value")
                  .clone(),
              ));
            }
            nested_exports_info
          })
          .filter(|e| !visited.contains(e)),
      );
    }
  }

  Ok(None)
}

impl Plugin for InlineExportsPlugin {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .optimize_dependencies
      .tap(optimize_dependencies::new(self));
    Ok(())
  }
}
