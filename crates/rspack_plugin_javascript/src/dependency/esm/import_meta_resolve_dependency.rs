use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, ConnectionState, Dependency, DependencyCategory, DependencyCodeGeneration,
  DependencyCondition, DependencyConditionFn, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, ExportsInfoArtifact, GroupOptions, JavascriptParserUrl,
  Module, ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact, ModuleGraphConnection,
  RuntimeGlobals, RuntimeSpec, SideEffectsStateArtifact, TemplateContext, TemplateReplaceSource,
  UsedByExports,
};

use crate::connection_active_used_by_exports;

#[cacheable]
#[derive(Debug)]
pub struct ImportMetaResolveDependency {
  pub id: DependencyId,
  pub request: String,
  range: DependencyRange,
  mode: Option<JavascriptParserUrl>,
  optional: bool,
  used_by_exports: Option<UsedByExports>,
}

impl ImportMetaResolveDependency {
  pub fn new(
    request: String,
    range: DependencyRange,
    mode: Option<JavascriptParserUrl>,
    optional: bool,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      range,
      mode,
      optional,
      used_by_exports: None,
    }
  }

  pub fn set_used_by_exports(&mut self, used_by_exports: UsedByExports) {
    self.used_by_exports = Some(used_by_exports);
  }
}

/// Only executable JavaScript needs a standalone entry. Externals and ignored
/// modules supply values, just like assets, even when their type is JavaScript.
pub(crate) fn is_resolve_entry(module: &dyn Module) -> bool {
  module.module_type().is_js_like()
    && module.as_external_module().is_none()
    && !module.identifier().starts_with("ignored|")
}

#[cacheable_dyn]
impl Dependency for ImportMetaResolveDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }
  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Url
  }
  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ImportMetaResolve
  }
  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }
  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for ImportMetaResolveDependency {
  fn request(&self) -> &str {
    &self.request
  }
  fn user_request(&self) -> &str {
    &self.request
  }
  fn get_optional(&self) -> bool {
    self.optional
  }
  fn get_condition(&self) -> Option<DependencyCondition> {
    Some(DependencyCondition::new(
      ImportMetaResolveDependencyCondition,
    ))
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for ImportMetaResolveDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ImportMetaResolveDependencyTemplate::template_type())
  }
}

impl AsContextDependency for ImportMetaResolveDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ImportMetaResolveDependencyTemplate;

impl ImportMetaResolveDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::ImportMetaResolve)
  }
}

impl DependencyTemplate for ImportMetaResolveDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ImportMetaResolveDependency>()
      .expect(
        "ImportMetaResolveDependencyTemplate should only be used for ImportMetaResolveDependency",
      );
    let compilation = context.compilation;
    let module_graph = compilation.get_module_graph();
    if let Some(connection) = module_graph.connection_by_dependency_id(&dep.id)
      && !connection.is_target_active(
        module_graph,
        context.runtime,
        &compilation.module_graph_cache_artifact,
        &compilation
          .build_module_graph_artifact
          .side_effects_state_artifact,
        &compilation.exports_info_artifact,
      )
    {
      source.replace(
        dep.range.start,
        dep.range.end,
        "/* unused asset import */ undefined".into(),
        None,
      );
      return;
    }
    let request = if module_graph
      .module_identifier_by_dependency_id(&dep.id)
      .and_then(|id| module_graph.module_by_identifier(id))
      .is_some_and(|module| is_resolve_entry(module.as_ref()))
    {
      let graph = &compilation.build_chunk_graph_artifact;
      let entry = module_graph
        .get_parent_block(&dep.id)
        .and_then(|block| {
          graph
            .chunk_graph
            .get_block_chunk_group(block, &graph.chunk_group_by_ukey)
        })
        .expect("import.meta.resolve entry should have a chunk group");
      let chunk = graph
        .chunk_by_ukey
        .expect_get(&entry.get_entrypoint_chunk());
      format!(
        "{} + {}({})",
        context
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::PUBLIC_PATH),
        context
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME),
        rspack_util::json_stringify(chunk.expect_id())
      )
    } else {
      context
        .runtime_template
        .module_raw(compilation, &dep.id, &dep.request, false)
    };
    let expression = if matches!(dep.mode, Some(JavascriptParserUrl::Relative)) {
      format!(
        "(new {}(/* asset import */ {})).href",
        context
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::RELATIVE_URL),
        request
      )
    } else {
      format!(
        "(new URL(/* asset import */ {}, {})).href",
        request,
        context
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::BASE_URI)
      )
    };
    source.replace(dep.range.start, dep.range.end, expression, None);
  }
}

struct ImportMetaResolveDependencyCondition;

impl DependencyConditionFn for ImportMetaResolveDependencyCondition {
  fn get_connection_state(
    &self,
    connection: &ModuleGraphConnection,
    runtime: Option<&RuntimeSpec>,
    module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _side_effects_state_artifact: &SideEffectsStateArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
  ) -> ConnectionState {
    let dependency = module_graph
      .dependency_by_id(&connection.dependency_id)
      .downcast_ref::<ImportMetaResolveDependency>()
      .expect("should be ImportMetaResolveDependency");
    // The generated entry has a separate runtime, while used_by_exports refers
    // to the importing module. Keep runtime filtering when rendering the caller.
    let entry_runtime = module_graph
      .get_parent_block(&connection.dependency_id)
      .and_then(|id| module_graph.block_by_id(id))
      .and_then(|block| match block.get_group_options() {
        Some(GroupOptions::Entrypoint(options)) => options.runtime.as_ref(),
        _ => None,
      });
    let is_entry_runtime = matches!(entry_runtime, Some(rspack_core::EntryRuntime::String(name))
      if runtime.is_some_and(|runtime| runtime.iter().any(|value| value.as_str() == name)));
    let runtime = if is_entry_runtime { None } else { runtime };
    ConnectionState::Active(connection_active_used_by_exports(
      connection,
      runtime,
      module_graph,
      exports_info_artifact,
      dependency.used_by_exports.as_ref(),
    ))
  }
}

/// A standalone resolve entry must load its own split startup chunks because
/// loading its URL does not load the sibling chunks through an HTML entry.
pub fn is_import_meta_resolve_entry_chunk(
  compilation: &rspack_core::Compilation,
  chunk: &rspack_core::ChunkUkey,
) -> bool {
  let module_graph = compilation.get_module_graph();
  let graph = &compilation.build_chunk_graph_artifact;
  graph
    .chunk_graph
    .get_chunk_entry_modules_with_chunk_group_iterable(chunk)
    .keys()
    .any(|module| {
      module_graph
        .get_incoming_connections(module)
        .any(|connection| {
          module_graph
            .dependency_by_id(&connection.dependency_id)
            .dependency_type()
            == &DependencyType::ImportMetaResolve
            && module_graph
              .get_parent_block(&connection.dependency_id)
              .and_then(|block| {
                graph
                  .chunk_graph
                  .get_block_chunk_group(block, &graph.chunk_group_by_ukey)
              })
              .is_some_and(|group| {
                matches!(group.kind, rspack_core::ChunkGroupKind::Entrypoint { .. })
                  && group.get_entrypoint_chunk() == *chunk
              })
        })
    })
}
