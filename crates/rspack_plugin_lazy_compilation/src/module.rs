use std::sync::Arc;

use cow_utils::CowUtils;
use rspack_collections::Identifiable;
use rspack_core::{
  impl_module_meta_info, module_namespace_promise, module_update_hash,
  rspack_sources::{RawSource, Source},
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, BoxDependency, BuildContext, BuildInfo,
  BuildMeta, BuildResult, CodeGenerationData, CodeGenerationResult, Compilation,
  ConcatenationScope, Context, DependenciesBlock, DependencyId, FactoryMeta, Module,
  ModuleFactoryCreateData, ModuleIdentifier, ModuleLayer, ModuleType, RealDependencyLocation,
  RuntimeGlobals, RuntimeSpec, SourceType, TemplateContext,
};
use rspack_error::{Diagnosable, Diagnostic, Result};
use rspack_plugin_javascript::dependency::CommonJsRequireDependency;
use rspack_util::{
  ext::DynHash,
  source_map::{ModuleSourceMapConfig, SourceMapKind},
};
use rustc_hash::FxHashSet;

use crate::dependency::LazyCompilationDependency;

static MODULE_TYPE: ModuleType = ModuleType::JsAuto;
static SOURCE_TYPE: [SourceType; 1] = [SourceType::JavaScript];

#[derive(Debug)]
pub(crate) struct LazyCompilationProxyModule {
  build_info: Option<BuildInfo>,
  build_meta: Option<BuildMeta>,
  factory_meta: Option<FactoryMeta>,
  cacheable: bool,

  readable_identifier: String,
  identifier: ModuleIdentifier,

  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,

  source_map_kind: SourceMapKind,
  create_data: ModuleFactoryCreateData,
  pub resource: String,

  pub active: bool,
  pub data: String,
  pub client: String,
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
  pub(crate) fn new(
    original_module: ModuleIdentifier,
    create_data: ModuleFactoryCreateData,
    resource: String,
    cacheable: bool,
    active: bool,
    data: String,
    client: String,
  ) -> Self {
    let readable_identifier = format!(
      "lazy-compilation-proxy|{}",
      create_data.context.shorten(&original_module)
    );
    let identifier = format!("lazy-compilation-proxy|{original_module}").into();

    Self {
      build_info: None,
      build_meta: None,
      cacheable,
      create_data,
      readable_identifier,
      resource,
      identifier,
      source_map_kind: SourceMapKind::empty(),
      factory_meta: None,
      blocks: vec![],
      dependencies: vec![],
      active,
      client,
      data,
    }
  }
}

impl Diagnosable for LazyCompilationProxyModule {
  fn add_diagnostic(&self, _diagnostic: Diagnostic) {
    unimplemented!()
  }
  fn add_diagnostics(&self, _diagnostics: Vec<Diagnostic>) {
    unimplemented!()
  }
}

#[async_trait::async_trait]
impl Module for LazyCompilationProxyModule {
  impl_module_meta_info!();

  fn source_types(&self) -> &[SourceType] {
    &SOURCE_TYPE
  }

  fn module_type(&self) -> &ModuleType {
    &MODULE_TYPE
  }

  fn get_layer(&self) -> Option<&ModuleLayer> {
    self.create_data.issuer_layer.as_ref()
  }

  fn size(&self, _source_type: Option<&SourceType>, _compilation: &Compilation) -> f64 {
    200f64
  }

  fn original_source(&self) -> Option<&dyn Source> {
    None
  }

  fn readable_identifier(&self, _context: &Context) -> std::borrow::Cow<str> {
    std::borrow::Cow::Borrowed(&self.readable_identifier)
  }

  fn get_diagnostics(&self) -> Vec<Diagnostic> {
    vec![]
  }

  async fn build(
    &mut self,
    _build_context: BuildContext<'_>,
    _compilation: Option<&Compilation>,
  ) -> Result<BuildResult> {
    let client_dep = CommonJsRequireDependency::new(
      self.client.clone(),
      RealDependencyLocation::new(0, 0),
      None,
      false,
    );
    let mut dependencies = vec![];
    let mut blocks = vec![];

    dependencies.push(Box::new(client_dep) as BoxDependency);

    if self.active {
      let dep = LazyCompilationDependency::new(self.create_data.clone());

      blocks.push(Box::new(AsyncDependenciesBlock::new(
        self.identifier,
        None,
        None,
        vec![Box::new(dep)],
        None,
      )));
    }

    let mut files = FxHashSet::default();
    files.extend(self.create_data.file_dependencies.clone());
    files.insert(self.resource.to_owned().into());

    Ok(BuildResult {
      build_info: BuildInfo {
        cacheable: self.cacheable,
        file_dependencies: files,
        ..Default::default()
      },
      build_meta: BuildMeta::default(),
      dependencies,
      blocks,
      optimization_bailouts: vec![],
    })
  }

  // #[tracing::instrument("LazyCompilationProxyModule::code_generation", skip_all, fields(identifier = ?self.identifier()))]
  fn code_generation(
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
    let chunk_graph = &compilation.chunk_graph;

    let client_module = module_graph
      .module_identifier_by_dependency_id(&client_dep_id)
      .expect("should have module");

    let block = self.blocks.first();

    let client = format!(
      "var client = __webpack_require__(\"{}\");\nvar data = \"{}\"",
      chunk_graph
        .get_module_id(*client_module)
        .as_ref()
        .expect("should have module id"),
      self.data
    );

    let keep_active = format!(
      "var dispose = client.keepAlive({{ data: data, active: {}, module: module, onError: onError }})",
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

      RawSource::from(format!(
        "{client}
        module.exports = {};
        if (module.hot) {{
          module.hot.accept();
          module.hot.accept(\"{}\", function() {{ module.hot.invalidate(); }});
          module.hot.dispose(function(data) {{ delete data.resolveSelf; dispose(data); }});
          if (module.hot.data && module.hot.data.resolveSelf)
            module.hot.data.resolveSelf(module.exports);
        }}
        function onError() {{ /* ignore */ }}
        {}
        ",
        module_namespace_promise(
          &mut template_ctx,
          &dep_id,
          Some(block_id),
          &self.resource,
          "import()",
          false
        ),
        chunk_graph
          .get_module_id(*module)
          .as_ref()
          .expect("should have module id")
          .cow_replace('"', r#"\""#),
        keep_active,
      ))
    } else {
      RawSource::from(format!(
        "{}
        var resolveSelf, onError;
        module.exports = new Promise(function(resolve, reject) {{ resolveSelf = resolve; onError = reject; }});
        if (module.hot) {{
          module.hot.accept();
          if (module.hot.data && module.hot.data.resolveSelf) module.hot.data.resolveSelf(module.exports);
          module.hot.dispose(function(data) {{ data.resolveSelf = resolveSelf; dispose(data); }});
        }}
        {}
      ",
        client,
        keep_active
      ))
    };

    let mut codegen_result = CodeGenerationResult::default().with_javascript(Arc::new(source));
    codegen_result.runtime_requirements = runtime_requirements;
    codegen_result.data = codegen_data;
    codegen_result.set_hash(
      &compilation.options.output.hash_function,
      &compilation.options.output.hash_digest,
      &compilation.options.output.hash_salt,
    );

    Ok(codegen_result)
  }

  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<()> {
    module_update_hash(self, hasher, compilation, runtime);
    self.active.dyn_hash(hasher);
    self.data.dyn_hash(hasher);
    Ok(())
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

  fn get_dependencies(&self) -> &[rspack_core::DependencyId] {
    &self.dependencies
  }
}
