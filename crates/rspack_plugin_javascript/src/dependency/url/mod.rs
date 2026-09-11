use std::sync::LazyLock;

use concat_string::concat_string;
use regex::Regex;
use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  AsContextDependency, ChunkUkey, CodeGenerationPublicPathAutoReplace, Compilation,
  ConnectionState, Dependency, DependencyCategory, DependencyCodeGeneration, DependencyCondition,
  DependencyConditionFn, DependencyId, DependencyRange, DependencyTemplate, DependencyTemplateType,
  DependencyType, ExportsInfoArtifact, JavascriptParserUrl, ModuleDependency, ModuleGraph,
  ModuleGraphCacheArtifact, ModuleGraphConnection, ModuleType, RuntimeGlobals, RuntimeSpec,
  SideEffectsStateArtifact, SourceType, TemplateContext, TemplateReplaceSource, URLStaticMode,
  UsedByExports,
};

use crate::{Atom, connection_active_used_by_exports, runtime::AUTO_PUBLIC_PATH_PLACEHOLDER};

#[cacheable]
#[derive(Debug)]
pub struct URLDependency {
  id: DependencyId,
  #[cacheable(with=AsPreset)]
  request: Atom,
  range: DependencyRange,
  range_url: DependencyRange,
  used_by_exports: Option<UsedByExports>,
  mode: Option<JavascriptParserUrl>,
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
    }
  }

  pub fn set_used_by_exports(&mut self, used_by_exports: Option<UsedByExports>) {
    self.used_by_exports = used_by_exports;
  }

  pub fn used_by_exports(&self) -> Option<&UsedByExports> {
    self.used_by_exports.as_ref()
  }

  pub fn dependency_range(&self) -> DependencyRange {
    self.range
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

  fn url_mode(&self) -> Option<JavascriptParserUrl> {
    self.mode
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
  Regex::new(&concat_string!(URL_STATIC_PLACEHOLDER, r#"(?<dep>\d+)"#))
    .expect("should be valid regex")
});

pub(crate) fn url_entry_source_type(module_type: &ModuleType) -> Option<SourceType> {
  match module_type {
    ModuleType::Css | ModuleType::CssAuto | ModuleType::CssModule | ModuleType::CssGlobal => {
      Some(SourceType::Css)
    }
    _ if module_type.is_js_like() => Some(SourceType::JavaScript),
    _ => None,
  }
}

pub(crate) fn get_dependency_entry_chunk(
  compilation: &Compilation,
  dependency_id: &DependencyId,
) -> Option<ChunkUkey> {
  compilation
    .get_module_graph()
    .get_parent_block(dependency_id)
    .map(|block| {
      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_block_chunk_group(
          block,
          &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
        )
        .expect("URL dependency should have an entrypoint chunk")
        .get_entrypoint_chunk()
    })
}

fn render_static_url(
  dep: &URLDependency,
  source: &mut TemplateReplaceSource,
  context: &mut TemplateContext,
) {
  context.data.insert(URLStaticMode);
  context
    .data
    .insert(CodeGenerationPublicPathAutoReplace(true));
  let url = rspack_util::json_stringify_str(&format!(
    "{AUTO_PUBLIC_PATH_PLACEHOLDER}{URL_STATIC_PLACEHOLDER}{}",
    dep.id.as_u32()
  ));
  source.replace(
    dep.range.start,
    dep.range.end,
    concat_string!("new URL(", url, ", import.meta.url)"),
    None,
  );
}

fn render_url_expression(
  dep: &URLDependency,
  source: &mut TemplateReplaceSource,
  context: &mut TemplateContext,
  expression: &str,
  comment: &str,
) {
  let runtime_template = &mut context.runtime_template;
  if matches!(dep.mode, Some(JavascriptParserUrl::Relative)) {
    let relative_url = runtime_template.render_runtime_globals(&RuntimeGlobals::RELATIVE_URL);
    source.replace(
      dep.range.start,
      dep.range.end,
      concat_string!(comment, " new ", relative_url, "(", expression, ")"),
      None,
    );
  } else {
    let base_uri = runtime_template.render_runtime_globals(&RuntimeGlobals::BASE_URI);
    source.replace(
      dep.range_url.start,
      dep.range_url.end,
      concat_string!(comment, expression, ", ", base_uri),
      None,
    );
  }
}

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
    if matches!(dep.mode, Some(JavascriptParserUrl::NewUrlRelative)) {
      render_static_url(dep, source, code_generatable_context);
      return;
    }

    let TemplateContext {
      compilation,
      runtime_template,
      ..
    } = code_generatable_context;
    let (expression, comment) =
      if let Some(chunk_ukey) = get_dependency_entry_chunk(compilation, &dep.id) {
        let chunk_id = compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .expect_get(&chunk_ukey)
          .id()
          .map(rspack_util::json_stringify)
          .expect("URL entry should have a chunk id");
        let target_module = compilation
          .get_module_graph()
          .get_module_by_dependency_id(&dep.id)
          .expect("URL entry should have a target module");
        let chunk_filename_global = match url_entry_source_type(target_module.module_type()) {
          Some(SourceType::Css) => RuntimeGlobals::GET_CHUNK_CSS_FILENAME,
          _ => RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME,
        };
        let public_path = runtime_template.render_runtime_globals(&RuntimeGlobals::PUBLIC_PATH);
        let chunk_filename = runtime_template.render_runtime_globals(&chunk_filename_global);
        (
          concat_string!(public_path, " + ", chunk_filename, "(", chunk_id, ")"),
          "/* entry url */",
        )
      } else {
        let require = runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE);
        let module_id = runtime_template.module_id(compilation, &dep.id, &dep.request, false);
        (
          concat_string!(require, "(", module_id, ")"),
          "/* asset import */",
        )
      };
    render_url_expression(dep, source, code_generatable_context, &expression, comment);
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
    _side_effects_state_artifact: &SideEffectsStateArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
  ) -> ConnectionState {
    let dependency = module_graph.dependency_by_id(&connection.dependency_id);
    let dependency = dependency
      .downcast_ref::<URLDependency>()
      .expect("should be URLDependency");
    let runtime = if module_graph
      .get_parent_block(&connection.dependency_id)
      .is_some()
    {
      None
    } else {
      runtime
    };
    ConnectionState::Active(connection_active_used_by_exports(
      connection,
      runtime,
      module_graph,
      exports_info_artifact,
      dependency.used_by_exports.as_ref(),
    ))
  }
}
