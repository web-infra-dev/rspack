use std::sync::Arc;

use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  BuildModuleGraphArtifact, Compilation, DependenciesBlock, Dependency, DependencyId,
  DependencyTemplate, ExportsInfoArtifact, ExternalModule, InitFragmentExt, InitFragmentKey,
  InitFragmentStage, NormalInitFragment,
};
use rspack_plugin_javascript::dependency::{
  ESMExportImportedSpecifierDependency, ESMImportSideEffectDependency, ImportDependency,
};
use rustc_hash::FxHashSet;

fn has_unknown(exports: &rspack_core::ExportNameOrSpec) -> bool {
  match exports {
    rspack_core::ExportNameOrSpec::String(_) => false,
    rspack_core::ExportNameOrSpec::ExportSpec(export_spec) => {
      if let Some(exports) = &export_spec.exports {
        if exports.unknown_provided {
          return true;
        }

        exports.exports.iter().any(has_unknown)
      } else {
        false
      }
    }
  }
}

pub fn cutout_star_re_export_externals(
  compilation: &Compilation,
  build_module_graph_artifact: &mut BuildModuleGraphArtifact,
  exports_info_artifact: &mut ExportsInfoArtifact,
) {
  let mg = build_module_graph_artifact.get_module_graph();
  let mut connections_to_disable = FxHashSet::default();

  let entry_modules = build_module_graph_artifact
    .entry_dependencies
    .iter()
    .filter_map(|dep_id| mg.module_identifier_by_dependency_id(dep_id))
    .copied()
    .collect::<IdentifierSet>();

  for module_id in &entry_modules {
    let module = mg
      .module_by_identifier(module_id)
      .expect("should have entry module");
    let mut side_effects_deps_by_module = IdentifierMap::<Vec<DependencyId>>::default();
    let mut star_export_deps_by_module = IdentifierMap::default();

    for dep_id in module.get_dependencies() {
      let dep = mg.dependency_by_id(dep_id);
      let Some(module) = mg
        .module_identifier_by_dependency_id(dep_id)
        .and_then(|module_id| mg.module_by_identifier(module_id))
      else {
        continue;
      };

      let Some(external_module) = module.as_external_module() else {
        continue;
      };
      let external_type = external_module.get_external_type().as_str();
      if !external_type.starts_with("module") {
        continue;
      }

      if dep.as_any().is::<ESMImportSideEffectDependency>() {
        side_effects_deps_by_module
          .entry(external_module.id)
          .or_default()
          .push(*dep_id);
      } else if let Some(esm_export_dep) = dep
        .as_any()
        .downcast_ref::<ESMExportImportedSpecifierDependency>()
        && esm_export_dep.name.is_none()
        && esm_export_dep.other_star_exports.is_some()
      {
        *star_export_deps_by_module
          .entry(external_module.id)
          .or_insert(0) += 1;

        connections_to_disable.insert(*dep_id);
      }
    }

    for (module_id, side_effect_deps) in side_effects_deps_by_module {
      let star_count = star_export_deps_by_module.get(&module_id).unwrap_or(&0);
      if side_effect_deps.len() == *star_count {
        connections_to_disable.extend(side_effect_deps);
      }
    }
  }

  let mg = build_module_graph_artifact.get_module_graph_mut();
  for dep_id in &connections_to_disable {
    let conn = mg
      .connection_by_dependency_id_mut(dep_id)
      .expect("definitely has connection");
    conn.force_inactive();
  }

  // correct exports info
  for module_id in entry_modules {
    let module = mg
      .module_by_identifier(&module_id)
      .expect("should have entry module");

    if !module.build_meta().esm {
      continue;
    }

    let exports_info = exports_info_artifact.get_exports_info_data(&module_id);

    // re check
    if matches!(
      exports_info.other_exports_info().provided(),
      Some(rspack_core::ExportProvided::Unknown)
    ) {
      let has_unknown_exports = module.get_dependencies().iter().any(|dep_id| {
        if connections_to_disable.contains(dep_id) {
          return false;
        }

        let dep = mg.dependency_by_id(dep_id);
        let Some(exports) = dep.get_exports(
          mg,
          &compilation.module_graph_cache_artifact,
          exports_info_artifact,
        ) else {
          return false;
        };

        match &exports.exports {
          rspack_core::ExportsOfExportsSpec::UnknownExports => true,
          rspack_core::ExportsOfExportsSpec::NoExports => false,
          rspack_core::ExportsOfExportsSpec::Names(export_name_or_specs) => {
            export_name_or_specs.iter().any(has_unknown)
          }
        }
      });

      if !has_unknown_exports {
        let exports_info = exports_info_artifact.get_exports_info_data_mut(&module_id);
        exports_info
          .other_exports_info_mut()
          .set_provided(Some(rspack_core::ExportProvided::NotProvided));
      }
    }
  }
}

pub fn cutout_dyn_import_externals(build_module_graph_artifact: &mut BuildModuleGraphArtifact) {
  let mg = build_module_graph_artifact.get_module_graph();
  let mut connections_to_disable = Vec::new();
  for (_, module) in mg.modules() {
    for block_id in module.get_blocks() {
      let Some(block) = mg.block_by_id(block_id) else {
        continue;
      };
      for block_dep_id in block.get_dependencies() {
        let block_dep = mg.dependency_by_id(block_dep_id);
        if block_dep.as_any().is::<ImportDependency>() {
          let import_dep_connection = mg.connection_by_dependency_id(block_dep_id);
          if let Some(import_dep_connection) = import_dep_connection {
            // Try find the connection with a import dependency pointing to an external module.
            // If found, record the dependency so its connection can be disabled (forced inactive) and handled by external dyn-import rendering.
            let import_module_id = import_dep_connection.module_identifier();
            let Some(import_module) = mg.module_by_identifier(import_module_id) else {
              continue;
            };

            if import_module.as_external_module().is_some() {
              // remove connection of dyn-import external module
              connections_to_disable.push(*block_dep_id);
            }
          }
        }
      }
    }
  }

  let mg = build_module_graph_artifact.get_module_graph_mut();
  for dep_id in connections_to_disable {
    let conn = mg
      .connection_by_dependency_id_mut(&dep_id)
      .expect("definitely has connection");
    conn.force_inactive();
  }
}

pub fn render_dyn_import_external_module(
  import_dep: &ImportDependency,
  external_module: &ExternalModule,
  source: &mut rspack_core::TemplateReplaceSource,
) {
  let request = external_module.get_request();
  let attributes_str = if let Some(attributes) = import_dep.get_attributes() {
    format!(
      ", {{ with: {} }}",
      serde_json::to_string(attributes).expect("invalid json to_string")
    )
  } else {
    String::new()
  };
  let comments_str = {
    let mut comments_string = String::new();

    for (line_comment, comment) in import_dep.comments.iter() {
      if *line_comment {
        comments_string.push_str(&format!("//{comment}\n"));
      } else {
        comments_string.push_str(&format!("/*{comment}*/ "));
      }
    }

    comments_string
  };

  source.replace(
    import_dep.range.start,
    import_dep.range.end,
    &format!(
      "import({}{}{})",
      comments_str,
      serde_json::to_string(&request.primary).expect("should be valid json"),
      attributes_str
    ),
    None,
  );
}

#[derive(Debug)]
pub(crate) struct ImportDependencyTemplate {
  pub(crate) template: Option<Arc<dyn DependencyTemplate>>,
}

impl DependencyTemplate for ImportDependencyTemplate {
  fn render(
    &self,
    dep: &dyn rspack_core::DependencyCodeGeneration,
    source: &mut rspack_core::rspack_sources::ReplaceSource,
    code_generatable_context: &mut rspack_core::TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ImportDependency>()
      .expect("should be import dependency");
    let mg = code_generatable_context.compilation.get_module_graph();
    let ref_module = mg
      .module_identifier_by_dependency_id(&dep.id)
      .and_then(|module_id| mg.module_by_identifier(module_id));
    if let Some(external) = ref_module.and_then(|m| m.as_external_module()) {
      render_dyn_import_external_module(dep, external, source);
      return;
    }

    if let Some(template) = &self.template {
      template.render(dep, source, code_generatable_context);
    }
  }
}

#[derive(Debug)]
pub(crate) struct ExportImportedDependencyTemplate {
  pub(crate) template: Option<Arc<dyn DependencyTemplate>>,
}

impl DependencyTemplate for ExportImportedDependencyTemplate {
  fn render(
    &self,
    dep: &dyn rspack_core::DependencyCodeGeneration,
    source: &mut rspack_core::rspack_sources::ReplaceSource,
    code_generatable_context: &mut rspack_core::TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ESMExportImportedSpecifierDependency>()
      .expect("should be export imported dependency");
    let mg = code_generatable_context.compilation.get_module_graph();
    let module = mg.module_identifier_by_dependency_id(&dep.id);

    if dep.name.is_none()
      && dep.other_star_exports.is_some()
      && let Some(module) = module
        .and_then(|mid| mg.module_by_identifier(mid))
        .and_then(|m| {
          m.as_external_module().filter(|&m| m.get_external_type().starts_with("module"))
        })
      // TODO: should cache calculate results for this
      && code_generatable_context
        .compilation
        .entry_modules()
        .contains(&code_generatable_context.module.identifier())
    {
      let request = module.get_request();
      let chunk_init_fragments = code_generatable_context.chunk_init_fragments();
      chunk_init_fragments.push(
        NormalInitFragment::new(
          format!(
            "export * from {};\n",
            serde_json::to_string(request.primary()).expect("invalid json to_string")
          ),
          InitFragmentStage::StageESMImports,
          0,
          InitFragmentKey::Const(format!("esm_module_reexport_star_{}", dep.request)),
          None,
        )
        .boxed(),
      );
      return;
    }

    if let Some(template) = &self.template {
      template.render(dep, source, code_generatable_context);
    }
  }
}
