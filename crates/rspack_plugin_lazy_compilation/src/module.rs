use std::{borrow::Cow, sync::Arc};

use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::Identifiable;
use rspack_core::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, BoxDependency, BuildContext, BuildInfo,
  BuildMeta, BuildResult, ChunkGraph, CodeGenerationData, CodeGenerationResult, Compilation,
  ConcatenationScope, Context, DependenciesBlock, DependencyId, DependencyRange, FactoryMeta,
  LibIdentOptions, Module, ModuleFactoryCreateData, ModuleGraph, ModuleIdentifier, ModuleLayer,
  ModuleType, RuntimeGlobals, RuntimeSpec, SourceType, TemplateContext, ValueCacheVersions,
  impl_module_meta_info, module_update_hash,
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
    &mut self,
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
      dependencies,
      blocks,
      optimization_bailouts: vec![],
    })
  }

  // #[tracing::instrument("LazyCompilationProxyModule::code_generation", skip_all, fields(identifier = ?self.identifier()))]
  async fn code_generation(
    &self,
    compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
    mut concatenation_scope: Option<ConcatenationScope>,
  ) -> Result<CodeGenerationResult> {
    let mut runtime_requirements = RuntimeGlobals::empty();
    runtime_requirements.insert(RuntimeGlobals::MODULE);
    runtime_requirements.insert(RuntimeGlobals::REQUIRE);
    let mut codegen_data = CodeGenerationData::default();

    let client_dep_id = self.dependencies[0];
    let module_graph = &compilation.get_module_graph();

    let client_module = module_graph
      .module_identifier_by_dependency_id(&client_dep_id)
      .expect("should have module");

    let block = self.blocks.first();

    let client = format!(
      "var client = {}(\"{}\");\nvar data = {};",
      compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::REQUIRE),
      ChunkGraph::get_module_id(&compilation.module_ids_artifact, *client_module)
        .expect("should have module id"),
      serde_json::to_string(&self.identifier).expect("should serialize identifier")
    );

    let activate_code = format!(
      r#"
      window.dispatchEvent(new ErrorEvent('error', {{
  message: `"start to activdate ${{ new Date()}} ${{ Date.now() }} ${{ data }} `,
  filename: 'manual.js',
  lineno: 10,
  colno: 20,
  error: new Error('')
}}));
      console.error("start to activdate", new Date(), Date.now(), data);
var dispose = client.activate({{ data: data, active: {}, module: module, onError: onError }})"#,
      block.is_some()
    );

    // Activated Module
    let source = if let Some(block_id) = block {
      let block = module_graph
        .block_by_id(block_id)
        .expect("should have block");

      let dep_id = block.get_dependencies()[0];
      let module = module_graph
        .module_identifier_by_dependency_id(&dep_id)
        .expect("should have module");

      let mut template_ctx = TemplateContext {
        compilation,
        module: module_graph
          .module_by_identifier(module)
          .expect("should have module")
          .as_ref(),
        runtime_requirements: &mut runtime_requirements,
        init_fragments: &mut vec![],
        runtime: None,
        concatenation_scope: concatenation_scope.as_mut(),
        data: &mut codegen_data,
      };

      RawStringSource::from(format!(
        "{client}
        module.exports = {};
        if (module.hot) {{
          module.hot.accept();
          module.hot.accept({}, function() {{ module.hot.invalidate(); }});
          module.hot.dispose(function(data) {{ delete data.resolveSelf; }});
          if (module.hot.data && module.hot.data.resolveSelf){{
            console.error(\"Module Activated:\", new Date(), Date.now(), \"{}\");

      window.dispatchEvent(new ErrorEvent('error', {{
  message: `Module Activated: ${{ new Date()}} ${{ Date.now() }} {}`,
  filename: 'manual.js',
  lineno: 10,
  colno: 20,
  error: new Error('')
}}));

            module.hot.data.resolveSelf(module.exports);
          }}
        }}
        ",
        compilation.runtime_template.module_namespace_promise(
          &mut template_ctx,
          &dep_id,
          Some(block_id),
          &self.resource,
          "import()",
          false
        ),
        json_stringify(
          ChunkGraph::get_module_id(&compilation.module_ids_artifact, *module)
            .expect("should have module id")
        ),
        module.to_string(),
        module.to_string()
      ))
    } else {
      RawStringSource::from(format!(
        "{client}
        var resolveSelf, onError;
        module.exports = new Promise(function(resolve, reject) {{ resolveSelf = resolve; onError = reject; }});
        if (module.hot) {{
          module.hot.accept();
          if (module.hot.data && module.hot.data.resolveSelf) module.hot.data.resolveSelf(module.exports);
          module.hot.dispose(function(data) {{ data.resolveSelf = resolveSelf; dispose(data); }});
        }}
        {activate_code}
      "
      ))
    };

    let mut codegen_result = CodeGenerationResult::default().with_javascript(Arc::new(source));
    codegen_result.runtime_requirements = runtime_requirements;
    codegen_result.data = codegen_data;

    Ok(codegen_result)
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
