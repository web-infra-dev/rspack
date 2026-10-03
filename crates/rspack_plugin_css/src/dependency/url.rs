use cow_utils::CowUtils;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, CodeGenerationDataFilename, CodeGenerationDataUrl, Compilation,
  CssExportType, Dependency, DependencyCategory, DependencyCodeGeneration, DependencyId,
  DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType, ModuleDependency,
  ModuleIdentifier, TemplateContext, TemplateReplaceSource,
};

use crate::{
  css_syntax::serialize_url_value,
  parser_and_generator::CssParserAndGenerator,
  utils::{AUTO_PUBLIC_PATH_PLACEHOLDER, RUNTIME_PUBLIC_PATH_PLACEHOLDER, css_module_export_type},
};

const ASSET_AUTO_PUBLIC_PATH_PLACEHOLDER: &str = "__RSPACK_PLUGIN_ASSET_AUTO_PUBLIC_PATH__";

#[cacheable]
#[derive(Debug)]
pub struct CssUrlDependency {
  id: DependencyId,
  request: String,
  range: DependencyRange,
  replace_function: bool,
}

impl CssUrlDependency {
  pub fn new(request: String, range: DependencyRange, replace_function: bool) -> Self {
    Self {
      request,
      range,
      id: DependencyId::new(),
      replace_function,
    }
  }

  fn get_target_url(
    &self,
    identifier: &ModuleIdentifier,
    compilation: &Compilation,
    runtime_public_path: bool,
  ) -> Option<String> {
    // url points to asset modules, and asset modules should have same codegen results for all runtimes
    let code_gen_result = compilation.code_generation_results.get_one(identifier);

    // When enabled, injected asset URLs use import.meta.rspackPublicPath.
    // An explicit asset generator publicPath still takes precedence.
    if runtime_public_path
      && let Some(data) = code_gen_result.data().get::<CodeGenerationDataFilename>()
      && let Some(module) = compilation
        .get_module_graph()
        .module_by_identifier(identifier)
      && module
        .as_normal_module()
        .and_then(|module| module.get_generator_options())
        .and_then(|options| options.asset_public_path())
        .is_none()
    {
      let filename = code_gen_result
        .data()
        .get::<CodeGenerationDataUrl>()
        .and_then(|url| url.inner().strip_prefix(data.public_path()))
        .unwrap_or_else(|| data.filename());
      return Some(format!("{RUNTIME_PUBLIC_PATH_PLACEHOLDER}{filename}"));
    }

    if let Some(url) = code_gen_result.data().get::<CodeGenerationDataUrl>() {
      Some(
        url
          .inner()
          .cow_replace(
            ASSET_AUTO_PUBLIC_PATH_PLACEHOLDER,
            AUTO_PUBLIC_PATH_PLACEHOLDER,
          )
          .into_owned(),
      )
    } else if let Some(data) = code_gen_result.data().get::<CodeGenerationDataFilename>() {
      let filename = data.filename();
      let public_path = data.public_path().cow_replace(
        ASSET_AUTO_PUBLIC_PATH_PLACEHOLDER,
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

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
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
}

#[cacheable_dyn]
impl DependencyCodeGeneration for CssUrlDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(CssUrlDependencyTemplate::template_type())
  }
}

impl AsContextDependency for CssUrlDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct CssUrlDependencyTemplate;

impl CssUrlDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::CssUrl)
  }
}

impl DependencyTemplate for CssUrlDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<CssUrlDependency>()
      .expect("CssUrlDependencyTemplate should be used for CssUrlDependency");

    let module = code_generatable_context.module;
    let runtime_public_path = css_module_export_type(module) == Some(CssExportType::Style)
      && module
        .as_normal_module()
        .and_then(|module| {
          module
            .parser_and_generator()
            .downcast_ref::<CssParserAndGenerator>()
        })
        .is_some_and(|parser| parser.runtime_public_path);

    let TemplateContext { compilation, .. } = code_generatable_context;
    if let Some(mgm) = compilation
      .get_module_graph()
      .module_graph_module_by_dependency_id(dep.id())
      && let Some(target_url) =
        dep.get_target_url(&mgm.module_identifier, compilation, runtime_public_path)
    {
      let target_url = serialize_url_value(&target_url);
      let content = if dep.replace_function {
        format!("url({target_url})")
      } else {
        target_url
      };
      source.replace(dep.range.start, dep.range.end, content, None);
    }
  }
}
