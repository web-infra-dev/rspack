use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsModuleDependency, ContextDependency, ContextMode, ContextOptions, Dependency,
  DependencyCategory, DependencyCodeGeneration, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, ExportsInfoArtifact, FactorizeInfo, ModuleGraph,
  ModuleGraphCacheArtifact, TemplateContext, TemplateReplaceSource,
};
use rspack_error::Diagnostic;

use super::BasicContextDependency;

#[cacheable]
#[derive(Debug, Clone)]
pub struct ImportMetaGlobDependency {
  base: BasicContextDependency,
}

impl ImportMetaGlobDependency {
  pub fn new(options: ContextOptions, range: DependencyRange, optional: bool) -> Self {
    Self {
      base: BasicContextDependency::new(options, range, optional),
    }
  }
}

#[cacheable_dyn]
impl Dependency for ImportMetaGlobDependency {
  fn id(&self) -> &rspack_core::DependencyId {
    &self.base.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ImportMetaGlob
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.base.range)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }

  fn get_diagnostics(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
  ) -> Option<Vec<Diagnostic>> {
    self.base.critical.clone().map(|critical| vec![critical])
  }
}

impl ContextDependency for ImportMetaGlobDependency {
  fn request(&self) -> &str {
    &self.base.options.request
  }

  fn options(&self) -> &ContextOptions {
    &self.base.options
  }

  fn get_context(&self) -> Option<&str> {
    None
  }

  fn resource_identifier(&self) -> &str {
    &self.base.resource_identifier
  }

  fn get_optional(&self) -> bool {
    self.base.optional
  }

  fn type_prefix(&self) -> rspack_core::ContextTypePrefix {
    rspack_core::ContextTypePrefix::Normal
  }

  fn critical(&self) -> &Option<Diagnostic> {
    &self.base.critical
  }

  fn critical_mut(&mut self) -> &mut Option<Diagnostic> {
    &mut self.base.critical
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.base.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.base.factorize_info
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for ImportMetaGlobDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ImportMetaGlobDependencyTemplate::template_type())
  }
}

impl AsModuleDependency for ImportMetaGlobDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ImportMetaGlobDependencyTemplate;

impl ImportMetaGlobDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::ImportMetaGlob)
  }
}

impl DependencyTemplate for ImportMetaGlobDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ImportMetaGlobDependency>()
      .expect("ImportMetaGlobDependencyTemplate should be used for ImportMetaGlobDependency");

    let TemplateContext {
      compilation,
      runtime_template,
      ..
    } = code_generatable_context;

    let context_expr = runtime_template.module_raw(
      compilation,
      &dep.base.id,
      &dep.base.options.request,
      dep.base.optional,
    );
    let content = match dep.base.options.mode {
      ContextMode::Lazy => format!(
        "(function(modules) {{ if(typeof modules.keys !== \"function\") return modules; var map = {{}}; modules.keys().forEach(function(key) {{ map[key] = function() {{ return modules(key); }}; }}); return map; }})({context_expr})"
      ),
      _ => format!(
        "(function(modules) {{ if(typeof modules.keys !== \"function\") return modules; var map = {{}}; modules.keys().forEach(function(key) {{ map[key] = modules(key); }}); return map; }})({context_expr})"
      ),
    };
    source.replace(dep.base.range.start, dep.base.range.end, content, None);
  }
}
