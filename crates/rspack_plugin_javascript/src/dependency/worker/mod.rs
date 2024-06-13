use rspack_core::{
  get_chunk_from_ukey, AsContextDependency, Dependency, DependencyCategory, DependencyId,
  DependencyTemplate, DependencyType, ErrorSpan, ExtendedReferencedExport, ModuleDependency,
  ModuleGraph, RuntimeGlobals, RuntimeSpec, TemplateContext, TemplateReplaceSource,
};

#[derive(Debug, Clone)]
pub struct WorkerDependency {
  start: u32,
  end: u32,
  id: DependencyId,
  request: String,
  span: Option<ErrorSpan>,
  public_path: String,
}

impl WorkerDependency {
  pub fn new(
    start: u32,
    end: u32,
    request: String,
    public_path: String,
    span: Option<ErrorSpan>,
  ) -> Self {
    Self {
      start,
      end,
      id: DependencyId::new(),
      request,
      span,
      public_path,
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

  fn span(&self) -> Option<ErrorSpan> {
    self.span
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    vec![]
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
      self.start,
      self.end,
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
}

impl AsContextDependency for WorkerDependency {}
