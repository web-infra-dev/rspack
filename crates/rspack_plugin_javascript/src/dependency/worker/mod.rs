mod create_script_url_dependency;
use std::sync::LazyLock;

pub use create_script_url_dependency::{
  CreateScriptUrlDependency, CreateScriptUrlDependencyTemplate,
};
use regex::Regex;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, CodeGenerationPublicPathAutoReplace, Compilation, Dependency,
  DependencyCategory, DependencyCodeGeneration, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, ExportsInfoArtifact, ExtendedReferencedExport,
  FactorizeInfo, JavascriptParserWorkerUrl, ModuleDependency, ModuleGraph,
  ModuleGraphCacheArtifact, PublicPath, RuntimeGlobals, RuntimeSpec, TemplateContext,
  TemplateReplaceSource, URLStaticMode,
};
use rspack_hash::{RspackHash, RspackHasher};

use crate::runtime::AUTO_PUBLIC_PATH_PLACEHOLDER;

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
  url_mode: Option<JavascriptParserWorkerUrl>,
}

impl WorkerDependency {
  pub fn new(
    request: String,
    public_path: String,
    range: DependencyRange,
    range_path: DependencyRange,
    need_new_url: bool,
    url_mode: Option<JavascriptParserWorkerUrl>,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      public_path,
      range,
      range_path,
      factorize_info: Default::default(),
      need_new_url,
      url_mode,
    }
  }

  pub fn public_path(&self) -> &str {
    &self.public_path
  }
}

impl RspackHash for WorkerDependency {
  fn hash(&self, state: &mut RspackHasher) {
    rspack_hash::rspack_hash_object!(state, {
      "publicPath" => self.public_path.as_str(),
      "needNewUrl" => self.need_new_url,
      "urlMode" => self.url_mode,
    });
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
    _exports_info_artifact: &ExportsInfoArtifact,
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
    hasher: &mut RspackHasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    RspackHash::hash(self, hasher);
  }
}

impl AsContextDependency for WorkerDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct WorkerDependencyTemplate;

pub static WORKER_STATIC_URL_PLACEHOLDER: &str = "RSPACK_AUTO_WORKER_STATIC_URL_PLACEHOLDER_";
pub static WORKER_STATIC_URL_PLACEHOLDER_RE: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(&format!(r#"{WORKER_STATIC_URL_PLACEHOLDER}(?<dep>\d+)"#))
    .expect("should be valid regex")
});

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
      runtime_template,
      ..
    } = code_generatable_context;
    let chunk_id = compilation
      .get_module_graph()
      .get_parent_block(&dep.id)
      .and_then(|block| {
        compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .get_block_chunk_group(
            block,
            &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
          )
      })
      .map(|entrypoint| entrypoint.get_entrypoint_chunk())
      .and_then(|ukey| {
        compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .get(&ukey)
      })
      .and_then(|chunk| chunk.id())
      .map(rspack_util::json_stringify)
      .expect("failed to get json stringified chunk id");
    let mut worker_import_str = if matches!(
      dep.url_mode,
      Some(JavascriptParserWorkerUrl::NewUrlRelative)
    ) {
      code_generatable_context.data.insert(URLStaticMode);
      let public_path = if !dep.public_path.is_empty() {
        dep.public_path.clone()
      } else if matches!(&compilation.options.output.public_path, PublicPath::Auto) {
        code_generatable_context
          .data
          .insert(CodeGenerationPublicPathAutoReplace(true));
        AUTO_PUBLIC_PATH_PLACEHOLDER.to_string()
      } else {
        String::new()
      };

      format!(
        "{}, import.meta.url",
        rspack_util::json_stringify_str(&format!(
          "{public_path}{WORKER_STATIC_URL_PLACEHOLDER}{}",
          dep.id.as_u32()
        ))
      )
    } else {
      let worker_import_base_url = if !dep.public_path.is_empty() {
        format!("\"{}\"", dep.public_path)
      } else {
        runtime_template.render_runtime_globals(&RuntimeGlobals::PUBLIC_PATH)
      };

      format!(
        "/* worker import */{} + {}({}), {}",
        worker_import_base_url,
        runtime_template.render_runtime_globals(&RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME),
        chunk_id,
        runtime_template.render_runtime_globals(&RuntimeGlobals::BASE_URI)
      )
    };

    if dep.need_new_url {
      worker_import_str = format!("new URL({worker_import_str})");
    }

    source.replace(
      dep.range_path.start,
      dep.range_path.end,
      worker_import_str,
      None,
    );
  }
}
