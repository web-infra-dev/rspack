use cow_utils::CowUtils;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, CodeGenerationDataFilename, CodeGenerationDataUrl, Compilation, Dependency,
  DependencyCategory, DependencyId, DependencyRange, DependencyTemplate, DependencyType,
  DynamicDependencyTemplate, DynamicDependencyTemplateType, FactorizeInfo, ModuleDependency,
  ModuleIdentifier, TemplateContext, TemplateReplaceSource,
};

use crate::utils::{css_escape_string, AUTO_PUBLIC_PATH_PLACEHOLDER};

#[cacheable]
#[derive(Debug, Clone)]
pub struct CssUrlDependency {
  id: DependencyId,
  request: String,
  range: DependencyRange,
  replace_function: bool,
  factorize_info: FactorizeInfo,
}

impl CssUrlDependency {
  pub fn new(request: String, range: DependencyRange, replace_function: bool) -> Self {
    Self {
      request,
      range,
      id: DependencyId::new(),
      replace_function,
      factorize_info: Default::default(),
    }
  }

  fn get_target_url(
    &self,
    identifier: &ModuleIdentifier,
    compilation: &Compilation,
  ) -> Option<String> {
    // Here we need the asset module's runtime to get the code generation result, which is not equal to
    // the css module's runtime, but actually multiple runtime optimization doesn't affect asset module,
    // in different runtime asset module will always have the same code generation result, so we use
    // `runtime: None` to get the only one code generation result
    let code_gen_result = compilation.code_generation_results.get(identifier, None);
    if let Some(url) = code_gen_result.data.get::<CodeGenerationDataUrl>() {
      Some(url.inner().to_string())
    } else if let Some(data) = code_gen_result.data.get::<CodeGenerationDataFilename>() {
      let filename = data.filename();
      let public_path = data.public_path().cow_replace(
        "__RSPACK_PLUGIN_ASSET_AUTO_PUBLIC_PATH__",
        AUTO_PUBLIC_PATH_PLACEHOLDER,
      );
      Some(format!("{public_path}{filename}"))
    } else {
      None
    }
  }
}

#[cacheable_dyn]
impl Dependency for CssUrlDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Url
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssUrl
  }

  fn range(&self) -> Option<&DependencyRange> {
    Some(&self.range)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for CssUrlDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

#[cacheable_dyn]
impl DependencyTemplate for CssUrlDependency {
  fn dynamic_dependency_template(&self) -> Option<DynamicDependencyTemplateType> {
    Some(CssUrlDependencyTemplate::template_type())
  }
}

impl AsContextDependency for CssUrlDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct CssUrlDependencyTemplate;

impl CssUrlDependencyTemplate {
  pub fn template_type() -> DynamicDependencyTemplateType {
    DynamicDependencyTemplateType::DependencyType(DependencyType::CssUrl)
  }
}

impl DynamicDependencyTemplate for CssUrlDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyTemplate,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<CssUrlDependency>()
      .expect("CssUrlDependencyTemplate should be used for CssUrlDependency");

    let TemplateContext { compilation, .. } = code_generatable_context;
    if let Some(mgm) = compilation
      .get_module_graph()
      .module_graph_module_by_dependency_id(dep.id())
      && let Some(target_url) = dep.get_target_url(&mgm.module_identifier, compilation)
    {
      let target_url = css_escape_string(&target_url);
      let content = if dep.replace_function {
        format!("url({target_url})")
      } else {
        target_url
      };
      source.replace(dep.range.start, dep.range.end, &content, None);
    }
  }
}
