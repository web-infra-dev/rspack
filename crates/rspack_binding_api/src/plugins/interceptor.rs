use std::{
  hash::Hash,
  ptr::NonNull,
  sync::{Arc, RwLock},
};

use async_trait::async_trait;
use cow_utils::CowUtils;
use napi::{
  Env, JsValue,
  bindgen_prelude::{Buffer, FromNapiValue, Function, JsValuesTupleIntoVec, Promise, ToNapiValue},
};
use rspack_collections::IdentifierSet;
use rspack_core::{
  AfterResolveResult, AssetEmittedInfo, AsyncModulesArtifact, BeforeResolveResult, BindingCell,
  BoxModule, ChunkUkey, Compilation, CompilationAdditionalTreeRuntimeRequirements,
  CompilationAdditionalTreeRuntimeRequirementsHook, CompilationAfterOptimizeModules,
  CompilationAfterOptimizeModulesHook, CompilationAfterProcessAssets,
  CompilationAfterProcessAssetsHook, CompilationAfterSeal, CompilationAfterSealHook,
  CompilationBuildModule, CompilationBuildModuleHook, CompilationChunkAsset,
  CompilationChunkAssetHook, CompilationChunkHash, CompilationChunkHashHook,
  CompilationExecuteModule, CompilationExecuteModuleHook, CompilationFinishModules,
  CompilationFinishModulesHook, CompilationId, CompilationOptimizeChunkModules,
  CompilationOptimizeChunkModulesHook, CompilationOptimizeModules, CompilationOptimizeModulesHook,
  CompilationOptimizeTree, CompilationOptimizeTreeHook, CompilationParams,
  CompilationProcessAssets, CompilationProcessAssetsHook, CompilationRuntimeModule,
  CompilationRuntimeModuleHook, CompilationRuntimeRequirementInTree,
  CompilationRuntimeRequirementInTreeHook, CompilationSeal, CompilationSealHook,
  CompilationStillValidModule, CompilationStillValidModuleHook, CompilationSucceedModule,
  CompilationSucceedModuleHook, CompilerAfterEmit, CompilerAfterEmitHook, CompilerAssetEmitted,
  CompilerAssetEmittedHook, CompilerCompilation, CompilerCompilationHook, CompilerEmit,
  CompilerEmitHook, CompilerFinishMake, CompilerFinishMakeHook, CompilerId, CompilerMake,
  CompilerMakeHook, CompilerShouldEmit, CompilerShouldEmitHook, CompilerThisCompilation,
  CompilerThisCompilationHook, ContextModuleFactoryAfterResolve,
  ContextModuleFactoryAfterResolveHook, ContextModuleFactoryBeforeResolve,
  ContextModuleFactoryBeforeResolveHook, ExecuteModuleId, Module, ModuleFactoryCreateData,
  ModuleIdentifier, NormalModuleCreateData, NormalModuleFactoryAfterResolve,
  NormalModuleFactoryAfterResolveHook, NormalModuleFactoryBeforeResolve,
  NormalModuleFactoryBeforeResolveHook, NormalModuleFactoryCreateModule,
  NormalModuleFactoryCreateModuleHook, NormalModuleFactoryFactorize,
  NormalModuleFactoryFactorizeHook, NormalModuleFactoryResolve,
  NormalModuleFactoryResolveForScheme, NormalModuleFactoryResolveForSchemeHook,
  NormalModuleFactoryResolveHook, NormalModuleFactoryResolveResult, ResourceData, RuntimeGlobals,
  Scheme, build_module_graph::BuildModuleGraphArtifact, parse_resource,
  rspack_sources::RawStringSource,
};
use rspack_hash::RspackHash;
use rspack_hook::{Hook, Interceptor};
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_paths::Utf8PathBuf;
use rspack_plugin_html::{
  AfterEmitData, AfterTemplateExecutionData, AlterAssetTagGroupsData, AlterAssetTagsData,
  BeforeAssetTagGenerationData, BeforeEmitData, HtmlPluginAfterEmit, HtmlPluginAfterEmitHook,
  HtmlPluginAfterTemplateExecution, HtmlPluginAfterTemplateExecutionHook,
  HtmlPluginAlterAssetTagGroups, HtmlPluginAlterAssetTagGroupsHook, HtmlPluginAlterAssetTags,
  HtmlPluginAlterAssetTagsHook, HtmlPluginBeforeAssetTagGeneration,
  HtmlPluginBeforeAssetTagGenerationHook, HtmlPluginBeforeEmit, HtmlPluginBeforeEmitHook,
};
use rspack_plugin_javascript::{JavascriptModulesChunkHash, JavascriptModulesChunkHashHook};
use rspack_plugin_rsdoctor::{
  RsdoctorAssetPatch, RsdoctorChunkGraph, RsdoctorModuleGraph, RsdoctorModuleIdsPatch,
  RsdoctorModuleSourcesPatch, RsdoctorPluginAssets, RsdoctorPluginAssetsHook,
  RsdoctorPluginChunkGraph, RsdoctorPluginChunkGraphHook, RsdoctorPluginModuleGraph,
  RsdoctorPluginModuleGraphHook, RsdoctorPluginModuleIds, RsdoctorPluginModuleIdsHook,
  RsdoctorPluginModuleSources, RsdoctorPluginModuleSourcesHook,
};
use rspack_plugin_runtime::{
  CreateLinkData, CreateScriptData, LinkPrefetchData, LinkPreloadData, RuntimePluginCreateLink,
  RuntimePluginCreateLinkHook, RuntimePluginCreateScript, RuntimePluginCreateScriptHook,
  RuntimePluginLinkPrefetch, RuntimePluginLinkPrefetchHook, RuntimePluginLinkPreload,
  RuntimePluginLinkPreloadHook,
};

use crate::{
  asset::JsAssetEmittedArgs,
  chunk::{ChunkWrapper, JsChunkAssetArgs},
  compilation::JsCompilationWrapper,
  context_module_factory::{
    JsContextModuleFactoryAfterResolveDataWrapper, JsContextModuleFactoryAfterResolveResult,
    JsContextModuleFactoryBeforeResolveDataWrapper, JsContextModuleFactoryBeforeResolveResult,
  },
  dependency::DependencyWrapper,
  html::{
    JsAfterEmitData, JsAfterTemplateExecutionData, JsAlterAssetTagGroupsData, JsAlterAssetTagsData,
    JsBeforeAssetTagGenerationData, JsBeforeEmitData,
  },
  module::{JsExecuteModuleArg, JsRuntimeModule, JsRuntimeModuleArg, ModuleObject},
  normal_module_factory::{
    JsCreateData, JsNormalModuleFactoryCreateModuleArgs, JsResolveData, JsResolveForSchemeArgs,
    JsResolveForSchemeOutput,
  },
  rsdoctor::{
    JsRsdoctorAssetPatch, JsRsdoctorChunkGraph, JsRsdoctorModuleGraph, JsRsdoctorModuleIdsPatch,
    JsRsdoctorModuleSourcesPatch,
  },
  runtime::{
    JsAdditionalTreeRuntimeRequirementsArg, JsAdditionalTreeRuntimeRequirementsResult,
    JsCreateLinkData, JsCreateScriptData, JsLinkPrefetchData, JsLinkPreloadData, JsRuntimeGlobals,
    JsRuntimeRequirementInTreeArg, JsRuntimeRequirementInTreeResult, JsRuntimeSpec,
  },
  source::JsSourceToJs,
};

#[napi(object)]
pub struct JsTap<'f> {
  #[napi(ts_type = "(...args: any[]) => any")]
  pub function: Function<'f>,
  pub stage: i32,
}

pub struct ThreadsafeJsTap<T: 'static + JsValuesTupleIntoVec, R> {
  pub function: ThreadsafeFunction<T, R>,
  pub stage: i32,
}

impl<T: 'static + JsValuesTupleIntoVec, R> Clone for ThreadsafeJsTap<T, R> {
  fn clone(&self) -> Self {
    Self {
      function: self.function.clone(),
      stage: self.stage,
    }
  }
}

impl<T: 'static + ToNapiValue + JsValuesTupleIntoVec, R: 'static + FromNapiValue>
  ThreadsafeJsTap<T, R>
{
  pub fn from_js_tap(js_tap: JsTap, env: Env) -> napi::Result<Self> {
    let function =
      unsafe { ThreadsafeFunction::from_napi_value(env.raw(), js_tap.function.raw()) }?;
    Ok(Self {
      function,
      stage: js_tap.stage,
    })
  }
}

impl<T: 'static + ToNapiValue + JsValuesTupleIntoVec, R: 'static + FromNapiValue> FromNapiValue
  for ThreadsafeJsTap<T, R>
{
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    unsafe {
      let t = JsTap::from_napi_value(env, napi_val)?;
      ThreadsafeJsTap::from_js_tap(t, Env::from_raw(env))
    }
  }
}

type RegisterFunctionOutput<T, R> = Vec<ThreadsafeJsTap<T, R>>;
type RegisterFunction<T, R> = ThreadsafeFunction<Vec<i32>, RegisterFunctionOutput<T, R>>;

struct RegisterJsTapsInner<T: 'static + JsValuesTupleIntoVec, R> {
  register: RegisterFunction<T, R>,
  cache: RegisterJsTapsCache<T, R>,
  non_skippable_registers: Option<NonSkippableRegisters>,
}

impl<T: 'static + JsValuesTupleIntoVec, R> Clone for RegisterJsTapsInner<T, R> {
  fn clone(&self) -> Self {
    Self {
      register: self.register.clone(),
      cache: self.cache.clone(),
      non_skippable_registers: self.non_skippable_registers.clone(),
    }
  }
}

enum RegisterJsTapsCache<T: 'static + JsValuesTupleIntoVec, R> {
  NoCache,
  Cache(Arc<RwLock<Option<RegisterFunctionOutput<T, R>>>>),
}

impl<T: 'static + JsValuesTupleIntoVec, R> Clone for RegisterJsTapsCache<T, R> {
  fn clone(&self) -> Self {
    match self {
      Self::NoCache => Self::NoCache,
      Self::Cache(c) => Self::Cache(c.clone()),
    }
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R> RegisterJsTapsCache<T, R> {
  pub fn new(cache: bool) -> Self {
    if cache {
      Self::Cache(Default::default())
    } else {
      Self::NoCache
    }
  }
}

impl<T: 'static + ToNapiValue, R: 'static + FromNapiValue> RegisterJsTapsInner<T, R> {
  pub fn new(
    register: RegisterFunction<T, R>,
    non_skippable_registers: Option<NonSkippableRegisters>,
    cache: bool,
  ) -> Self {
    Self {
      register,
      cache: RegisterJsTapsCache::new(cache),
      non_skippable_registers,
    }
  }

  pub async fn call_register(
    &self,
    hook: &impl Hook,
  ) -> rspack_error::Result<RegisterFunctionOutput<T, R>> {
    if let RegisterJsTapsCache::Cache(rw) = &self.cache {
      let cache = {
        #[allow(clippy::unwrap_used)]
        rw.read().unwrap().clone()
      };
      Ok(match cache {
        Some(js_taps) => js_taps,
        None => {
          let js_taps = self.call_register_impl(hook).await?;
          {
            #[allow(clippy::unwrap_used)]
            let mut cache = rw.write().unwrap();
            *cache = Some(js_taps.clone());
          }
          js_taps
        }
      })
    } else {
      let js_taps = self.call_register_impl(hook).await?;
      Ok(js_taps)
    }
  }

  async fn call_register_impl(
    &self,
    hook: &impl Hook,
  ) -> rspack_error::Result<RegisterFunctionOutput<T, R>> {
    let mut used_stages = Vec::from_iter(hook.used_stages());
    used_stages.sort();
    self.register.call_with_sync(used_stages).await
  }

  fn clear_cache(&self) {
    match &self.cache {
      RegisterJsTapsCache::NoCache => {}
      RegisterJsTapsCache::Cache(cache) => {
        #[allow(clippy::unwrap_used)]
        let mut cache = cache.write().unwrap();
        *cache = None;
      }
    }
  }
}

/// define js taps register
/// cache: add cache for register function, used for `before_resolve` or `build_module`
///        which run register function multiple times for every module, cache will ensure
///        it only run once.
/// sync: synchronously/blocking call the register function, most of the register shouldn't
///       be sync since calling a ThreadsafeFunction is async, for now it's only used by
///       execute_module, which strongly required sync call.
macro_rules! define_register {
  ($name:ident, tap = $tap_name:ident<$arg:ty, $ret:ty> @ $tap_hook:ty, cache = $cache:literal, kind = $kind:expr, skip = $skip:tt,) => {
    define_register!(@BASE $name, $tap_name<$arg, $ret>, $cache);
    define_register!(@SKIP $name, $arg, $ret, $cache, $skip);
    define_register!(@INTERCEPTOR $name, $tap_name, $tap_hook, $cache, $kind);
  };
  (@BASE $name:ident, $tap_name:ident<$arg:ty, $ret:ty>, $cache:literal) => {
    #[derive(Clone)]
    pub struct $name {
      inner: RegisterJsTapsInner<$arg, $ret>,
    }

    impl $name {
      pub fn clear_cache(&self) {
        self.inner.clear_cache();
      }
    }

    #[derive(Clone)]
    struct $tap_name {
      function: ThreadsafeFunction<$arg, $ret>,
      stage: i32,
    }

    impl $tap_name {
      pub fn new(tap: ThreadsafeJsTap<$arg, $ret>) -> Self {
        Self {
          function: tap.function,
          stage: tap.stage,
        }
      }
    }
  };
  (@SKIP $name:ident, $arg:ty, $ret:ty, $cache:literal, $skip:literal) => {
    impl $name {
      pub fn new(register: RegisterFunction<$arg, $ret>, non_skippable_registers: NonSkippableRegisters) -> Self {
        Self {
          inner: RegisterJsTapsInner::new(register, $skip.then_some(non_skippable_registers), $cache),
        }
      }
    }
  };
  (@INTERCEPTOR $name:ident, $tap_name:ident, $tap_hook:ty, $cache:literal, $kind:expr) => {
    #[async_trait]
    impl Interceptor<$tap_hook> for $name {
      async fn call(
        &self,
        hook: &$tap_hook,
      ) -> rspack_error::Result<Vec<<$tap_hook as Hook>::Tap>> {
        if let Some(non_skippable_registers) = &self.inner.non_skippable_registers && !non_skippable_registers.is_non_skippable(&$kind) {
          return Ok(Vec::new());
        }
        let js_taps = self.inner.call_register(hook).await?;
        let js_taps = js_taps
          .iter()
          .map(|t| Box::new($tap_name::new(t.clone())) as <$tap_hook as Hook>::Tap)
          .collect();
        Ok(js_taps)
      }
    }
  };
}

#[napi]
#[derive(Debug, PartialEq, Eq)]
pub enum RegisterJsTapKind {
  CompilerThisCompilation,
  CompilerCompilation,
  CompilerMake,
  CompilerFinishMake,
  CompilerShouldEmit,
  CompilerEmit,
  CompilerAfterEmit,
  CompilerAssetEmitted,
  CompilationBuildModule,
  CompilationStillValidModule,
  CompilationSucceedModule,
  CompilationExecuteModule,
  CompilationFinishModules,
  CompilationOptimizeModules,
  CompilationAfterOptimizeModules,
  CompilationOptimizeTree,
  CompilationOptimizeChunkModules,
  CompilationAdditionalTreeRuntimeRequirements,
  CompilationRuntimeRequirementInTree,
  CompilationRuntimeModule,
  CompilationChunkHash,
  CompilationChunkAsset,
  CompilationProcessAssets,
  CompilationAfterProcessAssets,
  CompilationSeal,
  CompilationAfterSeal,
  NormalModuleFactoryBeforeResolve,
  NormalModuleFactoryFactorize,
  NormalModuleFactoryResolve,
  NormalModuleFactoryAfterResolve,
  NormalModuleFactoryCreateModule,
  NormalModuleFactoryResolveForScheme,
  ContextModuleFactoryBeforeResolve,
  ContextModuleFactoryAfterResolve,
  JavascriptModulesChunkHash,
  HtmlPluginBeforeAssetTagGeneration,
  HtmlPluginAlterAssetTags,
  HtmlPluginAlterAssetTagGroups,
  HtmlPluginAfterTemplateExecution,
  HtmlPluginBeforeEmit,
  HtmlPluginAfterEmit,
  RuntimePluginCreateScript,
  RuntimePluginCreateLink,
  RuntimePluginLinkPreload,
  RuntimePluginLinkPrefetch,
  RsdoctorPluginModuleGraph,
  RsdoctorPluginChunkGraph,
  RsdoctorPluginModuleIds,
  RsdoctorPluginModuleSources,
  RsdoctorPluginAssets,
}

#[derive(Default, Clone)]
pub struct NonSkippableRegisters(Arc<RwLock<Vec<RegisterJsTapKind>>>);

impl NonSkippableRegisters {
  pub fn set_non_skippable_registers(&self, kinds: Vec<RegisterJsTapKind>) {
    let mut ks = self.0.write().expect("failed to write lock");
    *ks = kinds;
  }

  pub fn is_non_skippable(&self, kind: &RegisterJsTapKind) -> bool {
    self.0.read().expect("should lock").contains(kind)
  }
}

#[derive(Clone)]
#[napi(object, object_to_js = false)]
pub struct RegisterJsTaps {
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsCompilation) => void); stage: number; }>"
  )]
  pub register_compiler_this_compilation_taps: RegisterFunction<JsCompilationWrapper, ()>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsCompilation) => void); stage: number; }>"
  )]
  pub register_compiler_compilation_taps: RegisterFunction<JsCompilationWrapper, ()>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsCompilation) => Promise<void>); stage: number; }>"
  )]
  pub register_compiler_make_taps: RegisterFunction<JsCompilationWrapper, Promise<()>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsCompilation) => void); stage: number; }>"
  )]
  pub register_compiler_finish_make_taps: RegisterFunction<JsCompilationWrapper, Promise<()>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsCompilation) => boolean | undefined); stage: number; }>"
  )]
  pub register_compiler_should_emit_taps: RegisterFunction<JsCompilationWrapper, Option<bool>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: (() => Promise<void>); stage: number; }>"
  )]
  pub register_compiler_emit_taps: RegisterFunction<(), Promise<()>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: (() => Promise<void>); stage: number; }>"
  )]
  pub register_compiler_after_emit_taps: RegisterFunction<(), Promise<()>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsAssetEmittedArgs) => Promise<void>); stage: number; }>"
  )]
  pub register_compiler_asset_emitted_taps: RegisterFunction<JsAssetEmittedArgs, Promise<()>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: Module) => void); stage: number; }>"
  )]
  pub register_compilation_build_module_taps: RegisterFunction<ModuleObject, ()>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: Module) => void); stage: number; }>"
  )]
  pub register_compilation_still_valid_module_taps: RegisterFunction<ModuleObject, ()>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: Module) => void); stage: number; }>"
  )]
  pub register_compilation_succeed_module_taps: RegisterFunction<ModuleObject, ()>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsExecuteModuleArg) => void); stage: number; }>"
  )]
  pub register_compilation_execute_module_taps: RegisterFunction<JsExecuteModuleArg, ()>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsAdditionalTreeRuntimeRequirementsArg) => JsAdditionalTreeRuntimeRequirementsResult | undefined); stage: number; }>"
  )]
  pub register_compilation_additional_tree_runtime_requirements_taps: RegisterFunction<
    JsAdditionalTreeRuntimeRequirementsArg,
    Option<JsAdditionalTreeRuntimeRequirementsResult>,
  >,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsRuntimeRequirementInTreeArg) => JsRuntimeRequirementInTreeResult | undefined); stage: number; }>"
  )]
  pub register_compilation_runtime_requirement_in_tree_taps:
    RegisterFunction<JsRuntimeRequirementInTreeArg, Option<JsRuntimeRequirementInTreeResult>>,

  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsRuntimeModuleArg) => JsRuntimeModule | undefined); stage: number; }>"
  )]
  pub register_compilation_runtime_module_taps:
    RegisterFunction<JsRuntimeModuleArg, Option<JsRuntimeModule>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsCompilation) => Promise<void>); stage: number; }>"
  )]
  pub register_compilation_finish_modules_taps: RegisterFunction<JsCompilationWrapper, Promise<()>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: (() => boolean | undefined); stage: number; }>"
  )]
  pub register_compilation_optimize_modules_taps: RegisterFunction<(), Option<bool>>,
  #[napi(ts_type = "(stages: Array<number>) => Array<{ function: (() => void); stage: number; }>")]
  pub register_compilation_after_optimize_modules_taps: RegisterFunction<(), ()>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: (() => Promise<void>); stage: number; }>"
  )]
  pub register_compilation_optimize_tree_taps: RegisterFunction<(), Promise<()>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: (() => Promise<boolean | undefined>); stage: number; }>"
  )]
  pub register_compilation_optimize_chunk_modules_taps: RegisterFunction<(), Promise<Option<bool>>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: Chunk) => Buffer); stage: number; }>"
  )]
  pub register_compilation_chunk_hash_taps: RegisterFunction<ChunkWrapper, Buffer>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsChunkAssetArgs) => void); stage: number; }>"
  )]
  pub register_compilation_chunk_asset_taps: RegisterFunction<JsChunkAssetArgs, ()>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsCompilation) => Promise<void>); stage: number; }>"
  )]
  pub register_compilation_process_assets_taps: RegisterFunction<JsCompilationWrapper, Promise<()>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsCompilation) => void); stage: number; }>"
  )]
  pub register_compilation_after_process_assets_taps: RegisterFunction<JsCompilationWrapper, ()>,
  #[napi(ts_type = "(stages: Array<number>) => Array<{ function: (() => void); stage: number; }>")]
  pub register_compilation_seal_taps: RegisterFunction<(), ()>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: (() => Promise<void>); stage: number; }>"
  )]
  pub register_compilation_after_seal_taps: RegisterFunction<(), Promise<()>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsResolveData) => Promise<[boolean | undefined, JsResolveData]>); stage: number; }>"
  )]
  pub register_normal_module_factory_before_resolve_taps:
    RegisterFunction<JsResolveData, Promise<(Option<bool>, JsResolveData)>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsResolveData) => Promise<JsResolveData>); stage: number; }>"
  )]
  pub register_normal_module_factory_factorize_taps:
    RegisterFunction<JsResolveData, Promise<JsResolveData>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsResolveData) => Promise<JsResolveData>); stage: number; }>"
  )]
  pub register_normal_module_factory_resolve_taps:
    RegisterFunction<JsResolveData, Promise<JsResolveData>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsResolveForSchemeArgs) => Promise<[boolean | undefined, JsResolveForSchemeArgs]>); stage: number; }>"
  )]
  pub register_normal_module_factory_resolve_for_scheme_taps:
    RegisterFunction<JsResolveForSchemeArgs, Promise<JsResolveForSchemeOutput>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsResolveData) => Promise<[boolean | undefined, JsResolveData]>); stage: number; }>"
  )]
  pub register_normal_module_factory_after_resolve_taps:
    RegisterFunction<JsResolveData, Promise<(Option<bool>, JsResolveData)>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsNormalModuleFactoryCreateModuleArgs) => Promise<void>); stage: number; }>"
  )]
  pub register_normal_module_factory_create_module_taps:
    RegisterFunction<JsNormalModuleFactoryCreateModuleArgs, Promise<()>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: false | JsContextModuleFactoryBeforeResolveData) => Promise<false | JsContextModuleFactoryBeforeResolveData>); stage: number; }>"
  )]
  pub register_context_module_factory_before_resolve_taps: RegisterFunction<
    JsContextModuleFactoryBeforeResolveResult,
    Promise<JsContextModuleFactoryBeforeResolveResult>,
  >,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: false | JsContextModuleFactoryAfterResolveData) => Promise<false | JsContextModuleFactoryAfterResolveData>); stage: number; }>"
  )]
  pub register_context_module_factory_after_resolve_taps: RegisterFunction<
    JsContextModuleFactoryAfterResolveResult,
    Promise<JsContextModuleFactoryAfterResolveResult>,
  >,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: Chunk) => Buffer); stage: number; }>"
  )]
  pub register_javascript_modules_chunk_hash_taps: RegisterFunction<ChunkWrapper, Buffer>,
  // html plugin
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsBeforeAssetTagGenerationData) => JsBeforeAssetTagGenerationData); stage: number; }>"
  )]
  pub register_html_plugin_before_asset_tag_generation_taps:
    RegisterFunction<JsBeforeAssetTagGenerationData, Promise<JsBeforeAssetTagGenerationData>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsAlterAssetTagsData) => JsAlterAssetTagsData); stage: number; }>"
  )]
  pub register_html_plugin_alter_asset_tags_taps:
    RegisterFunction<JsAlterAssetTagsData, Promise<JsAlterAssetTagsData>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsAlterAssetTagGroupsData) => JsAlterAssetTagGroupsData); stage: number; }>"
  )]
  pub register_html_plugin_alter_asset_tag_groups_taps:
    RegisterFunction<JsAlterAssetTagGroupsData, Promise<JsAlterAssetTagGroupsData>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsAfterTemplateExecutionData) => JsAfterTemplateExecutionData); stage: number; }>"
  )]
  pub register_html_plugin_after_template_execution_taps:
    RegisterFunction<JsAfterTemplateExecutionData, Promise<JsAfterTemplateExecutionData>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsBeforeEmitData) => JsBeforeEmitData); stage: number; }>"
  )]
  pub register_html_plugin_before_emit_taps:
    RegisterFunction<JsBeforeEmitData, Promise<JsBeforeEmitData>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsAfterEmitData) => JsAfterEmitData); stage: number; }>"
  )]
  pub register_html_plugin_after_emit_taps:
    RegisterFunction<JsAfterEmitData, Promise<JsAfterEmitData>>,
  // runtime plugin
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsCreateScriptData) => String); stage: number; }>"
  )]
  pub register_runtime_plugin_create_script_taps:
    RegisterFunction<JsCreateScriptData, Option<String>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsLinkPreloadData) => String); stage: number; }>"
  )]
  pub register_runtime_plugin_create_link_taps: RegisterFunction<JsCreateLinkData, Option<String>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsCreateLinkData) => String); stage: number; }>"
  )]
  pub register_runtime_plugin_link_preload_taps:
    RegisterFunction<JsLinkPreloadData, Option<String>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsLinkPrefetchData) => String); stage: number; }>"
  )]
  pub register_runtime_plugin_link_prefetch_taps:
    RegisterFunction<JsLinkPrefetchData, Option<String>>,
  // rsdoctor plugin
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsRsdoctorModuleGraph) => Promise<boolean | undefined>); stage: number; }>"
  )]
  pub register_rsdoctor_plugin_module_graph_taps:
    RegisterFunction<JsRsdoctorModuleGraph, Promise<Option<bool>>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsRsdoctorChunkGraph) => Promise<boolean | undefined>); stage: number; }>"
  )]
  pub register_rsdoctor_plugin_chunk_graph_taps:
    RegisterFunction<JsRsdoctorChunkGraph, Promise<Option<bool>>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsRsdoctorModuleIdsPatch) => Promise<boolean | undefined>); stage: number; }>"
  )]
  pub register_rsdoctor_plugin_module_ids_taps:
    RegisterFunction<JsRsdoctorModuleIdsPatch, Promise<Option<bool>>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsRsdoctorModuleSourcesPatch) => Promise<boolean | undefined>); stage: number; }>"
  )]
  pub register_rsdoctor_plugin_module_sources_taps:
    RegisterFunction<JsRsdoctorModuleSourcesPatch, Promise<Option<bool>>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsRsdoctorAssetPatch) => Promise<boolean | undefined>); stage: number; }>"
  )]
  pub register_rsdoctor_plugin_assets_taps:
    RegisterFunction<JsRsdoctorAssetPatch, Promise<Option<bool>>>,
}

/* Compiler Hooks */
define_register!(
  RegisterCompilerThisCompilationTaps,
  tap = CompilerThisCompilationTap<JsCompilationWrapper, ()> @ CompilerThisCompilationHook,
  cache = false,
  kind = RegisterJsTapKind::CompilerThisCompilation,
  skip = false,
);
define_register!(
  RegisterCompilerCompilationTaps,
  tap = CompilerCompilationTap<JsCompilationWrapper, ()> @ CompilerCompilationHook,
  cache = false,
  kind = RegisterJsTapKind::CompilerCompilation,
  skip = true,
);
define_register!(
  RegisterCompilerMakeTaps,
  tap = CompilerMakeTap<JsCompilationWrapper, Promise<()>> @ CompilerMakeHook,
  cache = false,
  kind = RegisterJsTapKind::CompilerMake,
  skip = true,
);
define_register!(
  RegisterCompilerFinishMakeTaps,
  tap = CompilerFinishMakeTap<JsCompilationWrapper, Promise<()>> @ CompilerFinishMakeHook,
  cache = false,
  kind = RegisterJsTapKind::CompilerFinishMake,
  skip = true,
);
define_register!(
  RegisterCompilerShouldEmitTaps,
  tap = CompilerShouldEmitTap<JsCompilationWrapper, Option<bool>> @ CompilerShouldEmitHook,
  cache = false,
  kind = RegisterJsTapKind::CompilerShouldEmit,
  skip = true,
);
define_register!(
  RegisterCompilerEmitTaps,
  tap = CompilerEmitTap<(), Promise<()>> @ CompilerEmitHook,
  cache = false,
  kind = RegisterJsTapKind::CompilerEmit,
  skip = true,
);
define_register!(
  RegisterCompilerAfterEmitTaps,
  tap = CompilerAfterEmitTap<(), Promise<()>> @ CompilerAfterEmitHook,
  cache = false,
  kind = RegisterJsTapKind::CompilerAfterEmit,
  skip = true,
);
define_register!(
  RegisterCompilerAssetEmittedTaps,
  tap = CompilerAssetEmittedTap<JsAssetEmittedArgs, Promise<()>> @ CompilerAssetEmittedHook,
  cache = true,
  kind = RegisterJsTapKind::CompilerAssetEmitted,
  skip = true,
);

/* Compilation Hooks */
define_register!(
  RegisterCompilationBuildModuleTaps,
  tap = CompilationBuildModuleTap<ModuleObject, ()> @ CompilationBuildModuleHook,
  cache = true,
  kind = RegisterJsTapKind::CompilationBuildModule,
  skip = true,
);
define_register!(
  RegisterCompilationStillValidModuleTaps,
  tap = CompilationStillValidModuleTap<ModuleObject, ()> @ CompilationStillValidModuleHook,
  cache = true,
  kind = RegisterJsTapKind::CompilationStillValidModule,
  skip = true,
);
define_register!(
  RegisterCompilationSucceedModuleTaps,
  tap = CompilationSucceedModuleTap<ModuleObject, ()> @ CompilationSucceedModuleHook,
  cache = true,
  kind = RegisterJsTapKind::CompilationSucceedModule,
  skip = true,
);
define_register!(
  RegisterCompilationExecuteModuleTaps,
  tap = CompilationExecuteModuleTap<JsExecuteModuleArg, ()> @ CompilationExecuteModuleHook,
  cache = false,
  kind = RegisterJsTapKind::CompilationExecuteModule,
  skip = true,
);
define_register!(
  RegisterCompilationFinishModulesTaps,
  tap = CompilationFinishModulesTap<JsCompilationWrapper, Promise<()>> @ CompilationFinishModulesHook,
  cache = false,
  kind = RegisterJsTapKind::CompilationFinishModules,
  skip = true,
);
define_register!(
  RegisterCompilationOptimizeModulesTaps,
  tap = CompilationOptimizeModulesTap<(), Option<bool>> @ CompilationOptimizeModulesHook,
  cache = true,
  kind = RegisterJsTapKind::CompilationOptimizeModules,
  skip = true,
);
define_register!(
  RegisterCompilationAfterOptimizeModulesTaps,
  tap = CompilationAfterOptimizeModulesTap<(), ()> @ CompilationAfterOptimizeModulesHook,
  cache = false,
  kind = RegisterJsTapKind::CompilationAfterOptimizeModules,
  skip = true,
);
define_register!(
  RegisterCompilationOptimizeTreeTaps,
  tap = CompilationOptimizeTreeTap<(), Promise<()>> @ CompilationOptimizeTreeHook,
  cache = false,
  kind = RegisterJsTapKind::CompilationOptimizeTree,
  skip = true,
);
define_register!(
  RegisterCompilationOptimizeChunkModulesTaps,
  tap = CompilationOptimizeChunkModulesTap<(), Promise<Option<bool>>> @ CompilationOptimizeChunkModulesHook,
  cache = false,
  kind = RegisterJsTapKind::CompilationOptimizeChunkModules,
  skip = true,
);
define_register!(
  RegisterCompilationAdditionalTreeRuntimeRequirementsTaps,
  tap = CompilationAdditionalTreeRuntimeRequirementsTap<JsAdditionalTreeRuntimeRequirementsArg, Option<JsAdditionalTreeRuntimeRequirementsResult>> @ CompilationAdditionalTreeRuntimeRequirementsHook,
  cache = true,
  kind = RegisterJsTapKind::CompilationAdditionalTreeRuntimeRequirements,
  skip = true,
);
define_register!(
  RegisterCompilationRuntimeRequirementInTreeTaps,
  tap = CompilationRuntimeRequirementInTreeTap<JsRuntimeRequirementInTreeArg, Option<JsRuntimeRequirementInTreeResult>> @ CompilationRuntimeRequirementInTreeHook,
  cache = true,
  kind = RegisterJsTapKind::CompilationRuntimeRequirementInTree,
  skip = true,
);
define_register!(
  RegisterCompilationRuntimeModuleTaps,
  tap = CompilationRuntimeModuleTap<JsRuntimeModuleArg, Option<JsRuntimeModule>> @ CompilationRuntimeModuleHook,
  cache = true,
  kind = RegisterJsTapKind::CompilationRuntimeModule,
  skip = true,
);
define_register!(
  RegisterCompilationChunkHashTaps,
  tap = CompilationChunkHashTap<ChunkWrapper, Buffer> @ CompilationChunkHashHook,
  cache = true,
  kind = RegisterJsTapKind::CompilationChunkHash,
  skip = true,
);
define_register!(
  RegisterCompilationChunkAssetTaps,
  tap = CompilationChunkAssetTap<JsChunkAssetArgs, ()> @ CompilationChunkAssetHook,
  cache = true,
  kind = RegisterJsTapKind::CompilationChunkAsset,
  skip = true,
);
define_register!(
  RegisterCompilationProcessAssetsTaps,
  tap = CompilationProcessAssetsTap<JsCompilationWrapper, Promise<()>> @ CompilationProcessAssetsHook,
  cache = false,
  kind = RegisterJsTapKind::CompilationProcessAssets,
  skip = true,
);
define_register!(
  RegisterCompilationAfterProcessAssetsTaps,
  tap = CompilationAfterProcessAssetsTap<JsCompilationWrapper, ()> @ CompilationAfterProcessAssetsHook,
  cache = false,
  kind = RegisterJsTapKind::CompilationAfterProcessAssets,
  skip = true,
);
define_register!(
  RegisterCompilationSealTaps,
  tap = CompilationSealTap<(), ()> @ CompilationSealHook,
  cache = false,
  kind = RegisterJsTapKind::CompilationSeal,
  skip = true,
);
define_register!(
  RegisterCompilationAfterSealTaps,
  tap = CompilationAfterSealTap<(), Promise<()>> @ CompilationAfterSealHook,
  cache = false,
  kind = RegisterJsTapKind::CompilationAfterSeal,
  skip = true,
);

/* NormalModuleFactory Hooks */
define_register!(
  RegisterNormalModuleFactoryBeforeResolveTaps,
  tap = NormalModuleFactoryBeforeResolveTap<JsResolveData, Promise<(Option<bool>, JsResolveData)>> @ NormalModuleFactoryBeforeResolveHook,
  cache = true,
  kind = RegisterJsTapKind::NormalModuleFactoryBeforeResolve,
  skip = true,
);
define_register!(
  RegisterNormalModuleFactoryFactorizeTaps,
  tap = NormalModuleFactoryFactorizeTap<JsResolveData, Promise<JsResolveData>> @ NormalModuleFactoryFactorizeHook,
  cache = true,
  kind = RegisterJsTapKind::NormalModuleFactoryFactorize,
  skip = true,
);
define_register!(
  RegisterNormalModuleFactoryResolveTaps,
  tap = NormalModuleFactoryResolveTap<JsResolveData, Promise<JsResolveData>> @ NormalModuleFactoryResolveHook,
  cache = true,
  kind = RegisterJsTapKind::NormalModuleFactoryResolve,
  skip = true,
);
define_register!(
  RegisterNormalModuleFactoryResolveForSchemeTaps,
  tap = NormalModuleFactoryResolveForSchemeTap<JsResolveForSchemeArgs, Promise<JsResolveForSchemeOutput>> @ NormalModuleFactoryResolveForSchemeHook,
  cache = true,
  kind = RegisterJsTapKind::NormalModuleFactoryResolveForScheme,
  skip = true,
);
define_register!(
  RegisterNormalModuleFactoryAfterResolveTaps,
  tap = NormalModuleFactoryAfterResolveTap<JsResolveData, Promise<(Option<bool>, JsResolveData)>> @ NormalModuleFactoryAfterResolveHook,
  cache = true,
  kind = RegisterJsTapKind::NormalModuleFactoryAfterResolve,
  skip = true,
);
define_register!(
  RegisterNormalModuleFactoryCreateModuleTaps,
  tap = NormalModuleFactoryCreateModuleTap<JsNormalModuleFactoryCreateModuleArgs, Promise<()>> @ NormalModuleFactoryCreateModuleHook,
  cache = true,
  kind = RegisterJsTapKind::NormalModuleFactoryCreateModule,
  skip = true,
);

/* ContextModuleFactory Hooks */
define_register!(
  RegisterContextModuleFactoryBeforeResolveTaps,
  tap = ContextModuleFactoryBeforeResolveTap<JsContextModuleFactoryBeforeResolveResult, Promise<JsContextModuleFactoryBeforeResolveResult>> @ ContextModuleFactoryBeforeResolveHook,
  cache = true,
  kind = RegisterJsTapKind::ContextModuleFactoryBeforeResolve,
  skip = true,
);
define_register!(
  RegisterContextModuleFactoryAfterResolveTaps,
  tap = ContextModuleFactoryAfterResolveTap<JsContextModuleFactoryAfterResolveResult, Promise<JsContextModuleFactoryAfterResolveResult>> @ ContextModuleFactoryAfterResolveHook,
  cache = true,
  kind = RegisterJsTapKind::ContextModuleFactoryAfterResolve,
  skip = true,
);

/* JavascriptModules Hooks */
define_register!(
  RegisterJavascriptModulesChunkHashTaps,
  tap = JavascriptModulesChunkHashTap<ChunkWrapper, Buffer> @ JavascriptModulesChunkHashHook,
  cache = true,
  kind = RegisterJsTapKind::JavascriptModulesChunkHash,
  skip = true,
);

/* HtmlPlugin Hooks */
define_register!(
  RegisterHtmlPluginBeforeAssetTagGenerationTaps,
  tap = HtmlPluginBeforeAssetTagGenerationTap<JsBeforeAssetTagGenerationData, Promise<JsBeforeAssetTagGenerationData>> @ HtmlPluginBeforeAssetTagGenerationHook,
  cache = true,
  kind = RegisterJsTapKind::HtmlPluginBeforeAssetTagGeneration,
  skip = true,
);

define_register!(
  RegisterHtmlPluginAlterAssetTagsTaps,
  tap = HtmlPluginAlterAssetTagsTap<JsAlterAssetTagsData, Promise<JsAlterAssetTagsData>> @ HtmlPluginAlterAssetTagsHook,
  cache = true,
  kind = RegisterJsTapKind::HtmlPluginAlterAssetTags,
  skip = true,
);

define_register!(
  RegisterHtmlPluginAlterAssetTagGroupsTaps,
  tap = HtmlPluginAlterAssetTagGroupsTap<JsAlterAssetTagGroupsData, Promise<JsAlterAssetTagGroupsData>> @ HtmlPluginAlterAssetTagGroupsHook,
  cache = true,
  kind = RegisterJsTapKind::HtmlPluginAlterAssetTagGroups,
  skip = true,
);

define_register!(
  RegisterHtmlPluginAfterTemplateExecutionTaps,
  tap = HtmlPluginAfterTemplateExecutionTap<JsAfterTemplateExecutionData, Promise<JsAfterTemplateExecutionData>> @ HtmlPluginAfterTemplateExecutionHook,
  cache = true,
  kind = RegisterJsTapKind::HtmlPluginAfterTemplateExecution,
  skip = true,
);

define_register!(
  RegisterHtmlPluginBeforeEmitTaps,
  tap = HtmlPluginBeforeEmitTap<JsBeforeEmitData, Promise<JsBeforeEmitData>> @ HtmlPluginBeforeEmitHook,
  cache = true,
  kind = RegisterJsTapKind::HtmlPluginBeforeEmit,
  skip = true,
);

define_register!(
  RegisterHtmlPluginAfterEmitTaps,
  tap = HtmlPluginAfterEmitTap<JsAfterEmitData, Promise<JsAfterEmitData>> @ HtmlPluginAfterEmitHook,
  cache = true,
  kind = RegisterJsTapKind::HtmlPluginAfterEmit,
  skip = true,
);
define_register!(
  RegisterRuntimePluginCreateScriptTaps,
  tap = RuntimePluginCreateScriptTap<JsCreateScriptData, Option<String>> @ RuntimePluginCreateScriptHook,
  cache = true,
  kind = RegisterJsTapKind::RuntimePluginCreateScript,
  skip = true,
);
define_register!(
  RegisterRuntimePluginCreateLinkTaps,
  tap = RuntimePluginCreateLinkTap<JsCreateLinkData, Option<String>> @ RuntimePluginCreateLinkHook,
  cache = true,
  kind = RegisterJsTapKind::RuntimePluginCreateLink,
  skip = true,
);
define_register!(
  RegisterRuntimePluginLinkPreloadTaps,
  tap = RuntimePluginLinkPreloadTap<JsLinkPreloadData, Option<String>> @ RuntimePluginLinkPreloadHook,
  cache = true,
  kind = RegisterJsTapKind::RuntimePluginLinkPreload,
  skip = true,
);
define_register!(
  RegisterRuntimePluginLinkPrefetchTaps,
  tap = RuntimePluginLinkPrefetchTap<JsLinkPrefetchData, Option<String>> @ RuntimePluginLinkPrefetchHook,
  cache = true,
  kind = RegisterJsTapKind::RuntimePluginLinkPrefetch,
  skip = true,
);

/* Rsdoctor Plugin Hooks */
define_register!(
  RegisterRsdoctorPluginModuleGraphTaps,
  tap = RsdoctorPluginModuleGraphTap<JsRsdoctorModuleGraph, Promise<Option<bool>>> @ RsdoctorPluginModuleGraphHook,
  cache = true,
  kind = RegisterJsTapKind::RsdoctorPluginModuleGraph,
  skip = true,
);

define_register!(
  RegisterRsdoctorPluginChunkGraphTaps,
  tap = RsdoctorPluginChunkGraphTap<JsRsdoctorChunkGraph, Promise<Option<bool>>> @ RsdoctorPluginChunkGraphHook,
  cache = true,
  kind = RegisterJsTapKind::RsdoctorPluginChunkGraph,
  skip = true,
);

define_register!(
  RegisterRsdoctorPluginAssetsTaps,
  tap = RsdoctorPluginAssetsTap<JsRsdoctorAssetPatch, Promise<Option<bool>>> @ RsdoctorPluginAssetsHook,
  cache = true,
  kind = RegisterJsTapKind::RsdoctorPluginAssets,
  skip = true,
);

define_register!(
  RegisterRsdoctorPluginModuleIdsTaps,
  tap = RsdoctorPluginModuleIdsTap<JsRsdoctorModuleIdsPatch, Promise<Option<bool>>> @ RsdoctorPluginModuleIdsHook,
  cache = true,
  kind = RegisterJsTapKind::RsdoctorPluginModuleIds,
  skip = true,
);

define_register!(
  RegisterRsdoctorPluginModuleSourcesTaps,
  tap = RsdoctorPluginModuleSourcesTap<JsRsdoctorModuleSourcesPatch, Promise<Option<bool>>> @ RsdoctorPluginModuleSourcesHook,
  cache = true,
  kind = RegisterJsTapKind::RsdoctorPluginModuleSources,
  skip = true,
);

#[async_trait]
impl CompilerThisCompilation for CompilerThisCompilationTap {
  async fn run(
    &self,
    compilation: &mut Compilation,
    _: &mut CompilationParams,
  ) -> rspack_error::Result<()> {
    let compilation = JsCompilationWrapper::new(compilation);
    self.function.call_with_sync(compilation).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl CompilerCompilation for CompilerCompilationTap {
  async fn run(
    &self,
    compilation: &mut Compilation,
    _: &mut CompilationParams,
  ) -> rspack_error::Result<()> {
    let compilation = JsCompilationWrapper::new(compilation);
    self.function.call_with_sync(compilation).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl CompilerMake for CompilerMakeTap {
  async fn run(&self, compilation: &mut Compilation) -> rspack_error::Result<()> {
    let compilation = JsCompilationWrapper::new(compilation);
    self.function.call_with_promise(compilation).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl CompilerFinishMake for CompilerFinishMakeTap {
  async fn run(&self, compilation: &mut Compilation) -> rspack_error::Result<()> {
    let compilation = JsCompilationWrapper::new(compilation);
    self.function.call_with_promise(compilation).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl CompilerShouldEmit for CompilerShouldEmitTap {
  async fn run(&self, compilation: &mut Compilation) -> rspack_error::Result<Option<bool>> {
    let compilation = JsCompilationWrapper::new(compilation);
    self.function.call_with_sync(compilation).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl CompilerEmit for CompilerEmitTap {
  async fn run(&self, _compilation: &mut Compilation) -> rspack_error::Result<()> {
    self.function.call_with_promise(()).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl CompilerAfterEmit for CompilerAfterEmitTap {
  async fn run(&self, _compilation: &mut Compilation) -> rspack_error::Result<()> {
    self.function.call_with_promise(()).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl CompilerAssetEmitted for CompilerAssetEmittedTap {
  async fn run(
    &self,
    _compilation: &Compilation,
    filename: &str,
    info: &AssetEmittedInfo,
  ) -> rspack_error::Result<()> {
    self
      .function
      .call_with_promise(JsAssetEmittedArgs {
        filename: filename.to_string(),
        output_path: info.output_path.as_str().to_owned(),
        target_path: info.target_path.as_str().to_owned(),
      })
      .await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl CompilationBuildModule for CompilationBuildModuleTap {
  async fn run(
    &self,
    compiler_id: CompilerId,
    _compilation_id: CompilationId,
    module: &mut BoxModule,
  ) -> rspack_error::Result<()> {
    #[allow(clippy::unwrap_used)]
    let _ = self
      .function
      .call_with_sync(ModuleObject::with_ptr(
        NonNull::new(module.as_mut() as *const dyn Module as *mut dyn Module).unwrap(),
        compiler_id,
      ))
      .await?;
    Ok(())
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl CompilationStillValidModule for CompilationStillValidModuleTap {
  async fn run(
    &self,
    compiler_id: CompilerId,
    _compilation_id: CompilationId,
    module: &mut BoxModule,
  ) -> rspack_error::Result<()> {
    #[allow(clippy::unwrap_used)]
    let _ = self
      .function
      .call_with_sync(ModuleObject::with_ptr(
        NonNull::new(module.as_mut() as *const dyn Module as *mut dyn Module).unwrap(),
        compiler_id,
      ))
      .await?;
    Ok(())
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl CompilationSucceedModule for CompilationSucceedModuleTap {
  async fn run(
    &self,
    compiler_id: CompilerId,
    _compilation_id: CompilationId,
    module: &mut BoxModule,
  ) -> rspack_error::Result<()> {
    #[allow(clippy::unwrap_used)]
    let _ = self
      .function
      .call_with_sync(ModuleObject::with_ptr(
        NonNull::new(module.as_mut() as *const dyn Module as *mut dyn Module).unwrap(),
        compiler_id,
      ))
      .await?;
    Ok(())
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl CompilationExecuteModule for CompilationExecuteModuleTap {
  async fn run(
    &self,
    entry: &ModuleIdentifier,
    runtime_modules: &IdentifierSet,
    code_generation_results: &BindingCell<rspack_core::CodeGenerationResults>,
    id: &ExecuteModuleId,
  ) -> rspack_error::Result<()> {
    self
      .function
      .call_with_sync(JsExecuteModuleArg {
        entry: entry.to_string(),
        runtime_modules: runtime_modules.iter().map(|id| id.to_string()).collect(),
        codegen_results: code_generation_results.as_ref().into(),
        id: *id,
      })
      .await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl CompilationFinishModules for CompilationFinishModulesTap {
  async fn run(
    &self,
    compilation: &mut Compilation,
    async_modules_artifact: &mut AsyncModulesArtifact,
  ) -> rspack_error::Result<()> {
    let compilation = JsCompilationWrapper::new(compilation);
    self.function.call_with_promise(compilation).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl CompilationOptimizeModules for CompilationOptimizeModulesTap {
  async fn run(
    &self,
    _compilation: &Compilation,
    _diagnostics: &mut Vec<rspack_error::Diagnostic>,
  ) -> rspack_error::Result<Option<bool>> {
    self.function.call_with_sync(()).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl CompilationAfterOptimizeModules for CompilationAfterOptimizeModulesTap {
  async fn run(&self, _compilation: &mut Compilation) -> rspack_error::Result<()> {
    self.function.call_with_sync(()).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl CompilationOptimizeTree for CompilationOptimizeTreeTap {
  async fn run(&self, _compilation: &mut Compilation) -> rspack_error::Result<()> {
    self.function.call_with_promise(()).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl CompilationOptimizeChunkModules for CompilationOptimizeChunkModulesTap {
  async fn run(&self, _compilation: &mut Compilation) -> rspack_error::Result<Option<bool>> {
    self.function.call_with_promise(()).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl CompilationAdditionalTreeRuntimeRequirements
  for CompilationAdditionalTreeRuntimeRequirementsTap
{
  async fn run(
    &self,
    compilation: &mut Compilation,
    chunk_ukey: &ChunkUkey,
    runtime_requirements: &mut RuntimeGlobals,
  ) -> rspack_error::Result<()> {
    let arg = JsAdditionalTreeRuntimeRequirementsArg {
      chunk: ChunkWrapper::new(*chunk_ukey, compilation),
      runtime_requirements: JsRuntimeGlobals::from(*runtime_requirements),
    };
    let result = self.function.call_with_sync(arg).await?;
    if let Some(result) = result {
      runtime_requirements.insert(
        result
          .as_runtime_globals()
          .difference(*runtime_requirements),
      );
    }
    Ok(())
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl CompilationRuntimeRequirementInTree for CompilationRuntimeRequirementInTreeTap {
  async fn run(
    &self,
    compilation: &mut Compilation,
    chunk_ukey: &ChunkUkey,
    all_runtime_requirements: &RuntimeGlobals,
    runtime_requirements: &RuntimeGlobals,
    runtime_requirements_mut: &mut RuntimeGlobals,
  ) -> rspack_error::Result<Option<()>> {
    let arg = JsRuntimeRequirementInTreeArg {
      chunk: ChunkWrapper::new(*chunk_ukey, compilation),
      all_runtime_requirements: JsRuntimeGlobals::from(*all_runtime_requirements),
      runtime_requirements: JsRuntimeGlobals::from(*runtime_requirements),
    };
    let result = self.function.call_with_sync(arg).await?;
    if let Some(result) = result {
      runtime_requirements_mut.extend(
        result
          .as_runtime_globals()
          .difference(*all_runtime_requirements),
      );
    }
    Ok(None)
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl CompilationRuntimeModule for CompilationRuntimeModuleTap {
  async fn run(
    &self,
    compilation: &mut Compilation,
    m: &ModuleIdentifier,
    chunk_ukey: &ChunkUkey,
  ) -> rspack_error::Result<()> {
    let Some(module) = compilation.runtime_modules.get(m) else {
      return Ok(());
    };
    let source_string = module.generate(compilation).await?;
    let arg = JsRuntimeModuleArg {
      module: JsRuntimeModule {
        source: Some(JsSourceToJs::from(source_string)),
        module_identifier: module.identifier().to_string(),
        constructor_name: module.get_constructor_name(),
        name: module
          .name()
          .as_str()
          .cow_replace(compilation.runtime_template.runtime_module_prefix(), "")
          .into_owned(),
      },
      chunk: ChunkWrapper::new(*chunk_ukey, compilation),
    };
    if let Some(module) = self.function.call_with_sync(arg).await?
      && let Some(source) = module.source
    {
      let module = compilation
        .runtime_modules
        .get_mut(m)
        .expect("should have module");
      match source.source {
        napi::Either::A(string) => {
          module.set_custom_source(string);
        }
        napi::Either::B(buffer) => {
          module.set_custom_source(String::from_utf8_lossy(&buffer).into_owned());
        }
      }
    }
    Ok(())
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl CompilationChunkHash for CompilationChunkHashTap {
  async fn run(
    &self,
    compilation: &Compilation,
    chunk_ukey: &ChunkUkey,
    hasher: &mut RspackHash,
  ) -> rspack_error::Result<()> {
    let result = self
      .function
      .call_with_sync(ChunkWrapper::new(*chunk_ukey, compilation))
      .await?;
    result.hash(hasher);
    Ok(())
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl CompilationChunkAsset for CompilationChunkAssetTap {
  async fn run(
    &self,
    compilation: &Compilation,
    chunk_ukey: &ChunkUkey,
    file: &str,
  ) -> rspack_error::Result<()> {
    self
      .function
      .call_with_sync(JsChunkAssetArgs {
        chunk: ChunkWrapper::new(*chunk_ukey, compilation),
        filename: file.to_string(),
      })
      .await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl CompilationProcessAssets for CompilationProcessAssetsTap {
  async fn run(&self, compilation: &mut Compilation) -> rspack_error::Result<()> {
    let compilation = JsCompilationWrapper::new(compilation);
    self.function.call_with_promise(compilation).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl CompilationAfterProcessAssets for CompilationAfterProcessAssetsTap {
  async fn run(&self, compilation: &mut Compilation) -> rspack_error::Result<()> {
    let compilation = JsCompilationWrapper::new(compilation);
    self.function.call_with_sync(compilation).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl CompilationSeal for CompilationSealTap {
  async fn run(&self, _compilation: &mut Compilation) -> rspack_error::Result<()> {
    self.function.call_with_sync(()).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl CompilationAfterSeal for CompilationAfterSealTap {
  async fn run(&self, _compilation: &mut Compilation) -> rspack_error::Result<()> {
    self.function.call_with_promise(()).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl NormalModuleFactoryBeforeResolve for NormalModuleFactoryBeforeResolveTap {
  async fn run(&self, data: &mut ModuleFactoryCreateData) -> rspack_error::Result<Option<bool>> {
    match self
      .function
      .call_with_promise(JsResolveData::from_nmf_data(data, None))
      .await
    {
      Ok((ret, resolve_data)) => {
        resolve_data.update_nmf_data(data, None);
        Ok(ret)
      }
      Err(err) => Err(err),
    }
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl NormalModuleFactoryFactorize for NormalModuleFactoryFactorizeTap {
  async fn run(
    &self,
    data: &mut ModuleFactoryCreateData,
  ) -> rspack_error::Result<Option<BoxModule>> {
    match self
      .function
      .call_with_promise(JsResolveData::from_nmf_data(data, None))
      .await
    {
      Ok(resolve_data) => {
        resolve_data.update_nmf_data(data, None);
        Ok(None)
      }
      Err(err) => Err(err),
    }
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl NormalModuleFactoryResolve for NormalModuleFactoryResolveTap {
  async fn run(
    &self,
    data: &mut ModuleFactoryCreateData,
  ) -> rspack_error::Result<Option<NormalModuleFactoryResolveResult>> {
    match self
      .function
      .call_with_promise(JsResolveData::from_nmf_data(data, None))
      .await
    {
      Ok(resolve_data) => {
        resolve_data.update_nmf_data(data, None);
        Ok(None)
      }
      Err(err) => Err(err),
    }
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl NormalModuleFactoryResolveForScheme for NormalModuleFactoryResolveForSchemeTap {
  async fn run(
    &self,
    _data: &mut ModuleFactoryCreateData,
    resource_data: &mut ResourceData,
    scheme: &Scheme,
  ) -> rspack_error::Result<Option<bool>> {
    let (bail, new_resource_data) = self
      .function
      .call_with_promise(JsResolveForSchemeArgs {
        resource_data: (&*resource_data).into(),
        scheme: scheme.to_string(),
      })
      .await?;
    resource_data.set_resource(new_resource_data.resource);
    resource_data.set_path_optional(new_resource_data.path.map(Utf8PathBuf::from));
    resource_data.set_query_optional(new_resource_data.query);
    resource_data.set_fragment_optional(new_resource_data.fragment);
    Ok(bail)
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl NormalModuleFactoryAfterResolve for NormalModuleFactoryAfterResolveTap {
  async fn run(
    &self,
    data: &mut ModuleFactoryCreateData,
    create_data: &mut NormalModuleCreateData,
  ) -> rspack_error::Result<Option<bool>> {
    match self
      .function
      .call_with_promise(JsResolveData::from_nmf_data(data, Some(create_data)))
      .await
    {
      Ok((ret, new_data)) => {
        new_data.update_nmf_data(data, Some(create_data));
        Ok(ret)
      }
      Err(err) => Err(err),
    }
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl NormalModuleFactoryCreateModule for NormalModuleFactoryCreateModuleTap {
  async fn run(
    &self,
    data: &mut ModuleFactoryCreateData,
    create_data: &mut NormalModuleCreateData,
  ) -> rspack_error::Result<Option<BoxModule>> {
    self
      .function
      .call_with_promise(JsNormalModuleFactoryCreateModuleArgs {
        dependency_type: data.dependencies[0].dependency_type().to_string(),
        raw_request: create_data.raw_request.clone(),
        resource_resolve_data: (&create_data.resource_resolve_data).into(),
        context: data.context.to_string(),
        match_resource: create_data.match_resource.clone(),
      })
      .await?;
    Ok(None)
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl ContextModuleFactoryBeforeResolve for ContextModuleFactoryBeforeResolveTap {
  async fn run(&self, result: BeforeResolveResult) -> rspack_error::Result<BeforeResolveResult> {
    let js_result = match result {
      BeforeResolveResult::Ignored => JsContextModuleFactoryBeforeResolveResult::A(false),
      BeforeResolveResult::Data(data) => JsContextModuleFactoryBeforeResolveResult::B(
        JsContextModuleFactoryBeforeResolveDataWrapper::new(data),
      ),
    };
    match self.function.call_with_promise(js_result).await {
      Ok(js_result) => match js_result {
        napi::bindgen_prelude::Either::A(_) => Ok(BeforeResolveResult::Ignored),
        napi::bindgen_prelude::Either::B(js_data) => Ok(BeforeResolveResult::Data(js_data.take())),
      },
      Err(err) => Err(err),
    }
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl ContextModuleFactoryAfterResolve for ContextModuleFactoryAfterResolveTap {
  async fn run(&self, result: AfterResolveResult) -> rspack_error::Result<AfterResolveResult> {
    let js_result = match result {
      AfterResolveResult::Ignored => JsContextModuleFactoryAfterResolveResult::A(false),
      AfterResolveResult::Data(data) => JsContextModuleFactoryAfterResolveResult::B(
        JsContextModuleFactoryAfterResolveDataWrapper::new(data),
      ),
    };
    match self.function.call_with_promise(js_result).await? {
      napi::Either::A(_) => Ok(AfterResolveResult::Ignored),
      napi::Either::B(js_data) => Ok(AfterResolveResult::Data(js_data.take())),
    }
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl JavascriptModulesChunkHash for JavascriptModulesChunkHashTap {
  async fn run(
    &self,
    compilation: &Compilation,
    chunk_ukey: &ChunkUkey,
    hasher: &mut RspackHash,
  ) -> rspack_error::Result<()> {
    let result = self
      .function
      .call_with_sync(ChunkWrapper::new(*chunk_ukey, compilation))
      .await?;
    result.hash(hasher);
    Ok(())
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl HtmlPluginBeforeAssetTagGeneration for HtmlPluginBeforeAssetTagGenerationTap {
  async fn run(
    &self,
    data: BeforeAssetTagGenerationData,
  ) -> rspack_error::Result<BeforeAssetTagGenerationData> {
    let result = self
      .function
      .call_with_promise(JsBeforeAssetTagGenerationData::from(data))
      .await?;
    Ok(result.into())
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl HtmlPluginAlterAssetTags for HtmlPluginAlterAssetTagsTap {
  async fn run(&self, data: AlterAssetTagsData) -> rspack_error::Result<AlterAssetTagsData> {
    let result = self
      .function
      .call_with_promise(JsAlterAssetTagsData::from(data))
      .await?;
    Ok(result.into())
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl HtmlPluginAlterAssetTagGroups for HtmlPluginAlterAssetTagGroupsTap {
  async fn run(
    &self,
    data: AlterAssetTagGroupsData,
  ) -> rspack_error::Result<AlterAssetTagGroupsData> {
    let result = self
      .function
      .call_with_promise(JsAlterAssetTagGroupsData::from(data))
      .await?;
    Ok(result.into())
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl HtmlPluginAfterTemplateExecution for HtmlPluginAfterTemplateExecutionTap {
  async fn run(
    &self,
    data: AfterTemplateExecutionData,
  ) -> rspack_error::Result<AfterTemplateExecutionData> {
    let result = self
      .function
      .call_with_promise(JsAfterTemplateExecutionData::from(data))
      .await?;
    Ok(result.into())
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl HtmlPluginBeforeEmit for HtmlPluginBeforeEmitTap {
  async fn run(&self, data: BeforeEmitData) -> rspack_error::Result<BeforeEmitData> {
    let result = self
      .function
      .call_with_promise(JsBeforeEmitData::from(data))
      .await?;
    Ok(result.into())
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl HtmlPluginAfterEmit for HtmlPluginAfterEmitTap {
  async fn run(&self, data: AfterEmitData) -> rspack_error::Result<AfterEmitData> {
    let result = self
      .function
      .call_with_promise(JsAfterEmitData::from(data))
      .await?;
    Ok(result.into())
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl RuntimePluginCreateScript for RuntimePluginCreateScriptTap {
  async fn run(&self, mut data: CreateScriptData) -> rspack_error::Result<CreateScriptData> {
    if let Some(code) = self
      .function
      .call_with_sync(JsCreateScriptData::from(data.clone()))
      .await?
    {
      data.code = code;
    }
    Ok(data)
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl RuntimePluginCreateLink for RuntimePluginCreateLinkTap {
  async fn run(&self, mut data: CreateLinkData) -> rspack_error::Result<CreateLinkData> {
    if let Some(code) = self
      .function
      .call_with_sync(JsCreateLinkData::from(data.clone()))
      .await?
    {
      data.code = code;
    }
    Ok(data)
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl RuntimePluginLinkPreload for RuntimePluginLinkPreloadTap {
  async fn run(&self, mut data: LinkPreloadData) -> rspack_error::Result<LinkPreloadData> {
    if let Some(code) = self
      .function
      .call_with_sync(JsLinkPreloadData::from(data.clone()))
      .await?
    {
      data.code = code;
    }
    Ok(data)
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl RuntimePluginLinkPrefetch for RuntimePluginLinkPrefetchTap {
  async fn run(&self, mut data: LinkPrefetchData) -> rspack_error::Result<LinkPrefetchData> {
    if let Some(code) = self
      .function
      .call_with_sync(JsLinkPrefetchData::from(data.clone()))
      .await?
    {
      data.code = code;
    }
    Ok(data)
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl RsdoctorPluginModuleGraph for RsdoctorPluginModuleGraphTap {
  async fn run(&self, data: &mut RsdoctorModuleGraph) -> rspack_error::Result<Option<bool>> {
    let data = std::mem::take(data);
    let bail = self
      .function
      .call_with_promise(JsRsdoctorModuleGraph::from(data))
      .await?;
    Ok(bail)
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl RsdoctorPluginChunkGraph for RsdoctorPluginChunkGraphTap {
  async fn run(&self, data: &mut RsdoctorChunkGraph) -> rspack_error::Result<Option<bool>> {
    let data = std::mem::take(data);
    let bail = self
      .function
      .call_with_promise(JsRsdoctorChunkGraph::from(data))
      .await?;
    Ok(bail)
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl RsdoctorPluginModuleIds for RsdoctorPluginModuleIdsTap {
  async fn run(&self, data: &mut RsdoctorModuleIdsPatch) -> rspack_error::Result<Option<bool>> {
    let data = std::mem::take(data);
    let bail = self
      .function
      .call_with_promise(JsRsdoctorModuleIdsPatch::from(data))
      .await?;
    Ok(bail)
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl RsdoctorPluginModuleSources for RsdoctorPluginModuleSourcesTap {
  async fn run(&self, data: &mut RsdoctorModuleSourcesPatch) -> rspack_error::Result<Option<bool>> {
    let data = std::mem::take(data);
    let bail = self
      .function
      .call_with_promise(JsRsdoctorModuleSourcesPatch::from(data))
      .await?;
    Ok(bail)
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl RsdoctorPluginAssets for RsdoctorPluginAssetsTap {
  async fn run(&self, data: &mut RsdoctorAssetPatch) -> rspack_error::Result<Option<bool>> {
    let data = std::mem::take(data);
    let bail = self
      .function
      .call_with_promise(JsRsdoctorAssetPatch::from(data))
      .await?;
    Ok(bail)
  }
  fn stage(&self) -> i32 {
    self.stage
  }
}
