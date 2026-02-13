use std::{borrow::Cow, sync::Arc};

use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::Identifiable;
use rspack_core::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, BoxDependency, BoxModule, BuildContext,
  BuildInfo, BuildMeta, BuildResult, ChunkGraph, CodeGenerationResult, Compilation, Context,
  DependenciesBlock, DependencyId, DependencyRange, FactoryMeta, ImportPhase, LibIdentOptions,
  Module, ModuleArgument, ModuleCodeGenerationContext, ModuleFactoryCreateData, ModuleGraph,
  ModuleIdentifier, ModuleLayer, ModuleType, RuntimeGlobals, RuntimeSpec, SourceType,
  ValueCacheVersions, impl_module_meta_info, module_update_hash,
  rspack_sources::{BoxSource, RawStringSource},
};
use rspack_error::{Result, impl_empty_diagnosable_trait};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_plugin_javascript::dependency::CommonJsRequireDependency;
use rspack_util::{
  ext::DynHash,
  json_stringify,
  source_map::{ModuleSourceMapConfig, SourceMapKind},
};

use crate::{
  dependency::{DependencyOptions, LazyCompilationDependency},
  utils::calc_value_dependency_key,
};

static MODULE_TYPE: ModuleType = ModuleType::JsAuto;
static SOURCE_TYPE: [SourceType; 1] = [SourceType::JavaScript];

#[cacheable]
#[derive(Debug)]
pub(crate) struct LazyCompilationProxyModule {
  build_info: BuildInfo,
  build_meta: BuildMeta,
  factory_meta: Option<FactoryMeta>,

  readable_identifier: String,
  identifier: ModuleIdentifier,
  lib_ident: Option<String>,

  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,

  source_map_kind: SourceMapKind,

  context: Box<Context>,
  layer: Option<ModuleLayer>,
  dep_options: DependencyOptions,
  resource: String,
  active: bool,
  client: String,
  need_build: bool,
}

impl ModuleSourceMapConfig for LazyCompilationProxyModule {
  fn get_source_map_kind(&self) -> &SourceMapKind {
    &self.source_map_kind
  }

  fn set_source_map_kind(&mut self, source_map: SourceMapKind) {
    self.source_map_kind = source_map;
  }
}

impl LazyCompilationProxyModule {
  #[allow(clippy::too_many_arguments)]
  pub(crate) fn new(
    identifier: ModuleIdentifier,
    readable_identifier: String,
    lib_ident: Option<String>,
    create_data: &ModuleFactoryCreateData,
    resource: String,
    active: bool,
    client: String,
  ) -> Self {
    let lib_ident = lib_ident.map(|s| format!("{s}!lazy-compilation-proxy"));

    let dep_options = DependencyOptions {
      request: create_data.request.clone(),
      file_dependencies: create_data.file_dependencies.clone(),
      context_dependencies: create_data.context_dependencies.clone(),
      missing_dependencies: create_data.missing_dependencies.clone(),
      diagnostics: create_data.diagnostics.clone(),
    };

    Self {
      build_info: Default::default(),
      build_meta: Default::default(),
      factory_meta: None,
      readable_identifier,
      lib_ident,
      identifier,
      source_map_kind: SourceMapKind::empty(),
      blocks: vec![],
      dependencies: vec![],
      context: Box::new(create_data.context.clone()),
      layer: create_data.issuer_layer.clone(),
      dep_options,
      resource,
      active,
      client,
      need_build: false,
    }
  }

  pub fn invalid(&mut self) {
    self.need_build = true;
  }
}

impl_empty_diagnosable_trait!(LazyCompilationProxyModule);

#[cacheable_dyn]
#[async_trait::async_trait]
impl Module for LazyCompilationProxyModule {
  impl_module_meta_info!();

  fn source_types(&self, _module_graph: &ModuleGraph) -> &[SourceType] {
    &SOURCE_TYPE
  }

  fn module_type(&self) -> &ModuleType {
    &MODULE_TYPE
  }

  fn get_context(&self) -> Option<Box<Context>> {
    Some(self.context.clone())
  }

  fn get_layer(&self) -> Option<&ModuleLayer> {
    self.layer.as_ref()
  }

  fn size(&self, _source_type: Option<&SourceType>, _compilation: Option<&Compilation>) -> f64 {
    200f64
  }

  fn source(&self) -> Option<&BoxSource> {
    None
  }

  fn readable_identifier(&self, _context: &Context) -> std::borrow::Cow<'_, str> {
    std::borrow::Cow::Borrowed(&self.readable_identifier)
  }

  fn lib_ident(&self, _options: LibIdentOptions) -> Option<Cow<'_, str>> {
    self.lib_ident.as_ref().map(|s| Cow::Borrowed(s.as_str()))
  }

  fn need_build(&self, value_cache_versions: &ValueCacheVersions) -> bool {
    if self.need_build {
      return true;
    }
    // check client changes
    let cache_key = calc_value_dependency_key("client");
    if let Some(client) = value_cache_versions.get(&cache_key)
      && client == &self.client
    {
      false
    } else {
      true
    }
  }

  async fn build(
    mut self: Box<Self>,
    _build_context: BuildContext,
    _compilation: Option<&Compilation>,
  ) -> Result<BuildResult> {
    let client_dep = CommonJsRequireDependency::new(
      self.client.clone(),
      DependencyRange::new(0, 0),
      None,
      false,
      None,
    );
    let mut dependencies = vec![];
    let mut blocks = vec![];

    dependencies.push(Box::new(client_dep) as BoxDependency);

    if self.active {
      let dep = LazyCompilationDependency::new(self.dep_options.clone());

      blocks.push(Box::new(AsyncDependenciesBlock::new(
        self.identifier,
        None,
        None,
        vec![Box::new(dep)],
        None,
      )));
    }

    Ok(BuildResult {
      module: BoxModule::new(self),
      dependencies,
      blocks,
      optimization_bailouts: vec![],
    })
  }

  // #[tracing::instrument("LazyCompilationProxyModule::code_generation", skip_all, fields(identifier = ?self.identifier()))]
  async fn code_generation(
    &self,
    code_generation_context: &mut ModuleCodeGenerationContext,
  ) -> Result<CodeGenerationResult> {
    let ModuleCodeGenerationContext {
      compilation,
      runtime_template,
      ..
    } = code_generation_context;

    let client_dep_id = self.dependencies[0];
    let module_graph = &compilation.get_module_graph();

    let client_module = module_graph
      .module_identifier_by_dependency_id(&client_dep_id)
      .expect("should have module");

    let block = self.blocks.first();

    let client = format!(
      "var client = {}(\"{}\");\nvar data = {};",
      runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
      ChunkGraph::get_module_id(&compilation.module_ids_artifact, *client_module)
        .expect("should have module id"),
      serde_json::to_string(&self.identifier).expect("should serialize identifier")
    );

    let module_argument = runtime_template.render_module_argument(ModuleArgument::Module);

    let keep_active = format!(
      "var dispose = client.activate({{ data: data, active: {}, module: {module_argument}, onError: onError }})",
      block.is_some()
    );

    let source = if let Some(block_id) = block {
      let block = module_graph
        .block_by_id(block_id)
        .expect("should have block");

      let dep_id = block.get_dependencies()[0];
      let module = module_graph
        .module_identifier_by_dependency_id(&dep_id)
        .expect("should have module");

      RawStringSource::from(format!(
        "{client}
        {module_argument}.exports = {};
        if ({module_argument}.hot) {{
          {module_argument}.hot.accept();
          {module_argument}.hot.accept({}, function() {{ {module_argument}.hot.invalidate(); }});
          {module_argument}.hot.dispose(function(data) {{ delete data.resolveSelf; }});
          if ({module_argument}.hot.data && {module_argument}.hot.data.resolveSelf)
            {module_argument}.hot.data.resolveSelf({module_argument}.exports);
        }}
        ",
        runtime_template.module_namespace_promise(
          compilation,
          *module,
          &dep_id,
          Some(block_id),
          &self.resource,
          "import()",
          false,
          ImportPhase::Evaluation,
        ),
        json_stringify(
          ChunkGraph::get_module_id(&compilation.module_ids_artifact, *module)
            .expect("should have module id")
        ),
      ))
    } else {
      RawStringSource::from(format!(
        "{client}
        var resolveSelf, onError;
        {module_argument}.exports = new Promise(function(resolve, reject) {{ resolveSelf = resolve; onError = reject; }});
        if ({module_argument}.hot) {{
          {module_argument}.hot.accept();
          if ({module_argument}.hot.data && {module_argument}.hot.data.resolveSelf) {module_argument}.hot.data.resolveSelf({module_argument}.exports);
          {module_argument}.hot.dispose(function(data) {{ data.resolveSelf = resolveSelf; dispose(data); }});
        }}
        {keep_active}
      "
      ))
    };

    Ok(CodeGenerationResult::default().with_javascript(Arc::new(source)))
  }

  async fn get_runtime_hash(
    &self,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<RspackHashDigest> {
    let mut hasher = RspackHash::from(&compilation.options.output);
    module_update_hash(self, &mut hasher, compilation, runtime);
    self.active.dyn_hash(&mut hasher);
    self.identifier.dyn_hash(&mut hasher);
    Ok(hasher.digest(&compilation.options.output.hash_digest))
  }
}

impl Identifiable for LazyCompilationProxyModule {
  fn identifier(&self) -> rspack_collections::Identifier {
    self.identifier
  }
}

impl DependenciesBlock for LazyCompilationProxyModule {
  fn add_block_id(&mut self, block: rspack_core::AsyncDependenciesBlockIdentifier) {
    self.blocks.push(block);
  }

  fn get_blocks(&self) -> &[rspack_core::AsyncDependenciesBlockIdentifier] {
    &self.blocks
  }

  fn add_dependency_id(&mut self, dependency: rspack_core::DependencyId) {
    self.dependencies.push(dependency);
  }

  fn remove_dependency_id(&mut self, dependency: rspack_core::DependencyId) {
    self.dependencies.retain(|d| d != &dependency);
  }

  fn get_dependencies(&self) -> &[rspack_core::DependencyId] {
    &self.dependencies
  }
}
