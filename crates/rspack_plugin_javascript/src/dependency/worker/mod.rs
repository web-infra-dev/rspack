mod create_script_url_dependency;
pub use create_script_url_dependency::{
  CreateScriptUrlDependency, CreateScriptUrlDependencyTemplate,
};
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, Compilation, Dependency, DependencyCategory, DependencyCodeGeneration,
  DependencyId, DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType,
  ExtendedReferencedExport, FactorizeInfo, ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact,
  RuntimeGlobals, RuntimeSpec, TemplateContext, TemplateReplaceSource,
};
use rspack_util::ext::DynHash;

#[cacheable]
#[derive(Debug, Clone)]
pub struct WorkerDependency {
  id: DependencyId,
  request: String,
  public_path: String,
  range: DependencyRange,
  range_path: DependencyRange,
  factorize_info: FactorizeInfo,
  need_new_url: bool,
}

impl WorkerDependency {
  pub fn new(
    request: String,
    public_path: String,
    range: DependencyRange,
    range_path: DependencyRange,
    need_new_url: bool,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      public_path,
      range,
      range_path,
      factorize_info: Default::default(),
      need_new_url,
    }
  }
}

#[cacheable_dyn]
impl Dependency for WorkerDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Worker
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::NewWorker
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    vec![]
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for WorkerDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for WorkerDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(WorkerDependencyTemplate::template_type())
  }

  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    self.public_path.dyn_hash(hasher);
  }
}

impl AsContextDependency for WorkerDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct WorkerDependencyTemplate;

impl WorkerDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::NewWorker)
  }
}

impl DependencyTemplate for WorkerDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<WorkerDependency>()
      .expect("WorkerDependencyTemplate should be used for WorkerDependency");
    let TemplateContext {
      compilation,
      runtime_requirements,
      ..
    } = code_generatable_context;
    let chunk_id = compilation
      .get_module_graph()
      .get_parent_block(&dep.id)
      .and_then(|block| {
        compilation
          .chunk_graph
          .get_block_chunk_group(block, &compilation.chunk_group_by_ukey)
      })
      .map(|entrypoint| entrypoint.get_entrypoint_chunk())
      .and_then(|ukey| compilation.chunk_by_ukey.get(&ukey))
      .and_then(|chunk| chunk.id())
      .and_then(|chunk_id| serde_json::to_string(chunk_id).ok())
      .expect("failed to get json stringified chunk id");
    let worker_import_base_url = if !dep.public_path.is_empty() {
      format!("\"{}\"", dep.public_path)
    } else {
      compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::PUBLIC_PATH)
    };

    runtime_requirements.insert(RuntimeGlobals::PUBLIC_PATH);
    runtime_requirements.insert(RuntimeGlobals::BASE_URI);
    runtime_requirements.insert(RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME);

    let mut worker_import_str = format!(
      "/* worker import */{} + {}({}), {}",
      worker_import_base_url,
      compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME),
      chunk_id,
      compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::BASE_URI)
    );

    if dep.need_new_url {
      worker_import_str = format!("new URL({worker_import_str})");
    }

    source.replace(
      dep.range_path.start,
      dep.range_path.end,
      worker_import_str.as_str(),
      None,
    );
  }
}
