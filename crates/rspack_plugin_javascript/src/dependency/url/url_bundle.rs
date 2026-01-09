use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, Compilation, Dependency, DependencyCategory, DependencyCodeGeneration,
  DependencyId, DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType,
  ExtendedReferencedExport, FactorizeInfo, ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact,
  RuntimeGlobals, RuntimeSpec, TemplateContext, TemplateReplaceSource,
};
use rspack_util::ext::DynHash;

/// Dependency for `new URL()` when in bundle mode.
/// This creates an async block that bundles the target module as a single-file entry.
#[cacheable]
#[derive(Debug, Clone)]
pub struct URLBundleDependency {
  id: DependencyId,
  request: String,
  public_path: String,
  range: DependencyRange,
  range_url: DependencyRange,
  factorize_info: FactorizeInfo,
  /// Whether to render using relative URL (when url mode is NewUrlRelative)
  use_relative: bool,
}

impl URLBundleDependency {
  pub fn new(
    request: String,
    public_path: String,
    range: DependencyRange,
    range_url: DependencyRange,
    use_relative: bool,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      public_path,
      range,
      range_url,
      factorize_info: Default::default(),
      use_relative,
    }
  }
}

#[cacheable_dyn]
impl Dependency for URLBundleDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Url
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::NewUrlBundle
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
impl ModuleDependency for URLBundleDependency {
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
impl DependencyCodeGeneration for URLBundleDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(URLBundleDependencyTemplate::template_type())
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

impl AsContextDependency for URLBundleDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct URLBundleDependencyTemplate;

impl URLBundleDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::NewUrlBundle)
  }
}

impl DependencyTemplate for URLBundleDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<URLBundleDependency>()
      .expect("URLBundleDependencyTemplate should be used for URLBundleDependency");
    let TemplateContext {
      compilation,
      runtime_requirements,
      ..
    } = code_generatable_context;

    // Get the chunk ID from the async block
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

    runtime_requirements.insert(RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME);

    let url_import_str = if dep.use_relative {
      // Render with relative path using new URL(filename, import.meta.url) pattern
      format!(
        "/* url bundle import */ new URL({}({}), import.meta.url)",
        compilation
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME),
        chunk_id
      )
    } else {
      // Standard rendering with PUBLIC_PATH and BASE_URI
      let url_base = if !dep.public_path.is_empty() {
        format!("\"{}\"", dep.public_path)
      } else {
        compilation
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::PUBLIC_PATH)
      };

      runtime_requirements.insert(RuntimeGlobals::PUBLIC_PATH);
      runtime_requirements.insert(RuntimeGlobals::BASE_URI);

      // Generate: new URL(publicPath + __webpack_require__.u(chunkId), __webpack_require__.b)
      format!(
        "/* url bundle import */ new URL({} + {}({}), {})",
        url_base,
        compilation
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME),
        chunk_id,
        compilation
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::BASE_URI)
      )
    };

    source.replace(
      dep.range.start,
      dep.range.end,
      url_import_str.as_str(),
      None,
    );
  }
}
