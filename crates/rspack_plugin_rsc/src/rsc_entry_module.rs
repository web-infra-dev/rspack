use std::{borrow::Cow, fmt::Write, sync::Arc};

use async_trait::async_trait;
use cow_utils::CowUtils;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsCacheable, AsMap, AsVec},
};
use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, BoxDependency, BoxModule, BuildContext,
  BuildInfo, BuildMeta, BuildMetaExportsType, BuildResult, CodeGenerationResult, Compilation,
  Context, DependenciesBlock, Dependency, DependencyId, DependencyRange, FactoryMeta, ImportPhase,
  LibIdentOptions, Module, ModuleCodeGenerationContext, ModuleDependency, ModuleGraph,
  ModuleIdentifier, ModuleLayer, ModuleType, ReferencedSpecifier, RuntimeSpec, SourceType,
  contextify, impl_module_meta_info, impl_source_map_config, module_update_hash,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
};
use rspack_error::{Result, impl_empty_diagnosable_trait};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_plugin_javascript::dependency::ImportEagerDependency;
use rspack_util::{fx_hash::FxIndexSet, source_map::SourceMapKind};
use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::ecma::atoms::Atom;

use crate::{
  client_reference_dependency::ClientReferenceDependency,
  constants::LAYERS_NAMES,
  plugin_state::{ClientModuleImport, CssImportsPerServerEntry},
};

#[impl_source_map_config]
#[cacheable]
#[derive(Debug)]
pub struct RscEntryModule {
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,
  identifier: ModuleIdentifier,
  lib_ident: String,
  client_modules: Vec<ClientModuleImport>,
  #[cacheable(with=AsMap<AsCacheable, AsVec>)]
  css_imports_per_server_entry: CssImportsPerServerEntry,
  name: Arc<str>,
  /// When true, client modules are loaded eagerly (not as code-split points).
  is_server_side_rendering: bool,
  factory_meta: Option<FactoryMeta>,
  build_info: BuildInfo,
  build_meta: BuildMeta,
  layer: Option<ModuleLayer>,
}

impl RscEntryModule {
  pub fn new(
    name: Arc<str>,
    client_modules: Vec<ClientModuleImport>,
    css_imports_per_server_entry: CssImportsPerServerEntry,
    is_server_side_rendering: bool,
  ) -> Self {
    let lib_ident = format!("rspack/rsc-entry?name={}", &name);
    let mut server_css_modules = css_imports_per_server_entry
      .iter()
      .flat_map(|(server_entry, imports)| {
        imports
          .iter()
          .map(move |request| format!("{server_entry}: {request}"))
      })
      .collect::<Vec<_>>();
    server_css_modules.sort();
    let identifier = ModuleIdentifier::from(format!(
      "rsc entry ({}) [client: {}; server css: {}]",
      name,
      client_modules
        .iter()
        .map(|m| m.request.as_str())
        .collect::<Vec<_>>()
        .join(", "),
      server_css_modules.join(", ")
    ));
    let layer = if is_server_side_rendering {
      Some(LAYERS_NAMES.server_side_rendering.to_string())
    } else {
      None
    };

    Self {
      blocks: Vec::new(),
      dependencies: Vec::new(),
      identifier,
      lib_ident,
      client_modules,
      css_imports_per_server_entry,
      name,
      is_server_side_rendering,
      factory_meta: None,
      build_info: BuildInfo {
        strict: true,
        top_level_declarations: Some(FxHashSet::default()),
        ..Default::default()
      },
      build_meta: BuildMeta {
        exports_type: BuildMetaExportsType::Namespace,
        ..Default::default()
      },
      source_map_kind: SourceMapKind::empty(),
      layer,
    }
  }

  fn render_debug_comments(&self, compilation: &Compilation) -> String {
    let module_graph = compilation.get_module_graph();
    let referenced_exports_by_request = create_referenced_exports_by_request(&self.client_modules);
    let mut source = String::new();

    if self.is_server_side_rendering {
      for dep_id in self.get_dependencies() {
        let dependency = module_graph.dependency_by_id(dep_id);
        let dep = dependency
          .downcast_ref::<ImportEagerDependency>()
          .unwrap_or_else(|| {
            panic!(
              "Expected dependency of eager RscEntryModule to be ImportEagerDependency, got {:?}",
              dependency.dependency_type()
            )
          });
        append_debug_comment_for_request(
          &mut source,
          referenced_exports_by_request
            .get(dep.request())
            .map(String::as_str),
          compilation,
          dep.request(),
          "import() eager",
        );
      }

      return source;
    }

    for block_id in self.get_blocks() {
      let block = module_graph
        .block_by_id(block_id)
        .expect("should have block");

      for dependency_id in block.get_dependencies() {
        let dependency = module_graph.dependency_by_id(dependency_id);
        let dep = dependency
          .downcast_ref::<ClientReferenceDependency>()
          .unwrap_or_else(|| {
            panic!(
              "Expected dependency of RscEntryModule to be ClientReferenceDependency, got {:?}",
              dependency.dependency_type()
            )
          });

        append_debug_comment_for_request(
          &mut source,
          referenced_exports_by_request
            .get(dep.user_request())
            .map(String::as_str)
            .or_else(|| {
              self
                .css_imports_per_server_entry
                .values()
                .any(|imports| imports.contains(dep.user_request()))
                .then_some("side-effect")
            }),
          compilation,
          dep.user_request(),
          "import()",
        );
      }
    }

    source
  }
}

impl Identifiable for RscEntryModule {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}

impl DependenciesBlock for RscEntryModule {
  fn add_block_id(&mut self, block: AsyncDependenciesBlockIdentifier) {
    self.blocks.push(block)
  }

  fn get_blocks(&self) -> &[AsyncDependenciesBlockIdentifier] {
    &self.blocks
  }

  fn add_dependency_id(&mut self, dependency: DependencyId) {
    self.dependencies.push(dependency)
  }

  fn remove_dependency_id(&mut self, dependency: DependencyId) {
    self.dependencies.retain(|d| d != &dependency)
  }

  fn get_dependencies(&self) -> &[DependencyId] {
    &self.dependencies
  }
}

#[cacheable_dyn]
#[async_trait]
impl Module for RscEntryModule {
  impl_module_meta_info!();

  fn size(&self, _source_type: Option<&SourceType>, _compilation: Option<&Compilation>) -> f64 {
    42.0
  }

  fn module_type(&self) -> &ModuleType {
    &ModuleType::JsDynamic
  }

  fn source_types(&self, _module_graph: &ModuleGraph) -> &[SourceType] {
    &[SourceType::JavaScript]
  }

  fn source(&self) -> Option<&BoxSource> {
    None
  }

  fn readable_identifier(&self, _context: &Context) -> Cow<'_, str> {
    format!("rsc client entry {}", self.name).into()
  }

  fn lib_ident(&self, _options: LibIdentOptions) -> Option<Cow<'_, str>> {
    Some(self.lib_ident.as_str().into())
  }

  fn get_layer(&self) -> Option<&ModuleLayer> {
    self.layer.as_ref()
  }

  async fn build(
    mut self: Box<Self>,
    _build_context: BuildContext,
    _: Option<&Compilation>,
  ) -> Result<BuildResult> {
    if self.is_server_side_rendering {
      // Eager: no code-split points; use ImportEagerDependency (CSS filtering done at call site).
      let mut dependencies: Vec<BoxDependency> = Vec::with_capacity(self.client_modules.len());
      for client_module in &self.client_modules {
        let referenced_specifiers = create_referenced_specifiers(&client_module.ids);
        let dep = ImportEagerDependency::new(
          Atom::from(client_module.request.as_str()),
          DependencyRange { start: 0, end: 0 },
          referenced_specifiers,
          None,
          ImportPhase::Evaluation,
        );
        dependencies.push(Box::new(dep));
      }
      Ok(BuildResult {
        module: BoxModule::new(self),
        dependencies,
        blocks: vec![],
        optimization_bailouts: vec![],
      })
    } else {
      // Non-eager: code-split points; use AsyncDependenciesBlock + ClientReferenceDependency.
      let mut blocks =
        Vec::with_capacity(self.client_modules.len() + self.css_imports_per_server_entry.len());
      let dependencies: Vec<BoxDependency> = vec![];

      for client_module in &self.client_modules {
        let dep = ClientReferenceDependency::new(
          client_module.request.clone(),
          client_module.ids.clone(),
          self.is_server_side_rendering,
        );
        let block = AsyncDependenciesBlock::new(
          self.identifier,
          None,
          None,
          vec![Box::new(dep) as Box<dyn Dependency>],
          Some(client_module.request.clone()),
        );
        blocks.push(Box::new(block));
      }

      for (server_entry, css_imports) in &self.css_imports_per_server_entry {
        if css_imports.is_empty() {
          continue;
        }

        let dependencies = css_imports
          .iter()
          .map(|request| {
            Box::new(ClientReferenceDependency::new(
              request.clone(),
              Default::default(),
              self.is_server_side_rendering,
            )) as Box<dyn Dependency>
          })
          .collect::<Vec<_>>();

        let block = AsyncDependenciesBlock::new(
          self.identifier,
          None,
          None,
          dependencies,
          Some(server_entry.clone()),
        );
        blocks.push(Box::new(block));
      }

      Ok(BuildResult {
        module: BoxModule::new(self),
        dependencies,
        blocks,
        optimization_bailouts: vec![],
      })
    }
  }

  // RscEntryModule is the bridge injected by the Server Compiler into the
  // Client Compiler to connect Client Component and CSS module graphs.
  // It never emits runtime code; code generation only writes debug comments to
  // help diagnose RSC entry composition issues.
  async fn code_generation(
    &self,
    code_generation_context: &mut ModuleCodeGenerationContext,
  ) -> Result<CodeGenerationResult> {
    let compilation = code_generation_context.compilation;
    let source = self.render_debug_comments(compilation);

    Ok(CodeGenerationResult::default().with_javascript(RawStringSource::from(source).boxed()))
  }

  async fn get_runtime_hash(
    &self,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<RspackHashDigest> {
    let mut hasher = RspackHash::from(&compilation.options.output);
    module_update_hash(self, &mut hasher, compilation, runtime);
    Ok(hasher.digest(&compilation.options.output.hash_digest))
  }
}

impl_empty_diagnosable_trait!(RscEntryModule);

fn create_referenced_specifiers(ids: &FxIndexSet<Atom>) -> Option<Vec<ReferencedSpecifier>> {
  if ids.is_empty() || ids.iter().any(|id| id == "*") {
    return None;
  }

  Some(
    ids
      .iter()
      .map(|id| ReferencedSpecifier::new(vec![Atom::from(id.as_str())]))
      .collect(),
  )
}

fn create_referenced_exports_by_request(
  client_modules: &[ClientModuleImport],
) -> FxHashMap<&str, String> {
  client_modules
    .iter()
    .map(|client_module| {
      let exports = format_referenced_exports(client_module);
      (client_module.request.as_str(), exports)
    })
    .collect()
}

fn append_debug_comment_for_request(
  source: &mut String,
  exports: Option<&str>,
  compilation: &Compilation,
  request: &str,
  entry_load: &str,
) {
  let request = contextify(compilation.options.context.as_path(), request);
  append_debug_comment(source, &request, entry_load, exports.unwrap_or("unknown"));
}

fn sanitize_comment_part(value: &str) -> Cow<'_, str> {
  if value.contains("*/") {
    value.cow_replace("*/", "* /")
  } else {
    Cow::Borrowed(value)
  }
}

fn append_debug_comment(source: &mut String, request: &str, entry_load: &str, exports: &str) {
  if !source.is_empty() {
    source.push('\n');
  }

  let request = sanitize_comment_part(request);
  let entry_load = sanitize_comment_part(entry_load);
  let exports = sanitize_comment_part(exports);
  write!(
    source,
    "/*!\n * module: {request}\n * import: {entry_load}\n * exports: {exports}\n */"
  )
  .expect("writing debug comments to String should not fail");
}

fn format_referenced_exports(client_module: &ClientModuleImport) -> String {
  if client_module.ids.is_empty() {
    return "side-effect".to_string();
  }

  if client_module.ids.iter().any(|id| id == "*") {
    return "*".to_string();
  }

  let mut exports = String::new();
  for id in &client_module.ids {
    if !exports.is_empty() {
      exports.push_str(", ");
    }
    exports.push_str(id.as_str());
  }
  exports
}
