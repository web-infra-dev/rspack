use std::sync::LazyLock;

use regex::Regex;
use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  AsContextDependency, CodeGenerationPublicPathAutoReplace, ConnectionState, Dependency,
  DependencyCategory, DependencyCodeGeneration, DependencyCondition, DependencyConditionFn,
  DependencyId, DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType,
  ExportsInfoArtifact, FactorizeInfo, JavascriptParserUrl, ModuleDependency, ModuleGraph,
  ModuleGraphCacheArtifact, ModuleGraphConnection, RuntimeGlobals, RuntimeSpec, TemplateContext,
  TemplateReplaceSource, URLStaticMode, UsedByExports,
};
use swc_core::ecma::atoms::Atom;

use crate::{connection_active_used_by_exports, runtime::AUTO_PUBLIC_PATH_PLACEHOLDER};

#[cacheable]
#[derive(Debug, Clone)]
pub struct URLDependency {
  id: DependencyId,
  #[cacheable(with=AsPreset)]
  request: Atom,
  range: DependencyRange,
  range_url: DependencyRange,
  used_by_exports: Option<UsedByExports>,
  mode: Option<JavascriptParserUrl>,
  factorize_info: FactorizeInfo,
}

impl URLDependency {
  pub fn new(
    request: Atom,
    range: DependencyRange,
    range_url: DependencyRange,
    mode: Option<JavascriptParserUrl>,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      range,
      range_url,
      used_by_exports: None,
      mode,
      factorize_info: Default::default(),
    }
  }

  pub fn set_used_by_exports(&mut self, used_by_exports: Option<UsedByExports>) {
    self.used_by_exports = used_by_exports;
  }
}

#[cacheable_dyn]
impl Dependency for URLDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Url
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::NewUrl
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for URLDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn get_condition(&self) -> Option<DependencyCondition> {
    Some(DependencyCondition::new(URLDependencyCondition))
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for URLDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(URLDependencyTemplate::template_type())
  }
}

impl AsContextDependency for URLDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct URLDependencyTemplate;

pub static URL_STATIC_PLACEHOLDER: &str = "RSPACK_AUTO_URL_STATIC_PLACEHOLDER_";
pub static URL_STATIC_PLACEHOLDER_RE: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(&format!(r#"{URL_STATIC_PLACEHOLDER}(?<dep>\d+)"#)).expect("should be valid regex")
});

impl URLDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::NewUrl)
  }
}

impl DependencyTemplate for URLDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<URLDependency>()
      .expect("URLDependencyTemplate should be used for URLDependency");
    let TemplateContext {
      compilation,
      runtime_template,
      ..
    } = code_generatable_context;

    match dep.mode {
      Some(JavascriptParserUrl::Relative) => {
        source.replace(
          dep.range.start,
          dep.range.end,
          format!(
            "/* asset import */ new {}({}({}))",
            runtime_template.render_runtime_globals(&RuntimeGlobals::RELATIVE_URL),
            runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
            runtime_template.module_id(compilation, &dep.id, &dep.request, false),
          )
          .as_str(),
          None,
        );
      }
      Some(JavascriptParserUrl::NewUrlRelative) => {
        code_generatable_context.data.insert(URLStaticMode);
        code_generatable_context
          .data
          .insert(CodeGenerationPublicPathAutoReplace(true));
        source.replace(
          dep.range.start,
          dep.range.end,
          format!(
            "new URL({}, import.meta.url)",
            serde_json::to_string(&format!(
              "{AUTO_PUBLIC_PATH_PLACEHOLDER}{URL_STATIC_PLACEHOLDER}{}",
              &dep.id.as_u32()
            ))
            .expect("should serde"),
          )
          .as_str(),
          None,
        );
      }
      _ => {
        source.replace(
          dep.range_url.start,
          dep.range_url.end,
          format!(
            "/* asset import */{}({}), {}",
            runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
            runtime_template.module_id(compilation, &dep.id, &dep.request, false),
            runtime_template.render_runtime_globals(&RuntimeGlobals::BASE_URI)
          )
          .as_str(),
          None,
        );
      }
    }
  }
}

struct URLDependencyCondition;

impl DependencyConditionFn for URLDependencyCondition {
  fn get_connection_state(
    &self,
    connection: &ModuleGraphConnection,
    runtime: Option<&RuntimeSpec>,
    module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
  ) -> ConnectionState {
    let dependency = module_graph.dependency_by_id(&connection.dependency_id);
    let dependency = dependency
      .downcast_ref::<URLDependency>()
      .expect("should be URLDependency");
    ConnectionState::Active(connection_active_used_by_exports(
      connection,
      runtime,
      module_graph,
      exports_info_artifact,
      dependency.used_by_exports.as_ref(),
    ))
  }
}
