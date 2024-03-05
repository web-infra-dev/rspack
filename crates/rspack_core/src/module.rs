use std::fmt::Display;
use std::hash::Hash;
use std::path::PathBuf;
use std::{any::Any, borrow::Cow, fmt::Debug};

use async_trait::async_trait;
use json::JsonValue;
use rspack_error::{Diagnosable, Diagnostic, Result};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_identifier::{Identifiable, Identifier};
use rspack_sources::Source;
use rspack_util::ext::{AsAny, DynEq, DynHash};
use rspack_util::source_map::ModuleSourceMapConfig;
use rustc_hash::FxHashSet as HashSet;
use swc_core::ecma::atoms::Atom;

use crate::tree_shaking::visitor::OptimizeAnalyzeResult;
use crate::{
  AsyncDependenciesBlock, BoxDependency, ChunkGraph, ChunkUkey, CodeGenerationResult, Compilation,
  CompilerContext, CompilerOptions, ConcatenationScope, ConnectionState, Context, ContextModule,
  DependenciesBlock, DependencyId, DependencyTemplate, ExportInfoProvided, ExternalModule,
  ImmutableModuleGraph, ModuleDependency, ModuleGraph, ModuleGraphAccessor, ModuleType,
  MutexModuleGraph, NormalModule, RawModule, Resolve, RuntimeSpec, SelfModule, SharedPluginDriver,
  SourceType,
};
pub struct BuildContext<'a> {
  pub compiler_context: CompilerContext,
  pub plugin_driver: SharedPluginDriver,
  pub compiler_options: &'a CompilerOptions,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum BuildExtraDataType {
  CssParserAndGenerator,
  AssetParserAndGenerator,
  JavaScriptParserAndGenerator,
}

#[derive(Debug, Clone)]
pub struct BuildInfo {
  /// Whether the result is cacheable, i.e shared between builds.
  pub cacheable: bool,
  pub hash: Option<RspackHashDigest>,
  pub strict: bool,
  pub file_dependencies: HashSet<PathBuf>,
  pub context_dependencies: HashSet<PathBuf>,
  pub missing_dependencies: HashSet<PathBuf>,
  pub build_dependencies: HashSet<PathBuf>,
  pub asset_filenames: HashSet<String>,
  pub harmony_named_exports: HashSet<Atom>,
  pub all_star_exports: Vec<DependencyId>,
  pub need_create_require: bool,
  pub json_data: Option<JsonValue>,
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
      asset_filenames: HashSet::default(),
      harmony_named_exports: HashSet::default(),
      all_star_exports: Vec::default(),
      need_create_require: false,
      json_data: None,
    }
  }
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
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

#[derive(Debug, Default, Clone, Copy, Hash)]
pub enum BuildMetaDefaultObject {
  #[default]
  False,
  Redirect,
  RedirectWarn,
}

#[derive(Debug, Default, Clone, Copy, Hash)]
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

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
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

#[derive(Debug, Default, Clone, Hash)]
pub struct BuildMeta {
  pub strict_harmony_module: bool,
  pub has_top_level_await: bool,
  pub esm: bool,
  pub exports_type: BuildMetaExportsType,
  pub default_object: BuildMetaDefaultObject,
  pub module_argument: ModuleArgument,
  pub exports_argument: ExportsArgument,
  pub side_effect_free: Option<bool>,
}

// webpack build info
#[derive(Debug, Default, Clone)]
pub struct BuildResult {
  /// Whether the result is cacheable, i.e shared between builds.
  pub build_meta: BuildMeta,
  pub build_info: BuildInfo,
  pub analyze_result: OptimizeAnalyzeResult,
  pub dependencies: Vec<BoxDependency>,
  pub blocks: Vec<AsyncDependenciesBlock>,
  pub optimization_bailouts: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct FactoryMeta {
  pub side_effect_free: Option<bool>,
}

pub type ModuleIdentifier = Identifier;
#[async_trait]
pub trait Module:
  Debug
  + Send
  + Sync
  + AsAny
  + DynHash
  + DynEq
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
  fn size(&self, _source_type: &SourceType) -> f64;

  /// The actual build of the module, which will be called by the `Compilation`.
  /// Build can also returns the dependencies of the module, which will be used by the `Compilation` to build the dependency graph.
  async fn build(
    &mut self,
    build_context: BuildContext<'_>,
    _compilation: Option<&Compilation>,
  ) -> Result<BuildResult> {
    let mut hasher = RspackHash::from(&build_context.compiler_options.output);
    self.update_hash(&mut hasher);

    let build_info = BuildInfo {
      hash: Some(hasher.digest(&build_context.compiler_options.output.hash_digest)),
      ..Default::default()
    };

    Ok(BuildResult {
      build_info,
      build_meta: Default::default(),
      dependencies: Vec::new(),
      blocks: Vec::new(),
      analyze_result: Default::default(),
      optimization_bailouts: vec![],
    })
  }

  fn build_info(&self) -> Option<&BuildInfo>;

  fn build_meta(&self) -> Option<&BuildMeta>;

  fn set_module_build_info_and_meta(&mut self, build_info: BuildInfo, build_meta: BuildMeta);

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

  fn get_exports_type_readonly(&self, module_graph: &ModuleGraph, strict: bool) -> ExportsType {
    let mut mga = ImmutableModuleGraph::new(module_graph);
    get_exports_type_impl(self.identifier(), self.build_meta(), &mut mga, strict)
  }

  fn get_exports_type(&self, module_graph: &mut ModuleGraph, strict: bool) -> ExportsType {
    MutexModuleGraph::new(module_graph).with_lock(|mut mga| {
      get_exports_type_impl(self.identifier(), self.build_meta(), &mut mga, strict)
    })
  }

  fn get_strict_harmony_module(&self) -> bool {
    self
      .build_meta()
      .as_ref()
      .map(|m| m.strict_harmony_module)
      .unwrap_or(false)
  }

  /// The actual code generation of the module, which will be called by the `Compilation`.
  /// The code generation result should not be cached as it is implemented elsewhere to
  /// provide a universal cache mechanism (time to invalidate cache, etc.)
  ///
  /// Code generation will often iterate through every `source_types` given by the module
  /// to provide multiple code generation results for different `source_type`s.
  fn code_generation(
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

  /// Apply module hash to the provided hasher.
  fn update_hash(&self, state: &mut dyn std::hash::Hasher) {
    self.dyn_hash(state);
  }

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
  ) -> Option<String> {
    Some(format!(
      "Module Concatenation is not implemented for {}",
      self.module_type()
    ))
  }

  /// Resolve options matched by module rules.
  /// e.g `javascript/esm` may have special resolving options like `fullySpecified`.
  /// `css` and `css/module` may have special resolving options like `preferRelative`.
  fn get_resolve_options(&self) -> Option<Box<Resolve>> {
    None
  }

  fn get_context(&self) -> Option<Box<Context>> {
    None
  }

  fn chunk_condition(&self, _chunk_key: &ChunkUkey, _compilation: &Compilation) -> Option<bool> {
    None
  }

  fn get_side_effects_connection_state(
    &self,
    _module_graph: &ModuleGraph,
    _module_chain: &mut HashSet<ModuleIdentifier>,
  ) -> ConnectionState {
    ConnectionState::Bool(true)
  }
}

fn get_exports_type_impl(
  identifier: ModuleIdentifier,
  build_meta: Option<&BuildMeta>,
  mga: &mut dyn ModuleGraphAccessor,
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

          if let Some(export_info) =
            mga.get_read_only_export_info(&Atom::from("__esModule"), &identifier)
          {
            if matches!(export_info.provided, Some(ExportInfoProvided::False)) {
              handle_default(default_object)
            } else {
              let Some(target) = export_info.id.get_target(mga, None) else {
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
                let Some(target_exports_type) = mga.get_module_meta_exports_type(&target.module)
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
      // algin to undefined
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

pub trait ModuleExt {
  fn boxed(self) -> Box<dyn Module>;
}

impl<T: Module + 'static> ModuleExt for T {
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

impl PartialEq for dyn Module + '_ {
  fn eq(&self, other: &Self) -> bool {
    self.dyn_eq(other.as_any())
  }
}

impl Eq for dyn Module + '_ {}

impl Hash for dyn Module + '_ {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.dyn_hash(state)
  }
}

impl dyn Module + '_ {
  pub fn downcast_ref<T: Module + Any>(&self) -> Option<&T> {
    self.as_any().downcast_ref::<T>()
  }

  pub fn downcast_mut<T: Module + Any>(&mut self) -> Option<&mut T> {
    self.as_any_mut().downcast_mut::<T>()
  }
}

#[macro_export]
macro_rules! impl_build_info_meta {
  () => {
    fn build_info(&self) -> Option<&$crate::BuildInfo> {
      self.build_info.as_ref()
    }

    fn build_meta(&self) -> Option<&$crate::BuildMeta> {
      self.build_meta.as_ref()
    }

    fn set_module_build_info_and_meta(
      &mut self,
      build_info: $crate::BuildInfo,
      build_meta: $crate::BuildMeta,
    ) {
      self.build_info = Some(build_info);
      self.build_meta = Some(build_meta);
    }
  };
}

macro_rules! impl_module_downcast_helpers {
  ($ty:ty, $ident:ident) => {
    impl dyn Module + '_ {
      ::paste::paste! {
        pub fn [<as_ $ident>](&self) -> Option<& $ty> {
          self.as_any().downcast_ref::<$ty>()
        }

        pub fn [<as_ $ident _mut>](&mut self) -> Option<&mut $ty> {
          self.as_any_mut().downcast_mut::<$ty>()
        }

        pub fn [<try_as_ $ident>](&self) -> Result<& $ty> {
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

pub struct LibIdentOptions<'me> {
  pub context: &'me str,
}

#[cfg(test)]
mod test {
  use std::borrow::Cow;
  use std::hash::Hash;

  use rspack_error::{Diagnosable, Diagnostic, Result};
  use rspack_identifier::{Identifiable, Identifier};
  use rspack_sources::Source;
  use rspack_util::source_map::{ModuleSourceMapConfig, SourceMapKind};

  use super::Module;
  use crate::{
    AsyncDependenciesBlockIdentifier, BuildContext, BuildResult, CodeGenerationResult, Compilation,
    ConcatenationScope, Context, DependenciesBlock, DependencyId, ModuleExt, ModuleType,
    RuntimeSpec, SourceType,
  };

  #[derive(Debug, Eq)]
  struct RawModule(&'static str);

  impl PartialEq for RawModule {
    fn eq(&self, other: &Self) -> bool {
      self.identifier() == other.identifier()
    }
  }

  #[derive(Debug, Eq)]
  struct ExternalModule(&'static str);

  impl PartialEq for ExternalModule {
    fn eq(&self, other: &Self) -> bool {
      self.identifier() == other.identifier()
    }
  }

  impl Hash for RawModule {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
      self.identifier().hash(state);
    }
  }

  impl Hash for ExternalModule {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
      self.identifier().hash(state);
    }
  }

  macro_rules! impl_noop_trait_module_type {
    ($ident: ident) => {
      impl Identifiable for $ident {
        fn identifier(&self) -> Identifier {
          (stringify!($ident).to_owned() + self.0).into()
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

        fn get_dependencies(&self) -> &[DependencyId] {
          unreachable!()
        }
      }

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

        fn size(&self, _source_type: &SourceType) -> f64 {
          unreachable!()
        }

        fn readable_identifier(&self, _context: &Context) -> Cow<str> {
          (stringify!($ident).to_owned() + self.0).into()
        }

        async fn build(
          &mut self,
          _build_context: BuildContext<'_>,
          _compilation: Option<&Compilation>,
        ) -> Result<BuildResult> {
          unreachable!()
        }

        fn get_diagnostics(&self) -> Vec<Diagnostic> {
          vec![]
        }

        fn code_generation(
          &self,
          _compilation: &Compilation,
          _runtime: Option<&RuntimeSpec>,
          _concatenation_scope: Option<ConcatenationScope>,
        ) -> Result<CodeGenerationResult> {
          unreachable!()
        }

        fn build_info(&self) -> Option<&crate::BuildInfo> {
          unreachable!()
        }

        fn build_meta(&self) -> Option<&crate::BuildMeta> {
          unreachable!()
        }

        fn set_module_build_info_and_meta(
          &mut self,
          _build_info: crate::BuildInfo,
          _build_meta: crate::BuildMeta,
        ) {
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
    let a: Box<dyn Module> = ExternalModule("a").boxed();
    let b: Box<dyn Module> = RawModule("a").boxed();

    assert!(a.downcast_ref::<ExternalModule>().is_some());
    assert!(b.downcast_ref::<RawModule>().is_some());

    let a = a.as_ref();
    let b = b.as_ref();
    assert!(a.downcast_ref::<ExternalModule>().is_some());
    assert!(b.downcast_ref::<RawModule>().is_some());
  }

  #[test]
  fn hash_should_work() {
    let e1: Box<dyn Module> = ExternalModule("e").boxed();
    let e2: Box<dyn Module> = ExternalModule("e").boxed();

    let mut state1 = rspack_hash::RspackHash::new(&rspack_hash::HashFunction::Xxhash64);
    let mut state2 = rspack_hash::RspackHash::new(&rspack_hash::HashFunction::Xxhash64);
    e1.hash(&mut state1);
    e2.hash(&mut state2);

    let hash1 = state1.digest(&rspack_hash::HashDigest::Hex);
    let hash2 = state2.digest(&rspack_hash::HashDigest::Hex);
    assert_eq!(hash1, hash2);

    let e3: Box<dyn Module> = ExternalModule("e3").boxed();
    let mut state3 = rspack_hash::RspackHash::new(&rspack_hash::HashFunction::Xxhash64);
    e3.hash(&mut state3);

    let hash3 = state3.digest(&rspack_hash::HashDigest::Hex);
    assert_ne!(hash1, hash3);
  }

  #[test]
  fn eq_should_work() {
    let e1 = ExternalModule("e");
    let e2 = ExternalModule("e");

    assert_eq!(e1, e2);
    assert_eq!(&e1.boxed(), &e2.boxed());

    let r1 = RawModule("r1");
    let r2 = RawModule("r2");
    assert_ne!(r1, r2);
    assert_ne!(&r1.boxed(), &r2.boxed());
  }
}
