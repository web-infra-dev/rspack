use std::sync::atomic::Ordering;

use rspack_error::Result;
use rspack_util::tracing_preset::TRACING_BENCH_TARGET;
use tracing::instrument;

use crate::{
  BuildMetaExportsType, Compilation, DependencyType, ModuleGraph, ModuleType,
  compilation::build_module_graph::finish_build_module_graph, logger::Logger,
};

fn is_static_esm_dependency(dependency_type: &DependencyType) -> bool {
  matches!(
    dependency_type,
    DependencyType::EsmImport
      | DependencyType::EsmImportSpecifier
      | DependencyType::EsmExportImport
      | DependencyType::EsmExportImportedSpecifier
  )
}

fn can_define_runtime_exports(dependency_type: &DependencyType) -> bool {
  matches!(
    dependency_type,
    DependencyType::CjsExports
      | DependencyType::CjsExportRequire
      | DependencyType::CjsSelfReference
      | DependencyType::AmdDefine
      | DependencyType::StaticExports
      | DependencyType::ModuleDecorator
  )
}

fn infer_js_auto_modules_without_exports_as_esm(module_graph: &mut ModuleGraph) {
  let inferred_states = module_graph
    .modules()
    .filter_map(|(identifier, module)| {
      (matches!(module.module_type(), ModuleType::JsAuto)
        && module.build_meta().exports_type == BuildMetaExportsType::Unset)
        .then(|| {
          let has_runtime_exports = module.get_dependencies().iter().any(|dependency_id| {
            can_define_runtime_exports(
              module_graph
                .dependency_by_id(dependency_id)
                .dependency_type(),
            )
          });
          let inferred = module_graph
            .get_incoming_connections(identifier)
            .map(|connection| module_graph.dependency_by_id(&connection.dependency_id))
            .any(|dependency| is_static_esm_dependency(dependency.dependency_type()))
            && !has_runtime_exports;
          (*identifier, inferred)
        })
    })
    .collect::<Vec<_>>();

  for (identifier, inferred) in inferred_states {
    module_graph
      .module_by_identifier_mut(&identifier)
      .expect("javascript/auto module without exports should exist")
      .build_meta_mut()
      .set_inferred_js_auto_esm(inferred);
  }
}

pub async fn finish_module_graph_pass(compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compiler");
  let start = logger.time("finish compilation");
  finish_build_module_graph_pass(compilation).await?;
  logger.time_end(start);

  Ok(())
}

#[instrument("Compilation:finish",target=TRACING_BENCH_TARGET, skip_all)]
pub async fn finish_build_module_graph_pass(compilation: &mut Compilation) -> Result<()> {
  compilation.in_finish_make.store(false, Ordering::Release);
  // clean up the entry deps
  let make_artifact = compilation.build_module_graph_artifact.steal();
  let exports_info_artifact = compilation.exports_info_artifact.steal();
  let (make_artifact, exports_info_artifact) =
    finish_build_module_graph(compilation, make_artifact, exports_info_artifact).await?;
  compilation.build_module_graph_artifact = make_artifact.into();
  compilation.exports_info_artifact = exports_info_artifact.into();
  // sync assets to module graph from module_executor
  if let Some(module_executor) = &mut compilation.module_executor {
    let mut module_executor = std::mem::take(module_executor);
    module_executor
      .after_build_module_graph(compilation)
      .await?;
    compilation.module_executor = Some(module_executor);
  }
  infer_js_auto_modules_without_exports_as_esm(compilation.get_module_graph_mut());
  // make finished, make artifact should be readonly thereafter.
  Ok(())
}
