use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  DependencyCodeGeneration, DependencyId, DependencyLocation, DependencyRange, DependencyTemplate,
  DependencyTemplateType, ImportPhase, RuntimeCondition, TemplateContext, TemplateReplaceSource,
};

use crate::dependency::import_emitted_runtime;

#[cacheable]
#[derive(Debug, Clone)]
pub struct ImportMetaHotAcceptRefreshDependency {
  range: DependencyRange,
  dependency_ids: Vec<DependencyId>,
  loc: Option<DependencyLocation>,
}

impl ImportMetaHotAcceptRefreshDependency {
  pub fn new(
    range: DependencyRange,
    dependency_ids: Vec<DependencyId>,
    loc: Option<DependencyLocation>,
  ) -> Self {
    Self {
      range,
      dependency_ids,
      loc,
    }
  }

  pub fn loc(&self) -> Option<DependencyLocation> {
    self.loc.clone()
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for ImportMetaHotAcceptRefreshDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ImportMetaHotAcceptRefreshDependencyTemplate::template_type())
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ImportMetaHotAcceptRefreshDependencyTemplate;

impl ImportMetaHotAcceptRefreshDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("ImportMetaHotAcceptRefreshDependency")
  }
}

impl DependencyTemplate for ImportMetaHotAcceptRefreshDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ImportMetaHotAcceptRefreshDependency>()
      .expect(
        "ImportMetaHotAcceptRefreshDependencyTemplate should be used for ImportMetaHotAcceptRefreshDependency",
      );

    let TemplateContext {
      compilation,
      module,
      runtime,
      runtime_template,
      ..
    } = code_generatable_context;

    let mut content = String::default();
    let module_graph = compilation.get_module_graph();
    let module_identifier = module.identifier();
    dep.dependency_ids.iter().for_each(|id| {
      let dependency = module_graph.dependency_by_id(id);
      let target_module = module_graph.get_module_by_dependency_id(dependency.id());
      let runtime_condition = match target_module {
        Some(target_module) => {
          import_emitted_runtime::get_runtime(&module_identifier, &target_module.identifier())
        }
        None => RuntimeCondition::Boolean(false),
      };

      if matches!(runtime_condition, RuntimeCondition::Boolean(false)) {
        return;
      }

      let condition = runtime_template.runtime_condition_expression(
        &compilation.build_chunk_graph_artifact.chunk_graph,
        Some(&runtime_condition),
        *runtime,
      );
      let module_dependency = dependency
        .as_module_dependency()
        .expect("should be module dependency");
      let phase = ImportPhase::Evaluation;
      let import_var = compilation.get_import_var(
        module_identifier,
        target_module,
        module_dependency.user_request(),
        phase,
        *runtime,
      );
      let stmts = runtime_template.import_statement(
        *module,
        compilation,
        id,
        &import_var,
        module_dependency.request(),
        phase,
        true,
      );
      let mut refresh = String::default();
      if condition == "true" {
        refresh.push_str(stmts.0.as_str());
        refresh.push_str(stmts.1.as_str());
      } else {
        refresh.push_str(format!("if ({condition}) {{\n").as_str());
        refresh.push_str(stmts.0.as_str());
        refresh.push_str(stmts.1.as_str());
        refresh.push_str("\n}\n");
      }
      content.push_str("try {\n");
      content.push_str(&refresh);
      content.push_str("\n} catch (err) {\n");
      content.push_str("__rspack_hot_report_error(err);\n");
      content.push_str("}\n");
    });

    source.insert(
      dep.range.start,
      format!(", function(__rspack_hot_report_error) {{\n{content}\n}}"),
      None,
    );
  }
}
