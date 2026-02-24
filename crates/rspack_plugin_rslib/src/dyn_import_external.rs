use std::sync::Arc;

use rspack_core::{
  BuildModuleGraphArtifact, DependenciesBlock, Dependency, DependencyTemplate, ExternalModule,
};
use rspack_plugin_javascript::dependency::ImportDependency;

pub fn cutout_dyn_import_external(build_module_graph_artifact: &mut BuildModuleGraphArtifact) {
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
