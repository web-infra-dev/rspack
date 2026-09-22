use std::sync::LazyLock;

use concat_string::concat_string;
use regex::Regex;
use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  AsContextDependency, ChunkGroup, ChunkUkey, CodeGenerationPublicPathAutoReplace, Compilation,
  ConnectionState, Dependency, DependencyCategory, DependencyCodeGeneration, DependencyCondition,
  DependencyConditionFn, DependencyId, DependencyLocation, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, ExportsInfoArtifact, GroupOptions, JavascriptParserUrl,
  Module, ModuleCodeTemplate, ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact,
  ModuleGraphConnection, ModuleType, RuntimeGlobals, RuntimeSpec, SideEffectsStateArtifact,
  TemplateContext, TemplateReplaceSource, URLStaticMode, UsedByExports,
};

use crate::{Atom, connection_active_used_by_exports, runtime::AUTO_PUBLIC_PATH_PLACEHOLDER};

#[cacheable]
// Cloned for factory-only type probes and to move a nested parser dependency into a URL entry.
#[derive(Debug, Clone)]
pub struct URLDependency {
  pub(crate) loc: Option<DependencyLocation>,
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
      loc: None,
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

  /// Replace the arguments of `new URL(request, import.meta.url)` with the given
  /// request, keeping the `new URL(...)` call itself untouched.
  pub fn replace_request(&self, source: &mut TemplateReplaceSource, request: String) {
    source.replace(
      self.range_url.start,
      self.range_url.end,
      concat_string!(request, ", import.meta.url"),
      None,
    );
  }
}

#[cacheable_dyn]
impl Dependency for URLDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn loc(&self) -> Option<DependencyLocation> {
    self.loc.clone()
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
        let (expression, comment) = get_url_expression(dep, compilation, runtime_template);
        source.replace(
          dep.range.start,
          dep.range.end,
          format!(
            "{comment} new {}({expression})",
            runtime_template.render_runtime_globals(&RuntimeGlobals::RELATIVE_URL),
          ),
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
            rspack_util::json_stringify_str(&format!(
              "{AUTO_PUBLIC_PATH_PLACEHOLDER}{URL_STATIC_PLACEHOLDER}{}",
              &dep.id.as_u32()
            )),
          ),
          None,
        );
      }
      _ => {
        let (expression, comment) = get_url_expression(dep, compilation, runtime_template);
        source.replace(
          dep.range_url.start,
          dep.range_url.end,
          format!(
            "{comment}{expression}, {}",
            runtime_template.render_runtime_globals(&RuntimeGlobals::BASE_URI)
          ),
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
    _side_effects_state_artifact: &SideEffectsStateArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
  ) -> ConnectionState {
    let dependency = module_graph.dependency_by_id(&connection.dependency_id);
    let dependency = dependency
      .downcast_ref::<URLDependency>()
      .expect("should be URLDependency");
    let runtime = if module_graph
      .get_parent_block(&connection.dependency_id)
      .is_some_and(|block| {
        matches!(
          module_graph.block_by_id_expect(block).get_group_options(),
          Some(GroupOptions::Entrypoint(_))
        )
      }) {
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

pub(crate) fn is_url_value_module(module: &dyn Module) -> bool {
  module.module_type().is_asset_like()
    || matches!(
      module.module_type(),
      ModuleType::AssetSource | ModuleType::AssetBytes
    )
    || module.as_external_module().is_some()
    || module.identifier().as_str().starts_with("ignored|")
}

fn get_dependency_entrypoint<'a>(
  compilation: &'a Compilation,
  dependency_id: &DependencyId,
) -> Option<&'a ChunkGroup> {
  let module_graph = compilation.get_module_graph();
  module_graph
    .get_parent_block(dependency_id)
    .filter(|block| {
      matches!(
        module_graph.block_by_id_expect(block).get_group_options(),
        Some(GroupOptions::Entrypoint(_))
      )
    })
    .and_then(|block| {
      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_block_chunk_group(
          block,
          &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
        )
    })
}

pub(crate) fn get_dependency_entry_chunk(
  compilation: &Compilation,
  dependency_id: &DependencyId,
) -> Option<ChunkUkey> {
  get_dependency_entrypoint(compilation, dependency_id).map(ChunkGroup::get_entrypoint_chunk)
}

/// URL entries are loaded through a single URL, including when their modules are split out.
pub fn is_url_entry_chunk(compilation: &Compilation, chunk_ukey: &ChunkUkey) -> bool {
  let module_graph = compilation.get_module_graph();
  compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .get_chunk_entry_modules_with_chunk_group_iterable(chunk_ukey)
    .keys()
    .any(|module| {
      module_graph
        .get_incoming_connections(module)
        .any(|connection| {
          module_graph
            .dependency_by_id(&connection.dependency_id)
            .downcast_ref::<URLDependency>()
            .is_some()
            && get_dependency_entry_chunk(compilation, &connection.dependency_id)
              == Some(*chunk_ukey)
        })
    })
}

fn get_url_expression(
  dep: &URLDependency,
  compilation: &Compilation,
  runtime_template: &mut ModuleCodeTemplate,
) -> (String, &'static str) {
  if let Some(chunk_ukey) = get_dependency_entry_chunk(compilation, &dep.id) {
    let chunk_id = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(&chunk_ukey)
      .id()
      .map(rspack_util::json_stringify)
      .expect("URL entry should have a chunk id");
    let public_path = runtime_template.render_runtime_globals(&RuntimeGlobals::PUBLIC_PATH);
    let chunk_filename =
      runtime_template.render_runtime_globals(&RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME);
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
  }
}
