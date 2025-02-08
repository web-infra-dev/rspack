use std::fmt::Display;
use std::hash::Hash;
use std::sync::Arc;
use std::{any::Any, borrow::Cow, fmt::Debug};

use async_trait::async_trait;
use json::JsonValue;
use rspack_cacheable::with::AsPreset;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsOption, AsVec},
};
use rspack_collections::{Identifiable, Identifier, IdentifierSet};
use rspack_error::{Diagnosable, Diagnostic, Result};
use rspack_fs::ReadableFileSystem;
use rspack_hash::RspackHashDigest;
use rspack_paths::ArcPath;
use rspack_sources::Source;
use rspack_util::atom::Atom;
use rspack_util::ext::{AsAny, DynHash};
use rspack_util::source_map::ModuleSourceMapConfig;
use rustc_hash::FxHashSet as HashSet;
use serde::Serialize;

use crate::concatenated_module::ConcatenatedModule;
use crate::dependencies_block::dependencies_block_update_hash;
use crate::{
  AsyncDependenciesBlock, BoxDependency, ChunkGraph, ChunkUkey, CodeGenerationResult, Compilation,
  CompilationId, CompilerOptions, ConcatenationScope, ConnectionState, Context, ContextModule,
  DependenciesBlock, DependencyId, DependencyTemplate, ExportInfoProvided, ExternalModule,
  ModuleDependency, ModuleGraph, ModuleLayer, ModuleType, NormalModule, RawModule, Resolve,
  ResolverFactory, RuntimeSpec, SelfModule, SharedPluginDriver, SourceType,
};

pub struct BuildContext {
  pub compilation_id: CompilationId,
  pub compiler_options: Arc<CompilerOptions>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub plugin_driver: SharedPluginDriver,
  pub fs: Arc<dyn ReadableFileSystem>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum BuildExtraDataType {
  CssParserAndGenerator,
  AssetParserAndGenerator,
  JavaScriptParserAndGenerator,
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct BuildInfo {
  /// Whether the result is cacheable, i.e shared between builds.
  pub cacheable: bool,
  pub hash: Option<RspackHashDigest>,
  pub strict: bool,
  pub file_dependencies: HashSet<ArcPath>,
  pub context_dependencies: HashSet<ArcPath>,
  pub missing_dependencies: HashSet<ArcPath>,
  pub build_dependencies: HashSet<ArcPath>,
  #[cacheable(with=AsVec<AsPreset>)]
  pub esm_named_exports: HashSet<Atom>,
  pub all_star_exports: Vec<DependencyId>,
  pub need_create_require: bool,
  #[cacheable(with=AsOption<AsPreset>)]
  pub json_data: Option<JsonValue>,
  #[cacheable(with=AsOption<AsVec<AsPreset>>)]
  pub top_level_declarations: Option<HashSet<Atom>>,
  pub module_concatenation_bailout: Option<String>,
}

impl Default for BuildInfo {
  fn default() -> Self {
    Self {
      cacheable: true,
      hash: None,
      strict: false,
      file_dependencies: HashSet::default(),
      context_dependencies: HashSet::default(),
      missing_dependencies: HashSet::default(),
      build_dependencies: HashSet::default(),
      esm_named_exports: HashSet::default(),
      all_star_exports: Vec::default(),
      need_create_require: false,
      json_data: None,
      top_level_declarations: None,
      module_concatenation_bailout: None,
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
  RedirectWarn {
    // Whether to ignore the warning, should use false for most cases
    // Only ignore the cases that do not follow the standards but are
    // widely used by the community, making it difficult to migrate.
    // For example, JSON named exports.
    ignore: bool,
  },
}

#[cacheable]
#[derive(Debug, Default, Clone, Copy, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ModuleArgument {
  #[default]
  Module,
  WebpackModule,
}

impl Display for ModuleArgument {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ModuleArgument::Module => write!(f, "module"),
      ModuleArgument::WebpackModule => write!(f, "__webpack_module__"),
    }
  }
}

#[cacheable]
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ExportsArgument {
  #[default]
  Exports,
  WebpackExports,
}

impl Display for ExportsArgument {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ExportsArgument::Exports => write!(f, "exports"),
      ExportsArgument::WebpackExports => write!(f, "__webpack_exports__"),
    }
  }
}

#[cacheable]
#[derive(Debug, Default, Clone, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildMeta {
  pub strict_esm_module: bool,
  pub has_top_level_await: bool,
  pub esm: bool,
  pub exports_type: BuildMetaExportsType,
  pub default_object: BuildMetaDefaultObject,
  pub module_argument: ModuleArgument,
  pub exports_argument: ExportsArgument,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub side_effect_free: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub exports_final_name: Option<Vec<(String, String)>>,
}

// webpack build info
#[derive(Debug, Default)]
pub struct BuildResult {
  /// Whether the result is cacheable, i.e shared between builds.
  pub build_meta: BuildMeta,
  pub build_info: BuildInfo,
  pub dependencies: Vec<BoxDependency>,
  pub blocks: Vec<Box<AsyncDependenciesBlock>>,
  pub optimization_bailouts: Vec<String>,
}

#[cacheable]
#[derive(Debug, Default, Clone)]
pub struct FactoryMeta {
  pub side_effect_free: Option<bool>,
}

pub type ModuleIdentifier = Identifier;

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
  fn source_types(&self) -> &[SourceType];
  fn get_diagnostics(&self) -> Vec<Diagnostic>;
  /// The original source of the module. This could be optional, modules like the `NormalModule` can have the corresponding original source.
  /// However, modules that is created from "nowhere" (e.g. `ExternalModule` and `MissingModule`) does not have its original source.
  fn original_source(&self) -> Option<&dyn Source>;

  /// User readable identifier of the module.
  fn readable_identifier(&self, _context: &Context) -> Cow<str>;

  /// The size of the original source, which will used as a parameter for code-splitting.
  /// Only when calculating the size of the RuntimeModule is the Compilation depended on
  fn size(&self, source_type: Option<&SourceType>, compilation: Option<&Compilation>) -> f64;

  /// The actual build of the module, which will be called by the `Compilation`.
  /// Build can also returns the dependencies of the module, which will be used by the `Compilation` to build the dependency graph.
  async fn build(
    &mut self,
    _build_context: BuildContext,
    _compilation: Option<&Compilation>,
  ) -> Result<BuildResult> {
    Ok(BuildResult {
      build_info: Default::default(),
      build_meta: Default::default(),
      dependencies: Vec::new(),
      blocks: Vec::new(),
      optimization_bailouts: vec![],
    })
  }

  fn factory_meta(&self) -> Option<&FactoryMeta>;

  fn set_factory_meta(&mut self, factory_meta: FactoryMeta);

  fn build_info(&self) -> Option<&BuildInfo>;

  fn set_build_info(&mut self, build_info: BuildInfo);

  fn build_meta(&self) -> Option<&BuildMeta>;

  fn set_build_meta(&mut self, build_meta: BuildMeta);

  fn get_exports_argument(&self) -> ExportsArgument {
    self
      .build_meta()
      .as_ref()
      .map(|m| m.exports_argument)
      .unwrap_or_default()
  }

  fn get_module_argument(&self) -> ModuleArgument {
    self
      .build_meta()
      .as_ref()
      .map(|m| m.module_argument)
      .unwrap_or_default()
  }

  fn get_exports_type(&self, module_graph: &ModuleGraph, strict: bool) -> ExportsType {
    get_exports_type_impl(self.identifier(), self.build_meta(), module_graph, strict)
  }

  fn get_strict_esm_module(&self) -> bool {
    self
      .build_meta()
      .as_ref()
      .is_some_and(|m| m.strict_esm_module)
  }

  /// The actual code generation of the module, which will be called by the `Compilation`.
  /// The code generation result should not be cached as it is implemented elsewhere to
  /// provide a universal cache mechanism (time to invalidate cache, etc.)
  ///
  /// Code generation will often iterate through every `source_types` given by the module
  /// to provide multiple code generation results for different `source_type`s.
  async fn code_generation(
    &self,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
    _concatenation_scope: Option<ConcatenationScope>,
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
  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<()>;

  fn lib_ident(&self, _options: LibIdentOptions) -> Option<Cow<str>> {
    // Align with https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/Module.js#L845
    None
  }

  /// Code generation dependencies of the module, which means the code generation of this module
  /// depends on the code generation results of dependencies which are returned by this function.
  /// e.g `Css` module may rely on the code generation result of `CssUrlDependency` to re-direct
  /// the url of the referenced assets.
  fn get_code_generation_dependencies(&self) -> Option<&[Box<dyn ModuleDependency>]> {
    None
  }

  fn get_presentational_dependencies(&self) -> Option<&[Box<dyn DependencyTemplate>]> {
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
    _module_chain: &mut IdentifierSet,
  ) -> ConnectionState {
    ConnectionState::Bool(true)
  }

  fn need_build(&self) -> bool {
    if let Some(build_info) = self.build_info() {
      if !build_info.cacheable {
        return true;
      }
    }
    false
  }

  fn depends_on(&self, modified_file: &HashSet<ArcPath>) -> bool {
    if let Some(build_info) = self.build_info() {
      for item in modified_file {
        if build_info.file_dependencies.contains(item)
          || build_info.build_dependencies.contains(item)
          || build_info.context_dependencies.contains(item)
          || build_info.missing_dependencies.contains(item)
        {
          return true;
        }
      }
    }

    false
  }

  fn need_id(&self) -> bool {
    true
  }
}

fn get_exports_type_impl(
  identifier: ModuleIdentifier,
  build_meta: Option<&BuildMeta>,
  mg: &ModuleGraph,
  strict: bool,
) -> ExportsType {
  if let Some((export_type, default_object)) = build_meta
    .as_ref()
    .map(|m| (&m.exports_type, &m.default_object))
  {
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
        BuildMetaDefaultObject::RedirectWarn { .. } => {
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
              BuildMetaDefaultObject::RedirectWarn { .. } => ExportsType::DefaultWithNamed,
              _ => ExportsType::DefaultOnly,
            }
          }

          if let Some(export_info) =
            mg.get_read_only_export_info(&identifier, Atom::from("__esModule"))
          {
            if matches!(export_info.provided(mg), Some(ExportInfoProvided::False)) {
              handle_default(default_object)
            } else {
              let Some(target) = export_info.get_target(mg) else {
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
                  .and_then(|m| m.build_meta())
                  .map(|meta| meta.exports_type)
                else {
                  return ExportsType::Dynamic;
                };
                match target_exports_type {
                  BuildMetaExportsType::Flagged => ExportsType::Namespace,
                  BuildMetaExportsType::Namespace => ExportsType::Namespace,
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
  } else if strict {
    ExportsType::DefaultWithNamed
  } else {
    ExportsType::Dynamic
  }
}

pub fn module_update_hash(
  module: &dyn Module,
  hasher: &mut dyn std::hash::Hasher,
  compilation: &Compilation,
  runtime: Option<&RuntimeSpec>,
) {
  let chunk_graph = &compilation.chunk_graph;
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
  fn boxed(self) -> Box<dyn Module>;
}

impl<T: Module> ModuleExt for T {
  fn boxed(self) -> Box<dyn Module> {
    Box::new(self)
  }
}

pub type BoxModule = Box<dyn Module>;

impl Identifiable for Box<dyn Module> {
  /// Uniquely identify a module. If two modules share the same module identifier, then they are considered as the same module.
  /// e.g `javascript/auto|<absolute-path>/index.js` and `javascript/auto|<absolute-path>/index.js` are considered as the same.
  fn identifier(&self) -> Identifier {
    self.as_ref().identifier()
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

    fn build_info(&self) -> Option<&$crate::BuildInfo> {
      self.build_info.as_ref()
    }

    fn set_build_info(&mut self, v: $crate::BuildInfo) {
      self.build_info = Some(v);
    }

    fn build_meta(&self) -> Option<&$crate::BuildMeta> {
      self.build_meta.as_ref()
    }

    fn set_build_meta(&mut self, v: $crate::BuildMeta) {
      self.build_meta = Some(v);
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
  use rspack_error::{Diagnosable, Diagnostic, Result};
  use rspack_sources::Source;
  use rspack_util::source_map::{ModuleSourceMapConfig, SourceMapKind};

  use super::Module;
  use crate::{
    AsyncDependenciesBlockIdentifier, BuildContext, BuildResult, CodeGenerationResult, Compilation,
    ConcatenationScope, Context, DependenciesBlock, DependencyId, ModuleExt, ModuleType,
    RuntimeSpec, SourceType,
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

      impl Diagnosable for $ident {}

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

        fn source_types(&self) -> &[SourceType] {
          unreachable!()
        }

        fn original_source(&self) -> Option<&dyn Source> {
          unreachable!()
        }

        fn size(
          &self,
          _source_type: Option<&SourceType>,
          _compilation: Option<&Compilation>,
        ) -> f64 {
          unreachable!()
        }

        fn readable_identifier(&self, _context: &Context) -> Cow<str> {
          self.0.clone().into()
        }

        async fn build(
          &mut self,
          _build_context: BuildContext,
          _compilation: Option<&Compilation>,
        ) -> Result<BuildResult> {
          unreachable!()
        }

        fn get_diagnostics(&self) -> Vec<Diagnostic> {
          vec![]
        }

        fn update_hash(
          &self,
          _hasher: &mut dyn std::hash::Hasher,
          _compilation: &Compilation,
          _runtime: Option<&RuntimeSpec>,
        ) -> Result<()> {
          unreachable!()
        }

        async fn code_generation(
          &self,
          _compilation: &Compilation,
          _runtime: Option<&RuntimeSpec>,
          _concatenation_scope: Option<ConcatenationScope>,
        ) -> Result<CodeGenerationResult> {
          unreachable!()
        }

        fn factory_meta(&self) -> Option<&crate::FactoryMeta> {
          unreachable!()
        }

        fn build_info(&self) -> Option<&crate::BuildInfo> {
          unreachable!()
        }

        fn build_meta(&self) -> Option<&crate::BuildMeta> {
          unreachable!()
        }

        fn set_factory_meta(&mut self, _: crate::FactoryMeta) {
          unreachable!()
        }

        fn set_build_info(&mut self, _: crate::BuildInfo) {
          unreachable!()
        }

        fn set_build_meta(&mut self, _: crate::BuildMeta) {
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
    let a: Box<dyn Module> = ExternalModule(String::from("a")).boxed();
    let b: Box<dyn Module> = RawModule(String::from("a")).boxed();

    assert!(a.downcast_ref::<ExternalModule>().is_some());
    assert!(b.downcast_ref::<RawModule>().is_some());

    let a = a.as_ref();
    let b = b.as_ref();
    assert!(a.downcast_ref::<ExternalModule>().is_some());
    assert!(b.downcast_ref::<RawModule>().is_some());
  }
}
