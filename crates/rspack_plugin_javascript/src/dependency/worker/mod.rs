use rspack_core::{
  module_id, module_id_expr, ChunkGroupOptions, ChunkLoading, CodeGeneratable,
  CodeGeneratableContext, CodeGeneratableResult, CodeReplaceSourceDependency,
  CodeReplaceSourceDependencyContext, CodeReplaceSourceDependencyReplaceSource, Dependency,
  DependencyCategory, DependencyId, DependencyType, EntryOptions, ErrorSpan, ModuleDependency,
  RuntimeGlobals,
};

#[derive(Debug, Clone)]
pub struct WorkerDependency {
  start: u32,
  end: u32,
  id: Option<DependencyId>,
  request: String,
  span: Option<ErrorSpan>,
  group_options: ChunkGroupOptions,
  public_path: String,
}

impl WorkerDependency {
  pub fn new(
    start: u32,
    end: u32,
    request: String,
    public_path: String,
    span: Option<ErrorSpan>,
    group_options: ChunkGroupOptions,
  ) -> Self {
    Self {
      start,
      end,
      id: None,
      request,
      span,
      group_options,
      public_path,
    }
  }
}

impl Dependency for WorkerDependency {
  fn id(&self) -> Option<DependencyId> {
    self.id
  }
  fn set_id(&mut self, id: Option<DependencyId>) {
    self.id = id;
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Worker
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::NewWorker
  }
}

impl ModuleDependency for WorkerDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    self.span.as_ref()
  }

  fn as_code_replace_source_dependency(&self) -> Option<Box<dyn CodeReplaceSourceDependency>> {
    Some(Box::new(self.clone()))
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }

  fn group_options(&self) -> Option<&ChunkGroupOptions> {
    Some(&self.group_options)
  }
}

impl CodeGeneratable for WorkerDependency {
  fn generate(
    &self,
    _code_generatable_context: &mut CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
    todo!()
  }
}

impl CodeReplaceSourceDependency for WorkerDependency {
  fn apply(
    &self,
    source: &mut CodeReplaceSourceDependencyReplaceSource,
    code_generatable_context: &mut CodeReplaceSourceDependencyContext,
  ) {
    let CodeReplaceSourceDependencyContext {
      compilation,
      runtime_requirements,
      ..
    } = code_generatable_context;
    let id: DependencyId = self.id().expect("should have dependency id");
    let chunk_id = compilation
      .module_graph
      .module_identifier_by_dependency_id(&id)
      .map(|module| {
        compilation
          .chunk_graph
          .get_block_chunk_group(module, &compilation.chunk_group_by_ukey)
      })
      .map(|entrypoint| entrypoint.get_entry_point_chunk())
      .and_then(|ukey| compilation.chunk_by_ukey.get(&ukey))
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
}
