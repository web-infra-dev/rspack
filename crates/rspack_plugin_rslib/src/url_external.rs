use std::sync::Arc;

use rspack_core::{
  BuildModuleGraphArtifact, Dependency, DependencyCodeGeneration, DependencyTemplate,
  ExternalModule, JavascriptParserUrl, TemplateContext, TemplateReplaceSource,
};
use rspack_plugin_javascript::dependency::URLDependency;
#[cfg(allocative)]
use rspack_util::allocative;

fn should_cutout_url_external(
  cutout_all_externals: bool,
  output_module: bool,
  url_dependency: &URLDependency,
  external_module: &ExternalModule,
) -> bool {
  if !output_module {
    return false;
  }

  if !matches!(
    url_dependency.url_mode(),
    Some(JavascriptParserUrl::NewUrlRelative)
  ) {
    return false;
  }

  let request = &external_module.get_request().primary;
  if !request.starts_with("./") && !request.starts_with("../") {
    return false;
  }

  cutout_all_externals || matches!(external_module.resolve_external_type(), "import" | "module")
}

pub fn cutout_url_externals(
  cutout_all_externals: bool,
  output_module: bool,
  build_module_graph_artifact: &mut BuildModuleGraphArtifact,
) {
  let mg = build_module_graph_artifact.get_module_graph();
  let mut connections_to_disable = Vec::new();

  for (_, module) in mg.modules() {
    for dependency in module.get_dependencies() {
      let Some(url_dependency) = dependency.as_any().downcast_ref::<URLDependency>() else {
        continue;
      };

      let Some(connection) = mg.connection_by_dependency_id(dependency.id()) else {
        continue;
      };
      let Some(module) = mg.module_by_identifier(connection.module_identifier()) else {
        continue;
      };
      let Some(external_module) = module.as_external_module() else {
        continue;
      };

      if should_cutout_url_external(
        cutout_all_externals,
        output_module,
        url_dependency,
        external_module,
      ) {
        connections_to_disable.push(*dependency.id());
      }
    }
  }

  let mg = build_module_graph_artifact.get_module_graph_mut();
  for dep_id in connections_to_disable {
    let connection = mg
      .connection_by_dependency_id_mut(&dep_id)
      .expect("url external should have a module graph connection");
    connection.force_inactive();
  }
}

fn render_url_external_module(
  url_dependency: &URLDependency,
  external_module: &ExternalModule,
  source: &mut TemplateReplaceSource,
) {
  let request = rspack_util::json_stringify_str(&external_module.get_request().primary);
  url_dependency.replace_request(source, request);
}

#[derive(Debug)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct ExternalURLDependencyTemplate {
  pub cutout_all_externals: bool,
  pub template: Option<Arc<dyn DependencyTemplate>>,
}

impl DependencyTemplate for ExternalURLDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let url_dependency = dep
      .as_any()
      .downcast_ref::<URLDependency>()
      .expect("ExternalURLDependencyTemplate should be used for URLDependency");
    let compilation = code_generatable_context.compilation;
    let module_graph = compilation.get_module_graph();
    let external_module = module_graph
      .module_identifier_by_dependency_id(url_dependency.id())
      .and_then(|module_id| module_graph.module_by_identifier(module_id))
      .and_then(|module| module.as_external_module());

    if let Some(external_module) = external_module
      && should_cutout_url_external(
        self.cutout_all_externals,
        compilation.options.output.module,
        url_dependency,
        external_module,
      )
    {
      render_url_external_module(url_dependency, external_module, source);
      return;
    }

    if let Some(template) = &self.template {
      template.render(dep, source, code_generatable_context);
    }
  }
}
