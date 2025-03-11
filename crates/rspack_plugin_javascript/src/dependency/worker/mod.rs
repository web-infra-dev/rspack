mod create_script_url_dependency;
pub use create_script_url_dependency::CreateScriptUrlDependency;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, Compilation, Dependency, DependencyCategory, DependencyId, DependencyRange,
  DependencyTemplate, DependencyType, ExtendedReferencedExport, FactorizeInfo, ModuleDependency,
  ModuleGraph, RuntimeGlobals, RuntimeSpec, TemplateContext, TemplateReplaceSource,
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
}

impl WorkerDependency {
  pub fn new(
    request: String,
    public_path: String,
    range: DependencyRange,
    range_path: DependencyRange,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      public_path,
      range,
      range_path,
      factorize_info: Default::default(),
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

  fn range(&self) -> Option<&DependencyRange> {
    Some(&self.range)
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
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
impl DependencyTemplate for WorkerDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext {
      compilation,
      runtime_requirements,
      ..
    } = code_generatable_context;
    let chunk_id = compilation
      .get_module_graph()
      .get_parent_block(&self.id)
      .and_then(|block| {
        compilation
          .chunk_graph
          .get_block_chunk_group(block, &compilation.chunk_group_by_ukey)
      })
      .map(|entrypoint| entrypoint.get_entrypoint_chunk())
      .and_then(|ukey| compilation.chunk_by_ukey.get(&ukey))
      .and_then(|chunk| chunk.id(&compilation.chunk_ids_artifact))
      .and_then(|chunk_id| serde_json::to_string(chunk_id).ok())
      .expect("failed to get json stringified chunk id");
    let worker_import_base_url = if !self.public_path.is_empty() {
      format!("\"{}\"", self.public_path)
    } else {
      RuntimeGlobals::PUBLIC_PATH.to_string()
    };

    runtime_requirements.insert(RuntimeGlobals::PUBLIC_PATH);
    runtime_requirements.insert(RuntimeGlobals::BASE_URI);
    runtime_requirements.insert(RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME);

    source.replace(
      self.range_path.start,
      self.range_path.end,
      format!(
        "/* worker import */{} + {}({}), {}",
        worker_import_base_url,
        RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME,
        chunk_id,
        RuntimeGlobals::BASE_URI
      )
      .as_str(),
      None,
    );
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
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
