use std::{
  any::{Any, TypeId},
  borrow::Cow,
  fmt::{Debug, Display, Formatter},
  hash::Hash,
  sync::Arc,
};

use async_trait::async_trait;
use json::JsonValue;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsMap, AsOption, AsPreset, AsVec},
};
use rspack_collections::{Identifiable, Identifier, IdentifierMap, IdentifierSet};
use rspack_error::{Diagnosable, Result};
use rspack_fs::ReadableFileSystem;
use rspack_hash::RspackHashDigest;
use rspack_paths::ArcPathSet;
use rspack_sources::BoxSource;
use rspack_util::{
  atom::Atom,
  ext::{AsAny, DynHash},
  fx_hash::{FxIndexMap, FxIndexSet},
  source_map::ModuleSourceMapConfig,
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use serde::Serialize;
use smol_str::SmolStr;
use swc_core::atoms::Wtf8Atom;

use crate::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, BindingCell, BoxDependency,
  BoxDependencyTemplate, BoxModuleDependency, ChunkGraph, ChunkUkey, CodeGenerationResult,
  CollectedTypeScriptInfo, Compilation, CompilationAsset, CompilationId, CompilerId,
  CompilerOptions, ConcatenationScope, ConnectionState, Context, ContextModule, DependenciesBlock,
  DependencyId, ExportProvided, ExportsInfoArtifact, ExternalModule, Filename, GetTargetResult,
  ImportPhase, ModuleCodeTemplate, ModuleGraph, ModuleGraphCacheArtifact, ModuleLayer, ModuleType,
  NormalModule, OptimizationBailoutItem, RawModule, Resolve, ResolverFactory, RuntimeSpec,
  SelfModule, SharedPluginDriver, SideEffectsStateArtifact, SourceType,
  concatenated_module::ConcatenatedModule, dependencies_block::dependencies_block_update_hash,
  get_target, value_cache_versions::ValueCacheVersions,
};

pub struct BuildContext {
  pub compiler_id: CompilerId,
  pub compilation_id: CompilationId,
  pub compiler_options: Arc<CompilerOptions>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub runtime_template: ModuleCodeTemplate,
  pub plugin_driver: SharedPluginDriver,
  pub fs: Arc<dyn ReadableFileSystem>,
}

#[cacheable]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RscModuleType {
  /// Represents a server entry module with "use server-entry" directive.
  ///
  /// Transformation flow:
  /// 1. Original module with "use server-entry" is transformed into a proxy module
  /// 2. The proxy module (with `module_type = ServerEntry`) imports the original implementation
  /// 3. The original implementation module may resulting in `module_type = Client`
  ///
  /// Note: "use server" and "use client" directives can coexist in the same file.
  ServerEntry,
  Server,
  Client,
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct RscMeta {
  pub module_type: RscModuleType,

  #[cacheable(with=AsVec<AsPreset>)]
  pub server_refs: Vec<Wtf8Atom>,

  #[cacheable(with=AsVec<AsPreset>)]
  pub client_refs: Vec<Wtf8Atom>,

  /// Whether this server component uses `import.meta.rspackRsc`.
  ///
  /// RSC client manifest collection uses this to find the module's transitive
  /// CSS dependencies, so they can be exposed through `entryCssFiles` and
  /// rendered by `loadCss()`.
  pub import_meta_rsc: bool,

  pub is_cjs: bool,

  #[cacheable(with=AsMap<AsPreset, AsPreset>)]
  pub action_ids: FxIndexMap<Atom, Atom>,
}

#[cacheable]
#[derive(Debug, Clone)]
pub enum CanonicalizedDataUrlOption {
  Source,
  Bytes,
  Asset(bool),
}

impl CanonicalizedDataUrlOption {
  pub fn is_source(&self) -> bool {
    matches!(self, Self::Source)
  }

  pub fn is_bytes(&self) -> bool {
    matches!(self, Self::Bytes)
  }

  pub fn is_inline(&self) -> bool {
    matches!(self, Self::Asset(true))
  }

  pub fn is_resource(&self) -> bool {
    matches!(self, Self::Asset(false))
  }
}

#[cacheable]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CssExport {
  #[cacheable(with=AsPreset)]
  pub ident: SmolStr,
  #[cacheable(with=AsOption<AsPreset>)]
  pub from: Option<SmolStr>,
  pub id: Option<DependencyId>,
  #[cacheable(with=AsPreset)]
  pub orig_name: SmolStr,
}

pub type CssExports = FxIndexMap<SmolStr, FxIndexSet<CssExport>>;
pub type CssLocalNames = HashMap<SmolStr, SmolStr>;

#[cacheable]
#[derive(Debug, Clone)]
pub enum CssLayer {
  Anonymous,
  Named(#[cacheable(with=AsPreset)] SmolStr),
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct CssModuleRenderCondition {
  #[cacheable(with=AsOption<AsPreset>)]
  pub media: Option<SmolStr>,
  #[cacheable(with=AsOption<AsPreset>)]
  pub supports: Option<SmolStr>,
  pub layer: Option<CssLayer>,
}

impl CssModuleRenderCondition {
  pub fn new(media: Option<SmolStr>, supports: Option<SmolStr>, layer: Option<CssLayer>) -> Self {
    Self {
      media,
      supports,
      layer,
    }
  }

  pub fn is_empty(&self) -> bool {
    self.media.is_none() && self.supports.is_none() && self.layer.is_none()
  }
}

pub fn iter_css_module_render_conditions<'a>(
  inherited_render_conditions: &'a [CssModuleRenderCondition],
  render_condition: &'a CssModuleRenderCondition,
) -> impl Iterator<Item = &'a CssModuleRenderCondition> {
  inherited_render_conditions
    .iter()
    .chain(std::iter::once(render_condition))
    .filter(|condition| !condition.is_empty())
}

pub fn css_module_render_conditions_identifier<'a>(
  conditions: impl IntoIterator<Item = &'a CssModuleRenderCondition>,
) -> Option<String> {
  let mut key = String::new();
  let mut count = 0;
  for condition in conditions
    .into_iter()
    .filter(|condition| !condition.is_empty())
  {
    count += 1;
    let layer = match &condition.layer {
      Some(CssLayer::Anonymous) => "<anonymous>",
      Some(CssLayer::Named(layer)) => layer.as_str(),
      None => "",
    };
    push_css_module_identifier_part(&mut key, layer);
    push_css_module_identifier_part(&mut key, condition.supports.as_deref().unwrap_or_default());
    push_css_module_identifier_part(&mut key, condition.media.as_deref().unwrap_or_default());
  }

  if count == 0 {
    None
  } else {
    Some(format!("conditions={count}{key}"))
  }
}

pub fn push_css_module_identifier_part(identifier: &mut String, value: &str) {
  identifier.push('|');
  identifier.push_str(&value.len().to_string());
  identifier.push(':');
  identifier.push_str(value);
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct CssBuildInfo {
  #[cacheable(with=AsMap<AsPreset, AsVec>)]
  pub exports: CssExports,
  #[cacheable(with=AsMap<AsPreset, AsPreset>)]
  pub local_names: CssLocalNames,
  /// Conditions inherited from parent CSS modules.
  ///
  /// Webpack stores the current module condition before inherited conditions.
  /// Rspack stores inherited conditions from outermost to innermost
  pub inherited_render_conditions: Vec<CssModuleRenderCondition>,
  pub render_condition: CssModuleRenderCondition,
}

impl CssBuildInfo {
  pub fn exports(&self) -> Option<&CssExports> {
    (!self.exports.is_empty()).then_some(&self.exports)
  }

  pub fn local_names(&self) -> Option<&CssLocalNames> {
    (!self.local_names.is_empty()).then_some(&self.local_names)
  }

  pub fn render_conditions(&self) -> impl Iterator<Item = &CssModuleRenderCondition> {
    iter_css_module_render_conditions(&self.inherited_render_conditions, &self.render_condition)
  }
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct IsolatedDts {
  pub resource_path: String,
  pub code: String,
  pub references: Vec<String>,
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct AssetBuildInfo {
  pub data_url: CanonicalizedDataUrlOption,
  pub filename: Option<Filename>,
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct BuildInfo {
  /// Whether the result is cacheable, i.e shared between builds.
  pub cacheable: bool,
  pub hash: Option<RspackHashDigest>,
  pub strict: bool,
  pub module_argument: ModuleArgument,
  pub exports_argument: ExportsArgument,
  pub file_dependencies: ArcPathSet,
  pub context_dependencies: ArcPathSet,
  pub missing_dependencies: ArcPathSet,
  pub build_dependencies: ArcPathSet,
  pub value_dependencies: HashMap<String, String>,
  #[cacheable(with=AsVec<AsPreset>)]
  pub esm_named_exports: HashSet<Atom>,
  pub all_star_exports: Vec<DependencyId>,
  pub need_create_require: bool,
  #[cacheable(with=AsOption<AsPreset>)]
  pub json_data: Option<JsonValue>,
  pub asset: Option<Box<AssetBuildInfo>>,
  pub css: Option<Box<CssBuildInfo>>,
  #[cacheable(with=AsOption<AsVec<AsPreset>>)]
  pub side_effects_free: Option<HashSet<Atom>>,
  #[cacheable(with=AsOption<AsVec<AsPreset>>)]
  pub top_level_declarations: Option<HashSet<Atom>>,
  pub module_concatenation_bailout: Option<String>,
  pub assets: BindingCell<HashMap<String, CompilationAsset>>,
  pub module: bool,
  pub inline_exports: bool,
  pub collected_typescript_info: Option<CollectedTypeScriptInfo>,
  pub rsc: Option<RscMeta>,
  pub import_phase: ImportPhase,
  pub isolated_dts: Option<Box<IsolatedDts>>,
  /// Stores external fields from the JS side (Record<string, any>),
  /// while other properties are stored in KnownBuildInfo.
  #[cacheable(with=AsPreset)]
  pub extras: serde_json::Map<String, serde_json::Value>,
  #[cacheable(with=AsVec)]
  pub deferred_pure_checks: HashSet<DeferredPureCheck>,
}

impl Default for BuildInfo {
  fn default() -> Self {
    Self {
      cacheable: true,
      hash: None,
      strict: false,
      module_argument: Default::default(),
      exports_argument: Default::default(),
      file_dependencies: ArcPathSet::default(),
      context_dependencies: ArcPathSet::default(),
      missing_dependencies: ArcPathSet::default(),
      build_dependencies: ArcPathSet::default(),
      value_dependencies: HashMap::default(),
      esm_named_exports: HashSet::default(),
      all_star_exports: Vec::default(),
      need_create_require: false,
      json_data: None,
      asset: None,
      css: None,
      side_effects_free: None,
      top_level_declarations: None,
      module_concatenation_bailout: None,
      assets: Default::default(),
      module: false,
      inline_exports: false,
      collected_typescript_info: None,
      rsc: None,
      import_phase: ImportPhase::Evaluation,
      isolated_dts: None,
      extras: Default::default(),
      deferred_pure_checks: HashSet::default(),
    }
  }
}

#[cacheable]
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BuildMetaExportsType {
  #[default]
  Unset,
  Default,
  Namespace,
  Flagged,
  Dynamic,
}

impl Display for BuildMetaExportsType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let d = match self {
      BuildMetaExportsType::Unset => "unknown exports (runtime-defined)",
      BuildMetaExportsType::Default => "default exports",
      BuildMetaExportsType::Namespace => "namespace exports",
      BuildMetaExportsType::Flagged => "flagged exports",
      BuildMetaExportsType::Dynamic => "dynamic exports",
    };

    f.write_str(d)
  }
}

#[derive(Debug, Clone, Copy, Hash)]
pub enum ExportsType {
  DefaultOnly,
  Namespace,
  DefaultWithNamed,
  Dynamic,
}

#[cacheable]
#[derive(Debug, Default, Clone, Copy, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BuildMetaDefaultObject {
  #[default]
  False,
  Redirect,
  RedirectWarn,
}

#[cacheable]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeferredPureCheck {
  #[cacheable(with=AsPreset)]
  pub atom: Atom,
  pub dep_id: DependencyId,
  pub start: u32,
  pub end: u32,
}

#[cacheable]
#[derive(Debug, Default, Clone, Copy, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ModuleArgument {
  #[default]
  Module,
  RspackModule,
}

#[cacheable]
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ExportsArgument {
  #[default]
  Exports,
  RspackExports,
}

#[cacheable]
#[derive(Debug, Default, Clone, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildMeta {
  pub strict_esm_module: bool,
  // same as is_async https://github.com/webpack/webpack/blob/3919c844eca394d73ca930e4fc5506fb86e2b094/lib/Module.js#L107
  pub has_top_level_await: bool,
  pub esm: bool,
  pub exports_type: BuildMetaExportsType,
  pub default_object: BuildMetaDefaultObject,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub side_effect_free: Option<bool>,
}

// webpack build info
#[derive(Debug)]
pub struct BuildResult {
  pub module: BoxModule,
  /// Whether the result is cacheable, i.e shared between builds.
  pub dependencies: Vec<BoxDependency>,
  pub blocks: Vec<Box<AsyncDependenciesBlock>>,
  pub optimization_bailouts: Vec<OptimizationBailoutItem>,
}

#[cacheable]
#[derive(Debug, Default, Clone)]
pub struct FactoryMeta {
  pub side_effect_free: Option<bool>,
}

pub type ModuleIdentifier = Identifier;
pub type ResourceIdentifier = Identifier;

#[derive(Debug)]
pub struct ModuleCodeGenerationContext<'a> {
  pub compilation: &'a Compilation,
  pub runtime: Option<&'a RuntimeSpec>,
  pub concatenation_scope: Option<ConcatenationScope>,
  pub runtime_template: &'a mut ModuleCodeTemplate,
}

#[cacheable_dyn]
#[async_trait]
pub trait Module:
  Debug
  + Send
  + Sync
  + Any
  + AsAny
  + Identifiable
  + DependenciesBlock
  + Diagnosable
  + ModuleSourceMapConfig
{
  /// Defines what kind of module this is.
  fn module_type(&self) -> &ModuleType;

  /// Defines what kind of code generation results this module can generate.
  fn source_types(&self, module_graph: &ModuleGraph) -> &[SourceType];

  /// The source of the module. This could be optional, modules like the `NormalModule` can have the corresponding source.
  /// However, modules that is created from "nowhere" (e.g. `ExternalModule` and `MissingModule`) does not have its source.
  fn source(&self) -> Option<&BoxSource>;

  /// User readable identifier of the module.
  fn readable_identifier(&self, _context: &Context) -> Cow<'_, str>;

  /// The size of the original source, which will used as a parameter for code-splitting.
  /// Only when calculating the size of the RuntimeModule is the Compilation depended on
  fn size(&self, source_type: Option<&SourceType>, compilation: Option<&Compilation>) -> f64;

  /// The actual build of the module, which will be called by the `Compilation`.
  /// Build can also returns the dependencies of the module, which will be used by the `Compilation` to build the dependency graph.
  async fn build(
    self: Box<Self>,
    _build_context: BuildContext,
    _compilation: Option<&Compilation>,
  ) -> Result<BuildResult>;

  fn factory_meta(&self) -> Option<&FactoryMeta>;

  fn set_factory_meta(&mut self, factory_meta: FactoryMeta);

  fn build_info(&self) -> &BuildInfo;

  fn build_info_mut(&mut self) -> &mut BuildInfo;

  fn build_meta(&self) -> &BuildMeta;

  fn build_meta_mut(&mut self) -> &mut BuildMeta;

  fn get_exports_argument(&self) -> ExportsArgument {
    self.build_info().exports_argument
  }

  fn get_module_argument(&self) -> ModuleArgument {
    self.build_info().module_argument
  }

  fn get_exports_type(
    &self,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
    strict: bool,
  ) -> ExportsType {
    module_graph_cache.cached_get_exports_type((self.identifier(), strict), || {
      get_exports_type_impl(
        self.identifier(),
        self.build_meta(),
        module_graph,
        exports_info_artifact,
        strict,
      )
    })
  }

  fn get_strict_esm_module(&self) -> bool {
    self.build_meta().strict_esm_module
  }

  /// The actual code generation of the module, which will be called by the `Compilation`.
  /// The code generation result should not be cached as it is implemented elsewhere to
  /// provide a universal cache mechanism (time to invalidate cache, etc.)
  ///
  /// Code generation will often iterate through every `source_types` given by the module
  /// to provide multiple code generation results for different `source_type`s.
  async fn code_generation(
    &self,
    _code_generation_context: &mut ModuleCodeGenerationContext,
  ) -> Result<CodeGenerationResult>;

  /// Name matched against bundle-splitting conditions.
  fn name_for_condition(&self) -> Option<Box<str>> {
    // Align with https://github.com/webpack/webpack/blob/8241da7f1e75c5581ba535d127fa66aeb9eb2ac8/lib/Module.js#L852
    None
  }

  /// Update hash for cgm.hash (chunk graph module hash)
  /// Different cgm code generation result should have different cgm.hash,
  /// so this also accept compilation (mainly chunk graph) and runtime as args.
  /// (Difference with `impl Hash for Module`: this is just a part for calculating cgm.hash, not for Module itself)
  async fn get_runtime_hash(
    &self,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<RspackHashDigest>;

  fn lib_ident(&self, _options: LibIdentOptions) -> Option<Cow<'_, str>> {
    // Align with https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/Module.js#L845
    None
  }

  /// Code generation dependencies of the module, which means the code generation of this module
  /// depends on the code generation results of dependencies which are returned by this function.
  /// e.g `Css` module may rely on the code generation result of `CssUrlDependency` to re-direct
  /// the url of the referenced assets.
  fn get_code_generation_dependencies(&self) -> Option<&[BoxModuleDependency]> {
    None
  }

  fn get_presentational_dependencies(&self) -> Option<&[BoxDependencyTemplate]> {
    None
  }

  fn get_concatenation_bailout_reason(
    &self,
    _mg: &ModuleGraph,
    _cg: &ChunkGraph,
  ) -> Option<Cow<'static, str>> {
    Some(
      format!(
        "Module Concatenation is not implemented for {}",
        self.module_type()
      )
      .into(),
    )
  }

  /// Resolve options matched by module rules.
  /// e.g `javascript/esm` may have special resolving options like `fullySpecified`.
  /// `css` and `css/module` may have special resolving options like `preferRelative`.
  fn get_resolve_options(&self) -> Option<Arc<Resolve>> {
    None
  }

  fn get_context(&self) -> Option<Box<Context>> {
    None
  }

  fn get_layer(&self) -> Option<&ModuleLayer> {
    None
  }

  fn chunk_condition(&self, _chunk_key: &ChunkUkey, _compilation: &Compilation) -> Option<bool> {
    None
  }

  fn get_side_effects_connection_state(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _side_effects_state_artifact: &SideEffectsStateArtifact,
    _module_chain: &mut IdentifierSet,
    _connection_state_cache: &mut IdentifierMap<ConnectionState>,
  ) -> ConnectionState {
    ConnectionState::Active(true)
  }

  fn need_build(&self, value_cache_version: &ValueCacheVersions) -> bool {
    let build_info = self.build_info();
    !build_info.cacheable
      || value_cache_version.has_diff(&build_info.value_dependencies)
      || self.diagnostics().iter().any(|item| item.is_error())
  }

  fn need_id(&self) -> bool {
    true
  }
}

fn get_exports_type_impl(
  identifier: ModuleIdentifier,
  build_meta: &BuildMeta,
  mg: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
  strict: bool,
) -> ExportsType {
  let export_type = &build_meta.exports_type;
  let default_object = &build_meta.default_object;
  match export_type {
    BuildMetaExportsType::Flagged => {
      if strict {
        ExportsType::DefaultWithNamed
      } else {
        ExportsType::Namespace
      }
    }
    BuildMetaExportsType::Namespace => ExportsType::Namespace,
    BuildMetaExportsType::Default => match default_object {
      BuildMetaDefaultObject::Redirect => ExportsType::DefaultWithNamed,
      BuildMetaDefaultObject::RedirectWarn => {
        if strict {
          ExportsType::DefaultOnly
        } else {
          ExportsType::DefaultWithNamed
        }
      }
      BuildMetaDefaultObject::False => ExportsType::DefaultOnly,
    },
    BuildMetaExportsType::Dynamic => {
      if strict {
        ExportsType::DefaultWithNamed
      } else {
        fn handle_default(default_object: &BuildMetaDefaultObject) -> ExportsType {
          match default_object {
            BuildMetaDefaultObject::Redirect => ExportsType::DefaultWithNamed,
            BuildMetaDefaultObject::RedirectWarn => ExportsType::DefaultWithNamed,
            _ => ExportsType::DefaultOnly,
          }
        }

        let name = Atom::from("__esModule");
        let exports_info = exports_info_artifact.get_exports_info_optional(&identifier);
        if let Some(export_info) = exports_info.as_ref().map(|info| {
          info
            .as_data(exports_info_artifact)
            .get_read_only_export_info(&name)
        }) {
          if matches!(export_info.provided(), Some(ExportProvided::NotProvided)) {
            handle_default(default_object)
          } else {
            let Some(GetTargetResult::Target(target)) = get_target(
              export_info,
              mg,
              exports_info_artifact,
              &|_| true,
              &mut Default::default(),
            ) else {
              return ExportsType::Dynamic;
            };
            if target
              .export
              .and_then(|t| {
                if t.len() == 1 {
                  t.first().cloned()
                } else {
                  None
                }
              })
              .is_some_and(|v| v == "__esModule")
            {
              let Some(target_exports_type) = mg
                .module_by_identifier(&target.module)
                .map(|m| m.build_meta().exports_type)
              else {
                return ExportsType::Dynamic;
              };
              match target_exports_type {
                BuildMetaExportsType::Flagged | BuildMetaExportsType::Namespace => {
                  ExportsType::Namespace
                }
                BuildMetaExportsType::Default => handle_default(default_object),
                _ => ExportsType::Dynamic,
              }
            } else {
              ExportsType::Dynamic
            }
          }
        } else {
          ExportsType::DefaultWithNamed
        }
      }
    }
    // align to undefined
    BuildMetaExportsType::Unset => {
      if strict {
        ExportsType::DefaultWithNamed
      } else {
        ExportsType::Dynamic
      }
    }
  }
}

pub fn module_update_hash(
  module: &dyn Module,
  hasher: &mut dyn std::hash::Hasher,
  compilation: &Compilation,
  runtime: Option<&RuntimeSpec>,
) {
  let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
  chunk_graph
    .get_module_graph_hash(module, compilation, runtime)
    .dyn_hash(hasher);
  if let Some(deps) = module.get_presentational_dependencies() {
    for dep in deps {
      dep.update_hash(hasher, compilation, runtime);
    }
  }
  dependencies_block_update_hash(
    module.get_dependencies(),
    module.get_blocks(),
    hasher,
    compilation,
    runtime,
  );
}

pub trait ModuleExt {
  fn boxed(self) -> BoxModule;
}

impl<T: Module + 'static> ModuleExt for T {
  fn boxed(self) -> BoxModule {
    if TypeId::of::<T>() == TypeId::of::<NormalModule>() {
      let module = Box::new(self) as Box<dyn Any>;
      return BoxModule::Normal(
        module
          .downcast::<NormalModule>()
          .expect("module type id should match NormalModule"),
      );
    }

    if TypeId::of::<T>() == TypeId::of::<ContextModule>() {
      let module = Box::new(self) as Box<dyn Any>;
      return BoxModule::Context(
        module
          .downcast::<ContextModule>()
          .expect("module type id should match ContextModule"),
      );
    }

    if TypeId::of::<T>() == TypeId::of::<ExternalModule>() {
      let module = Box::new(self) as Box<dyn Any>;
      return BoxModule::External(
        module
          .downcast::<ExternalModule>()
          .expect("module type id should match ExternalModule"),
      );
    }

    if TypeId::of::<T>() == TypeId::of::<RawModule>() {
      let module = Box::new(self) as Box<dyn Any>;
      return BoxModule::Raw(
        module
          .downcast::<RawModule>()
          .expect("module type id should match RawModule"),
      );
    }

    if TypeId::of::<T>() == TypeId::of::<SelfModule>() {
      let module = Box::new(self) as Box<dyn Any>;
      return BoxModule::SelfModule(
        module
          .downcast::<SelfModule>()
          .expect("module type id should match SelfModule"),
      );
    }

    if TypeId::of::<T>() == TypeId::of::<ConcatenatedModule>() {
      let module = Box::new(self) as Box<dyn Any>;
      return BoxModule::Concatenated(
        module
          .downcast::<ConcatenatedModule>()
          .expect("module type id should match ConcatenatedModule"),
      );
    }

    BoxModule::Custom(Box::new(Box::new(self) as Box<dyn Module>))
  }
}

#[cacheable]
pub enum BoxModule {
  Normal(Box<NormalModule>),
  Context(Box<ContextModule>),
  External(Box<ExternalModule>),
  Raw(Box<RawModule>),
  SelfModule(Box<SelfModule>),
  Concatenated(Box<ConcatenatedModule>),
  // Boxed twice on purpose: storing a thin `Box<Box<dyn Module>>` keeps the
  // `BoxModule` enum the same size as the previous `Box<dyn Module>` newtype
  // (a single pointer + discriminant) instead of growing it to hold a fat
  // pointer. This avoids a cache regression when iterating large module maps.
  Custom(Box<Box<dyn Module>>),
}

impl BoxModule {
  /// Create a new BoxModule from a boxed Module trait object.
  pub fn new(module: Box<dyn Module>) -> Self {
    BoxModule::Custom(Box::new(module))
  }

  pub fn normal(module: NormalModule) -> Self {
    Self::Normal(Box::new(module))
  }

  pub fn context(module: ContextModule) -> Self {
    Self::Context(Box::new(module))
  }

  pub fn external(module: ExternalModule) -> Self {
    Self::External(Box::new(module))
  }

  pub fn raw(module: RawModule) -> Self {
    Self::Raw(Box::new(module))
  }

  pub fn self_module(module: SelfModule) -> Self {
    Self::SelfModule(Box::new(module))
  }

  pub fn concatenated(module: ConcatenatedModule) -> Self {
    Self::Concatenated(Box::new(module))
  }

  fn as_module(&self) -> &dyn Module {
    match self {
      Self::Normal(module) => module.as_ref(),
      Self::Context(module) => &**module,
      Self::External(module) => &**module,
      Self::Raw(module) => module.as_ref(),
      Self::SelfModule(module) => module.as_ref(),
      Self::Concatenated(module) => &**module,
      Self::Custom(module) => &***module,
    }
  }

  fn as_module_mut(&mut self) -> &mut dyn Module {
    match self {
      Self::Normal(module) => module.as_mut(),
      Self::Context(module) => &mut **module,
      Self::External(module) => &mut **module,
      Self::Raw(module) => module.as_mut(),
      Self::SelfModule(module) => module.as_mut(),
      Self::Concatenated(module) => &mut **module,
      Self::Custom(module) => &mut ***module,
    }
  }

  /// Whether the module needs a module id. Implemented as an inherent
  /// static-dispatch method so hot call sites resolve directly to the concrete
  /// implementation instead of going through `Deref` to `dyn Module`.
  pub fn need_id(&self) -> bool {
    match self {
      Self::Normal(module) => module.need_id(),
      Self::Context(module) => module.need_id(),
      Self::External(module) => module.need_id(),
      Self::Raw(module) => module.need_id(),
      Self::SelfModule(module) => module.need_id(),
      Self::Concatenated(module) => module.need_id(),
      Self::Custom(module) => module.need_id(),
    }
  }

  /// The module identifier. Implemented as an inherent static-dispatch method
  /// so hot call sites resolve directly to the concrete variant instead of
  /// going through `Deref` to `dyn Module` (which rebuilds a fat pointer in
  /// `as_module` and then performs a vtable call). This matters because
  /// `Identifiable` is not in scope at several hot iteration sites (e.g. the
  /// module-ids plugins), where `identifier()` would otherwise dispatch
  /// dynamically.
  pub fn identifier(&self) -> Identifier {
    match self {
      Self::Normal(module) => module.identifier(),
      Self::Context(module) => module.identifier(),
      Self::External(module) => module.identifier(),
      Self::Raw(module) => module.identifier(),
      Self::SelfModule(module) => module.identifier(),
      Self::Concatenated(module) => module.identifier(),
      Self::Custom(module) => module.identifier(),
    }
  }

  pub async fn build(
    self,
    build_context: BuildContext,
    compilation: Option<&Compilation>,
  ) -> Result<BuildResult> {
    match self {
      Self::Normal(module) => module.build(build_context, compilation).await,
      Self::Context(module) => module.build(build_context, compilation).await,
      Self::External(module) => module.build(build_context, compilation).await,
      Self::Raw(module) => module.build(build_context, compilation).await,
      Self::SelfModule(module) => module.build(build_context, compilation).await,
      Self::Concatenated(module) => module.build(build_context, compilation).await,
      Self::Custom(module) => module.build(build_context, compilation).await,
    }
  }

  pub fn module_type(&self) -> &ModuleType {
    match self {
      Self::Normal(module) => module.module_type(),
      Self::Context(module) => module.module_type(),
      Self::External(module) => module.module_type(),
      Self::Raw(module) => module.module_type(),
      Self::SelfModule(module) => module.module_type(),
      Self::Concatenated(module) => module.module_type(),
      Self::Custom(module) => module.module_type(),
    }
  }

  pub fn source(&self) -> Option<&BoxSource> {
    match self {
      Self::Normal(module) => module.source(),
      Self::Context(module) => module.source(),
      Self::External(module) => module.source(),
      Self::Raw(module) => module.source(),
      Self::SelfModule(module) => module.source(),
      Self::Concatenated(module) => module.source(),
      Self::Custom(module) => module.source(),
    }
  }

  pub fn factory_meta(&self) -> Option<&FactoryMeta> {
    match self {
      Self::Normal(module) => module.factory_meta(),
      Self::Context(module) => module.factory_meta(),
      Self::External(module) => module.factory_meta(),
      Self::Raw(module) => module.factory_meta(),
      Self::SelfModule(module) => module.factory_meta(),
      Self::Concatenated(module) => module.factory_meta(),
      Self::Custom(module) => module.factory_meta(),
    }
  }

  pub fn set_factory_meta(&mut self, factory_meta: FactoryMeta) {
    match self {
      Self::Normal(module) => module.set_factory_meta(factory_meta),
      Self::Context(module) => module.set_factory_meta(factory_meta),
      Self::External(module) => module.set_factory_meta(factory_meta),
      Self::Raw(module) => module.set_factory_meta(factory_meta),
      Self::SelfModule(module) => module.set_factory_meta(factory_meta),
      Self::Concatenated(module) => module.set_factory_meta(factory_meta),
      Self::Custom(module) => module.set_factory_meta(factory_meta),
    }
  }

  pub fn build_info(&self) -> &BuildInfo {
    match self {
      Self::Normal(module) => module.build_info(),
      Self::Context(module) => module.build_info(),
      Self::External(module) => module.build_info(),
      Self::Raw(module) => module.build_info(),
      Self::SelfModule(module) => module.build_info(),
      Self::Concatenated(module) => module.build_info(),
      Self::Custom(module) => module.build_info(),
    }
  }

  pub fn build_info_mut(&mut self) -> &mut BuildInfo {
    match self {
      Self::Normal(module) => module.build_info_mut(),
      Self::Context(module) => module.build_info_mut(),
      Self::External(module) => module.build_info_mut(),
      Self::Raw(module) => module.build_info_mut(),
      Self::SelfModule(module) => module.build_info_mut(),
      Self::Concatenated(module) => module.build_info_mut(),
      Self::Custom(module) => module.build_info_mut(),
    }
  }

  pub fn build_meta(&self) -> &BuildMeta {
    match self {
      Self::Normal(module) => module.build_meta(),
      Self::Context(module) => module.build_meta(),
      Self::External(module) => module.build_meta(),
      Self::Raw(module) => module.build_meta(),
      Self::SelfModule(module) => module.build_meta(),
      Self::Concatenated(module) => module.build_meta(),
      Self::Custom(module) => module.build_meta(),
    }
  }

  pub fn build_meta_mut(&mut self) -> &mut BuildMeta {
    match self {
      Self::Normal(module) => module.build_meta_mut(),
      Self::Context(module) => module.build_meta_mut(),
      Self::External(module) => module.build_meta_mut(),
      Self::Raw(module) => module.build_meta_mut(),
      Self::SelfModule(module) => module.build_meta_mut(),
      Self::Concatenated(module) => module.build_meta_mut(),
      Self::Custom(module) => module.build_meta_mut(),
    }
  }

  pub fn get_dependencies(&self) -> &[DependencyId] {
    match self {
      Self::Normal(module) => module.get_dependencies(),
      Self::Context(module) => module.get_dependencies(),
      Self::External(module) => module.get_dependencies(),
      Self::Raw(module) => module.get_dependencies(),
      Self::SelfModule(module) => module.get_dependencies(),
      Self::Concatenated(module) => module.get_dependencies(),
      Self::Custom(module) => module.get_dependencies(),
    }
  }

  pub fn get_blocks(&self) -> &[AsyncDependenciesBlockIdentifier] {
    match self {
      Self::Normal(module) => module.get_blocks(),
      Self::Context(module) => module.get_blocks(),
      Self::External(module) => module.get_blocks(),
      Self::Raw(module) => module.get_blocks(),
      Self::SelfModule(module) => module.get_blocks(),
      Self::Concatenated(module) => module.get_blocks(),
      Self::Custom(module) => module.get_blocks(),
    }
  }

  pub fn add_dependency_id(&mut self, dependency: DependencyId) {
    match self {
      Self::Normal(module) => module.add_dependency_id(dependency),
      Self::Context(module) => module.add_dependency_id(dependency),
      Self::External(module) => module.add_dependency_id(dependency),
      Self::Raw(module) => module.add_dependency_id(dependency),
      Self::SelfModule(module) => module.add_dependency_id(dependency),
      Self::Concatenated(module) => module.add_dependency_id(dependency),
      Self::Custom(module) => module.add_dependency_id(dependency),
    }
  }

  pub fn add_block_id(&mut self, block: AsyncDependenciesBlockIdentifier) {
    match self {
      Self::Normal(module) => module.add_block_id(block),
      Self::Context(module) => module.add_block_id(block),
      Self::External(module) => module.add_block_id(block),
      Self::Raw(module) => module.add_block_id(block),
      Self::SelfModule(module) => module.add_block_id(block),
      Self::Concatenated(module) => module.add_block_id(block),
      Self::Custom(module) => module.add_block_id(block),
    }
  }

  pub fn as_normal_module(&self) -> Option<&NormalModule> {
    match self {
      Self::Normal(module) => Some(module.as_ref()),
      Self::Custom(module) => module.as_normal_module(),
      _ => None,
    }
  }

  pub fn as_normal_module_mut(&mut self) -> Option<&mut NormalModule> {
    match self {
      Self::Normal(module) => Some(module.as_mut()),
      Self::Custom(module) => module.as_normal_module_mut(),
      _ => None,
    }
  }

  pub fn as_context_module(&self) -> Option<&ContextModule> {
    match self {
      Self::Context(module) => Some(module),
      Self::Custom(module) => module.as_context_module(),
      _ => None,
    }
  }

  pub fn as_external_module(&self) -> Option<&ExternalModule> {
    match self {
      Self::External(module) => Some(module),
      Self::Custom(module) => module.as_external_module(),
      _ => None,
    }
  }

  pub fn as_raw_module(&self) -> Option<&RawModule> {
    match self {
      Self::Raw(module) => Some(module.as_ref()),
      Self::Custom(module) => module.as_raw_module(),
      _ => None,
    }
  }

  pub fn as_self_module(&self) -> Option<&SelfModule> {
    match self {
      Self::SelfModule(module) => Some(module.as_ref()),
      Self::Custom(module) => module.as_self_module(),
      _ => None,
    }
  }

  pub fn as_concatenated_module(&self) -> Option<&ConcatenatedModule> {
    match self {
      Self::Concatenated(module) => Some(module),
      Self::Custom(module) => module.as_concatenated_module(),
      _ => None,
    }
  }
}

impl std::ops::Deref for BoxModule {
  type Target = dyn Module;

  fn deref(&self) -> &Self::Target {
    self.as_module()
  }
}

impl std::ops::DerefMut for BoxModule {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.as_module_mut()
  }
}

impl From<Box<dyn Module>> for BoxModule {
  fn from(inner: Box<dyn Module>) -> Self {
    BoxModule::Custom(Box::new(inner))
  }
}

impl AsRef<dyn Module> for BoxModule {
  fn as_ref(&self) -> &dyn Module {
    self.as_module()
  }
}

impl AsMut<dyn Module> for BoxModule {
  fn as_mut(&mut self) -> &mut dyn Module {
    self.as_module_mut()
  }
}

impl Debug for BoxModule {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    self.as_module().fmt(f)
  }
}

impl Identifiable for BoxModule {
  /// Uniquely identify a module. If two modules share the same module identifier, then they are considered as the same module.
  /// e.g `javascript/auto|<absolute-path>/index.js` and `javascript/auto|<absolute-path>/index.js` are considered as the same.
  fn identifier(&self) -> Identifier {
    match self {
      Self::Normal(module) => module.identifier(),
      Self::Context(module) => module.identifier(),
      Self::External(module) => module.identifier(),
      Self::Raw(module) => module.identifier(),
      Self::SelfModule(module) => module.identifier(),
      Self::Concatenated(module) => module.identifier(),
      Self::Custom(module) => module.identifier(),
    }
  }
}

impl dyn Module {
  pub fn downcast_ref<T: Module + Any>(&self) -> Option<&T> {
    self.as_any().downcast_ref::<T>()
  }

  pub fn downcast_mut<T: Module + Any>(&mut self) -> Option<&mut T> {
    self.as_any_mut().downcast_mut::<T>()
  }
}

#[macro_export]
macro_rules! impl_module_meta_info {
  () => {
    fn factory_meta(&self) -> Option<&$crate::FactoryMeta> {
      self.factory_meta.as_ref()
    }

    fn set_factory_meta(&mut self, v: $crate::FactoryMeta) {
      self.factory_meta = Some(v);
    }

    fn build_info(&self) -> &$crate::BuildInfo {
      &self.build_info
    }

    fn build_info_mut(&mut self) -> &mut $crate::BuildInfo {
      &mut self.build_info
    }

    fn build_meta(&self) -> &$crate::BuildMeta {
      &self.build_meta
    }

    fn build_meta_mut(&mut self) -> &mut $crate::BuildMeta {
      &mut self.build_meta
    }
  };
}

macro_rules! impl_module_downcast_helpers {
  ($ty:ty, $ident:ident) => {
    impl dyn Module {
      ::paste::paste! {
        pub fn [<as_ $ident>](&self) -> Option<&$ty> {
          self.as_any().downcast_ref::<$ty>()
        }

        pub fn [<as_ $ident _mut>](&mut self) -> Option<&mut $ty> {
          self.as_any_mut().downcast_mut::<$ty>()
        }

        pub fn [<try_as_ $ident>](&self) -> Result<&$ty> {
          self.[<as_ $ident>]().ok_or_else(|| {
            ::rspack_error::error!(
              "Failed to cast module to a {}",
              stringify!($ty)
            )
          })
        }

        pub fn [<try_as_ $ident _mut>](&mut self) -> Result<&mut $ty> {
          self.[<as_ $ident _mut>]().ok_or_else(|| {
            ::rspack_error::error!(
              "Failed to cast module to a {}",
              stringify!($ty)
            )
          })
        }
      }
    }
  };
}

impl_module_downcast_helpers!(NormalModule, normal_module);
impl_module_downcast_helpers!(RawModule, raw_module);
impl_module_downcast_helpers!(ContextModule, context_module);
impl_module_downcast_helpers!(ExternalModule, external_module);
impl_module_downcast_helpers!(SelfModule, self_module);
impl_module_downcast_helpers!(ConcatenatedModule, concatenated_module);

pub struct LibIdentOptions<'me> {
  pub context: &'me str,
}

#[cfg(test)]
mod test {
  use std::borrow::Cow;

  use rspack_cacheable::cacheable;
  use rspack_collections::{Identifiable, Identifier};
  use rspack_error::{Result, impl_empty_diagnosable_trait};
  use rspack_hash::RspackHashDigest;
  use rspack_sources::BoxSource;
  use rspack_util::source_map::{ModuleSourceMapConfig, SourceMapKind};

  use super::{BoxModule, Module};
  use crate::{
    AsyncDependenciesBlockIdentifier, BuildContext, BuildResult, CodeGenerationResult, Compilation,
    Context, DependenciesBlock, DependencyId, ModuleCodeGenerationContext, ModuleExt, ModuleGraph,
    ModuleType, RuntimeSpec, SourceType,
  };

  #[cacheable]
  #[derive(Debug)]
  struct RawModule(String);

  #[cacheable]
  #[derive(Debug)]
  struct ExternalModule(String);

  macro_rules! impl_noop_trait_module_type {
    ($ident: ident) => {
      impl Identifiable for $ident {
        fn identifier(&self) -> Identifier {
          self.0.clone().into()
        }
      }

      impl_empty_diagnosable_trait!($ident);

      impl DependenciesBlock for $ident {
        fn add_block_id(&mut self, _: AsyncDependenciesBlockIdentifier) {
          unreachable!()
        }

        fn get_blocks(&self) -> &[AsyncDependenciesBlockIdentifier] {
          unreachable!()
        }

        fn add_dependency_id(&mut self, _: DependencyId) {
          unreachable!()
        }

        fn remove_dependency_id(&mut self, _: DependencyId) {
          unreachable!()
        }

        fn get_dependencies(&self) -> &[DependencyId] {
          unreachable!()
        }
      }

      #[::rspack_cacheable::cacheable_dyn]
      #[::async_trait::async_trait]
      impl Module for $ident {
        fn module_type(&self) -> &ModuleType {
          unreachable!()
        }

        fn source_types(&self, _module_graph: &ModuleGraph) -> &[SourceType] {
          unreachable!()
        }

        fn source(&self) -> Option<&BoxSource> {
          unreachable!()
        }

        fn size(
          &self,
          _source_type: Option<&SourceType>,
          _compilation: Option<&Compilation>,
        ) -> f64 {
          unreachable!()
        }

        fn readable_identifier(&self, _context: &Context) -> Cow<'_, str> {
          self.0.clone().into()
        }

        async fn build(
          self: Box<Self>,
          _build_context: BuildContext,
          _compilation: Option<&Compilation>,
        ) -> Result<BuildResult> {
          unreachable!()
        }

        async fn get_runtime_hash(
          &self,
          _compilation: &Compilation,
          _runtime: Option<&RuntimeSpec>,
        ) -> Result<RspackHashDigest> {
          unreachable!()
        }

        async fn code_generation(
          &self,
          _code_generation_context: &mut ModuleCodeGenerationContext,
        ) -> Result<CodeGenerationResult> {
          unreachable!()
        }

        fn factory_meta(&self) -> Option<&crate::FactoryMeta> {
          unreachable!()
        }

        fn build_info(&self) -> &crate::BuildInfo {
          unreachable!()
        }

        fn build_info_mut(&mut self) -> &mut crate::BuildInfo {
          unreachable!()
        }

        fn build_meta(&self) -> &crate::BuildMeta {
          unreachable!()
        }

        fn build_meta_mut(&mut self) -> &mut crate::BuildMeta {
          unreachable!()
        }

        fn set_factory_meta(&mut self, _: crate::FactoryMeta) {
          unreachable!()
        }
      }

      impl ModuleSourceMapConfig for $ident {
        fn get_source_map_kind(&self) -> &SourceMapKind {
          unreachable!()
        }
        fn set_source_map_kind(&mut self, _source_map: SourceMapKind) {
          unreachable!()
        }
      }
    };
  }

  impl_noop_trait_module_type!(RawModule);
  impl_noop_trait_module_type!(ExternalModule);

  #[test]
  fn should_downcast_successfully() {
    let a: BoxModule = ExternalModule(String::from("a")).boxed();
    let b: BoxModule = RawModule(String::from("a")).boxed();

    assert!(a.downcast_ref::<ExternalModule>().is_some());
    assert!(b.downcast_ref::<RawModule>().is_some());

    let a = a.as_ref();
    let b = b.as_ref();
    assert!(a.downcast_ref::<ExternalModule>().is_some());
    assert!(b.downcast_ref::<RawModule>().is_some());
  }
}
