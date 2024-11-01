mod create_script_url_dependency;
pub use create_script_url_dependency::CreateScriptUrlDependency;
use rspack_core::{
  get_chunk_from_ukey, AsContextDependency, Compilation, Dependency, DependencyCategory,
  DependencyId, DependencyRange, DependencyTemplate, DependencyType, ExtendedReferencedExport,
  ModuleDependency, ModuleGraph, RuntimeGlobals, RuntimeSpec, TemplateContext,
  TemplateReplaceSource,
};
use rspack_util::ext::DynHash;

#[derive(Debug, Clone)]
pub struct WorkerDependency {
  id: DependencyId,
  request: String,
  public_path: String,
  range: DependencyRange,
  range_path: DependencyRange,
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
    }
  }
}

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
}

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
      .map(|entrypoint| entrypoint.get_entry_point_chunk())
      .and_then(|ukey| get_chunk_from_ukey(&ukey, &compilation.chunk_by_ukey))
      .and_then(|chunk| chunk.id.as_deref())
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
