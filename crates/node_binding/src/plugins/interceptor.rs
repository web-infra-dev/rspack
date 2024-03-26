use std::{borrow::Cow, sync::Arc};

use async_trait::async_trait;
use napi::{
  bindgen_prelude::{FromNapiValue, Promise, ToNapiValue},
  Env, JsFunction, NapiRaw,
};
use rspack_binding_values::{
  CompatSource, JsAfterResolveData, JsAfterResolveOutput, JsAssetEmittedArgs, JsBeforeResolveArgs,
  JsBeforeResolveOutput, JsChunk, JsChunkAssetArgs, JsCompilation, JsCreateData,
  JsExecuteModuleArg, JsFactoryMeta, JsModule, JsRuntimeModule, JsRuntimeModuleArg,
  ToJsCompatSource, ToJsModule,
};
use rspack_core::{
  rspack_sources::SourceExt, AssetEmittedInfo, BeforeResolveArgs, BoxModule, Chunk, ChunkUkey,
  CodeGenerationResults, Compilation, CompilationAfterOptimizeModulesHook,
  CompilationAfterProcessAssetsHook, CompilationBuildModuleHook, CompilationChunkAssetHook,
  CompilationExecuteModuleHook, CompilationFinishModulesHook, CompilationOptimizeChunkModulesHook,
  CompilationOptimizeModulesHook, CompilationOptimizeTreeHook, CompilationParams,
  CompilationProcessAssetsHook, CompilationRuntimeModuleHook, CompilationStillValidModuleHook,
  CompilationSucceedModuleHook, CompilerAfterEmitHook, CompilerAssetEmittedHook,
  CompilerCompilationHook, CompilerEmitHook, CompilerFinishMakeHook, CompilerMakeHook,
  CompilerShouldEmitHook, CompilerThisCompilationHook, CreateData, ExecuteModuleId, FactoryMeta,
  MakeParam, ModuleFactoryCreateData, ModuleIdentifier, NormalModuleFactoryAfterResolveHook,
  NormalModuleFactoryBeforeResolveHook, ResourceData,
};
use rspack_hook::{
  AsyncParallel3, AsyncSeries, AsyncSeries2, AsyncSeries3, AsyncSeriesBail, AsyncSeriesBail4, Hook,
  Interceptor, SyncSeries4,
};
use rspack_identifier::IdentifierSet;
use rspack_napi::threadsafe_function::ThreadsafeFunction;

#[napi(object)]
pub struct JsTap {
  pub function: JsFunction,
  pub stage: i32,
}

pub struct ThreadsafeJsTap<T: 'static, R> {
  pub function: ThreadsafeFunction<T, R>,
  pub stage: i32,
}

impl<T: 'static, R> Clone for ThreadsafeJsTap<T, R> {
  fn clone(&self) -> Self {
    Self {
      function: self.function.clone(),
      stage: self.stage,
    }
  }
}

impl<T: 'static + ToNapiValue, R> ThreadsafeJsTap<T, R> {
  pub fn from_js_tap(js_tap: JsTap, env: Env) -> napi::Result<Self> {
    let function =
      unsafe { ThreadsafeFunction::from_napi_value(env.raw(), js_tap.function.raw()) }?;
    Ok(Self {
      function,
      stage: js_tap.stage,
    })
  }
}

impl<T: 'static + ToNapiValue, R> FromNapiValue for ThreadsafeJsTap<T, R> {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    let t = JsTap::from_napi_value(env, napi_val)?;
    ThreadsafeJsTap::from_js_tap(t, Env::from_raw(env))
  }
}

type RegisterFunctionOutput<T, R> = Vec<ThreadsafeJsTap<T, R>>;
type RegisterFunction<T, R> = ThreadsafeFunction<Vec<i32>, RegisterFunctionOutput<T, R>>;

struct RegisterJsTapsInner<T: 'static, R> {
  register: RegisterFunction<T, R>,
  cache: RegisterJsTapsCache<T, R>,
}

impl<T: 'static, R> Clone for RegisterJsTapsInner<T, R> {
  fn clone(&self) -> Self {
    Self {
      register: self.register.clone(),
      cache: self.cache.clone(),
    }
  }
}

enum RegisterJsTapsCache<T: 'static, R> {
  NoCache,
  Cache(Arc<tokio::sync::OnceCell<RegisterFunctionOutput<T, R>>>),
  SyncCache(Arc<once_cell::sync::OnceCell<RegisterFunctionOutput<T, R>>>),
}

impl<T: 'static, R> Clone for RegisterJsTapsCache<T, R> {
  fn clone(&self) -> Self {
    match self {
      Self::NoCache => Self::NoCache,
      Self::Cache(c) => Self::Cache(c.clone()),
      Self::SyncCache(c) => Self::SyncCache(c.clone()),
    }
  }
}

impl<T: 'static, R> RegisterJsTapsCache<T, R> {
  pub fn new(cache: bool, sync: bool) -> Self {
    if cache {
      if sync {
        Self::SyncCache(Default::default())
      } else {
        Self::Cache(Default::default())
      }
    } else {
      Self::NoCache
    }
  }
}

impl<T: 'static + ToNapiValue, R: 'static> RegisterJsTapsInner<T, R> {
  pub fn new(register: RegisterFunction<T, R>, cache: bool, sync: bool) -> Self {
    Self {
      register,
      cache: RegisterJsTapsCache::new(cache, sync),
    }
  }

  pub async fn call_register(
    &self,
    hook: &impl Hook,
  ) -> rspack_error::Result<Cow<RegisterFunctionOutput<T, R>>> {
    if let RegisterJsTapsCache::Cache(cache) = &self.cache {
      let js_taps = cache
        .get_or_try_init(|| self.call_register_impl(hook))
        .await?;
      Ok(Cow::Borrowed(js_taps))
    } else {
      let js_taps = self.call_register_impl(hook).await?;
      Ok(Cow::Owned(js_taps))
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

  pub fn call_register_blocking(
    &self,
    hook: &impl Hook,
  ) -> rspack_error::Result<Cow<RegisterFunctionOutput<T, R>>> {
    if let RegisterJsTapsCache::SyncCache(cache) = &self.cache {
      let js_taps = cache.get_or_try_init(|| self.call_register_blocking_impl(hook))?;
      Ok(Cow::Borrowed(js_taps))
    } else {
      let js_taps = self.call_register_blocking_impl(hook)?;
      Ok(Cow::Owned(js_taps))
    }
  }

  fn call_register_blocking_impl(
    &self,
    hook: &impl Hook,
  ) -> rspack_error::Result<RegisterFunctionOutput<T, R>> {
    let mut used_stages = Vec::from_iter(hook.used_stages());
    used_stages.sort();
    self.register.blocking_call_with_sync(used_stages)
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
  ($name:ident, tap = $tap_name:ident<$arg:ty, $ret:ty> @ $tap_hook:ty, cache = $cache:literal, sync = $sync:tt,) => {
    define_register!(@BASE $name, $tap_name<$arg, $ret> @ $tap_hook, $cache, $sync);
    define_register!(@INTERCEPTOR $name, $tap_name<$arg, $ret> @ $tap_hook, $cache, $sync);
  };
  (@BASE $name:ident, $tap_name:ident<$arg:ty, $ret:ty> @ $tap_hook:ty, $cache:literal, $sync:literal) => {
    #[derive(Clone)]
    pub struct $name {
      inner: RegisterJsTapsInner<$arg, $ret>,
    }

    impl $name {
      pub fn new(register: RegisterFunction<$arg, $ret>) -> Self {
        Self {
          inner: RegisterJsTapsInner::new(register, $cache, $sync),
        }
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
  (@INTERCEPTOR $name:ident, $tap_name:ident<$arg:ty, $ret:ty> @ $tap_hook:ty, $cache:literal, false) => {
    #[async_trait]
    impl Interceptor<$tap_hook> for $name {
      async fn call(
        &self,
        hook: &$tap_hook,
      ) -> rspack_error::Result<Vec<<$tap_hook as Hook>::Tap>> {
        let js_taps = self.inner.call_register(hook).await?;
        let js_taps = js_taps
          .iter()
          .map(|t| Box::new($tap_name::new(t.clone())) as <$tap_hook as Hook>::Tap)
          .collect();
        Ok(js_taps)
      }
    }
  };
  (@INTERCEPTOR $name:ident, $tap_name:ident<$arg:ty, $ret:ty> @ $tap_hook:ty, $cache:literal, true) => {
    impl Interceptor<$tap_hook> for $name {
      fn call_blocking(
        &self,
        hook: &$tap_hook,
      ) -> rspack_error::Result<Vec<<$tap_hook as Hook>::Tap>> {
        let js_taps = self.inner.call_register_blocking(hook)?;
        let js_taps = js_taps
          .iter()
          .map(|t| Box::new($tap_name::new(t.clone())) as <$tap_hook as Hook>::Tap)
          .collect();
        Ok(js_taps)
      }
    }
  };
}

#[derive(Clone)]
#[napi(object, object_to_js = false)]
pub struct RegisterJsTaps {
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsCompilation) => void); stage: number; }>"
  )]
  pub register_compiler_this_compilation_taps: RegisterFunction<JsCompilation, ()>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsCompilation) => void); stage: number; }>"
  )]
  pub register_compiler_compilation_taps: RegisterFunction<JsCompilation, ()>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsCompilation) => Promise<void>); stage: number; }>"
  )]
  pub register_compiler_make_taps: RegisterFunction<JsCompilation, Promise<()>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsCompilation) => void); stage: number; }>"
  )]
  pub register_compiler_finish_make_taps: RegisterFunction<JsCompilation, Promise<()>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsCompilation) => boolean | undefined); stage: number; }>"
  )]
  pub register_compiler_should_emit_taps: RegisterFunction<JsCompilation, Option<bool>>,
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
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsModule) => void); stage: number; }>"
  )]
  pub register_compilation_build_module_taps: RegisterFunction<JsModule, ()>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsModule) => void); stage: number; }>"
  )]
  pub register_compilation_still_valid_module_taps: RegisterFunction<JsModule, ()>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsModule) => void); stage: number; }>"
  )]
  pub register_compilation_succeed_module_taps: RegisterFunction<JsModule, ()>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsExecuteModuleArg) => void); stage: number; }>"
  )]
  pub register_compilation_execute_module_taps: RegisterFunction<JsExecuteModuleArg, ()>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsRuntimeModuleArg) => JsRuntimeModule | undefined); stage: number; }>"
  )]
  pub register_compilation_runtime_module_taps:
    RegisterFunction<JsRuntimeModuleArg, Option<JsRuntimeModule>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsCompilation) => Promise<void>); stage: number; }>"
  )]
  pub register_compilation_finish_modules_taps: RegisterFunction<JsCompilation, Promise<()>>,
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
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsChunkAssetArgs) => void); stage: number; }>"
  )]
  pub register_compilation_chunk_asset_taps: RegisterFunction<JsChunkAssetArgs, ()>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsCompilation) => Promise<void>); stage: number; }>"
  )]
  pub register_compilation_process_assets_taps: RegisterFunction<JsCompilation, Promise<()>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsCompilation) => void); stage: number; }>"
  )]
  pub register_compilation_after_process_assets_taps: RegisterFunction<JsCompilation, ()>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsBeforeResolveArgs) => Promise<[boolean | undefined, JsBeforeResolveArgs]>); stage: number; }>"
  )]
  pub register_normal_module_factory_before_resolve_taps:
    RegisterFunction<JsBeforeResolveArgs, Promise<JsBeforeResolveOutput>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((arg: JsAfterResolveData) => Promise<[boolean | undefined, JsCreateData | undefined]>); stage: number; }>"
  )]
  pub register_normal_module_factory_after_resolve_taps:
    RegisterFunction<JsAfterResolveData, Promise<JsAfterResolveOutput>>,
}

/* Compiler Hooks */
define_register!(
  RegisterCompilerThisCompilationTaps,
  tap = CompilerThisCompilationTap<JsCompilation, ()> @ CompilerThisCompilationHook,
  cache = false,
  sync = false,
);
define_register!(
  RegisterCompilerCompilationTaps,
  tap = CompilerCompilationTap<JsCompilation, ()> @ CompilerCompilationHook,
  cache = false,
  sync = false,
);
define_register!(
  RegisterCompilerMakeTaps,
  tap = CompilerMakeTap<JsCompilation, Promise<()>> @ CompilerMakeHook,
  cache = false,
  sync = false,
);
define_register!(
  RegisterCompilerFinishMakeTaps,
  tap = CompilerFinishMakeTap<JsCompilation, Promise<()>> @ CompilerFinishMakeHook,
  cache = false,
  sync = false,
);
define_register!(
  RegisterCompilerShouldEmitTaps,
  tap = CompilerShouldEmitTap<JsCompilation, Option<bool>> @ CompilerShouldEmitHook,
  cache = false,
  sync = false,
);
define_register!(
  RegisterCompilerEmitTaps,
  tap = CompilerEmitTap<(), Promise<()>> @ CompilerEmitHook,
  cache = false,
  sync = false,
);
define_register!(
  RegisterCompilerAfterEmitTaps,
  tap = CompilerAfterEmitTap<(), Promise<()>> @ CompilerAfterEmitHook,
  cache = false,
  sync = false,
);
define_register!(
  RegisterCompilerAssetEmittedTaps,
  tap = CompilerAssetEmittedTap<JsAssetEmittedArgs, Promise<()>> @ CompilerAssetEmittedHook,
  cache = true,
  sync = false,
);

/* Compilation Hooks */
define_register!(
  RegisterCompilationBuildModuleTaps,
  tap = CompilationBuildModuleTap<JsModule, ()> @ CompilationBuildModuleHook,
  cache = true,
  sync = false,
);
define_register!(
  RegisterCompilationStillValidModuleTaps,
  tap = CompilationStillValidModuleTap<JsModule, ()> @ CompilationStillValidModuleHook,
  cache = true,
  sync = false,
);
define_register!(
  RegisterCompilationSucceedModuleTaps,
  tap = CompilationSucceedModuleTap<JsModule, ()> @ CompilationSucceedModuleHook,
  cache = true,
  sync = false,
);
define_register!(
  RegisterCompilationExecuteModuleTaps,
  tap = CompilationExecuteModuleTap<JsExecuteModuleArg, ()> @ CompilationExecuteModuleHook,
  cache = false,
  sync = true,
);
define_register!(
  RegisterCompilationFinishModulesTaps,
  tap = CompilationFinishModulesTap<JsCompilation, Promise<()>> @ CompilationFinishModulesHook,
  cache = false,
  sync = false,
);
define_register!(
  RegisterCompilationOptimizeModulesTaps,
  tap = CompilationOptimizeModulesTap<(), Option<bool>> @ CompilationOptimizeModulesHook,
  cache = true,
  sync = false,
);
define_register!(
  RegisterCompilationAfterOptimizeModulesTaps,
  tap = CompilationAfterOptimizeModulesTap<(), ()> @ CompilationAfterOptimizeModulesHook,
  cache = false,
  sync = false,
);
define_register!(
  RegisterCompilationOptimizeTreeTaps,
  tap = CompilationOptimizeTreeTap<(), Promise<()>> @ CompilationOptimizeTreeHook,
  cache = false,
  sync = false,
);
define_register!(
  RegisterCompilationOptimizeChunkModulesTaps,
  tap = CompilationOptimizeChunkModulesTap<(), Promise<Option<bool>>> @ CompilationOptimizeChunkModulesHook,
  cache = false,
  sync = false,
);
define_register!(
  RegisterCompilationRuntimeModuleTaps,
  tap = CompilationRuntimeModuleTap<JsRuntimeModuleArg, Option<JsRuntimeModule>> @ CompilationRuntimeModuleHook,
  cache = true,
  sync = false,
);
define_register!(
  RegisterCompilationChunkAssetTaps,
  tap = CompilationChunkAssetTap<JsChunkAssetArgs, ()> @ CompilationChunkAssetHook,
  cache = true,
  sync = false,
);
define_register!(
  RegisterCompilationProcessAssetsTaps,
  tap = CompilationProcessAssetsTap<JsCompilation, Promise<()>> @ CompilationProcessAssetsHook,
  cache = false,
  sync = false,
);
define_register!(
  RegisterCompilationAfterProcessAssetsTaps,
  tap = CompilationAfterProcessAssetsTap<JsCompilation, ()> @ CompilationAfterProcessAssetsHook,
  cache = false,
  sync = false,
);

/* NormalModuleFactory Hooks */
define_register!(
  RegisterNormalModuleFactoryBeforeResolveTaps,
  tap = NormalModuleFactoryBeforeResolveTap<JsBeforeResolveArgs, Promise<JsBeforeResolveOutput>> @ NormalModuleFactoryBeforeResolveHook,
  cache = true,
  sync = false,
);
define_register!(
  RegisterNormalModuleFactoryAfterResolveTaps,
  tap = NormalModuleFactoryAfterResolveTap<JsAfterResolveData, Promise<JsAfterResolveOutput>> @ NormalModuleFactoryAfterResolveHook,
  cache = true,
  sync = false,
);

#[async_trait]
impl AsyncSeries2<Compilation, CompilationParams> for CompilerThisCompilationTap {
  async fn run(
    &self,
    compilation: &mut Compilation,
    _: &mut CompilationParams,
  ) -> rspack_error::Result<()> {
    // SAFETY:
    // 1. `Compiler` is stored on the heap and pinned in binding crate.
    // 2. `Compilation` outlives `JsCompilation` and `Compiler` outlives `Compilation`.
    // 3. `JsCompilation` was replaced everytime a new `Compilation` was created before getting accessed.
    let compilation = unsafe { JsCompilation::from_compilation(compilation) };
    self.function.call_with_sync(compilation).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl AsyncSeries2<Compilation, CompilationParams> for CompilerCompilationTap {
  async fn run(
    &self,
    compilation: &mut Compilation,
    _: &mut CompilationParams,
  ) -> rspack_error::Result<()> {
    // SAFETY:
    // 1. `Compiler` is stored on the heap and pinned in binding crate.
    // 2. `Compilation` outlives `JsCompilation` and `Compiler` outlives `Compilation`.
    // 3. `JsCompilation` was replaced everytime a new `Compilation` was created before getting accessed.
    let compilation = unsafe { JsCompilation::from_compilation(compilation) };
    self.function.call_with_sync(compilation).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl AsyncSeries2<Compilation, Vec<MakeParam>> for CompilerMakeTap {
  async fn run(
    &self,
    compilation: &mut Compilation,
    _: &mut Vec<MakeParam>,
  ) -> rspack_error::Result<()> {
    // SAFETY:
    // 1. `Compiler` is stored on the heap and pinned in binding crate.
    // 2. `Compilation` outlives `JsCompilation` and `Compiler` outlives `Compilation`.
    // 3. `JsCompilation` was replaced everytime a new `Compilation` was created before getting accessed.
    let compilation = unsafe { JsCompilation::from_compilation(compilation) };

    self.function.call_with_promise(compilation).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl AsyncSeries<Compilation> for CompilerFinishMakeTap {
  async fn run(&self, compilation: &mut Compilation) -> rspack_error::Result<()> {
    // SAFETY:
    // 1. `Compiler` is stored on the heap and pinned in binding crate.
    // 2. `Compilation` outlives `JsCompilation` and `Compiler` outlives `Compilation`.
    // 3. `JsCompilation` was replaced everytime a new `Compilation` was created before getting accessed.
    let compilation = unsafe { JsCompilation::from_compilation(compilation) };

    self.function.call_with_promise(compilation).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl AsyncSeriesBail<Compilation, bool> for CompilerShouldEmitTap {
  async fn run(&self, compilation: &mut Compilation) -> rspack_error::Result<Option<bool>> {
    // SAFETY:
    // 1. `Compiler` is stored on the heap and pinned in binding crate.
    // 2. `Compilation` outlives `JsCompilation` and `Compiler` outlives `Compilation`.
    // 3. `JsCompilation` was replaced everytime a new `Compilation` was created before getting accessed.
    let compilation = unsafe { JsCompilation::from_compilation(compilation) };

    self.function.call_with_sync(compilation).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl AsyncSeries<Compilation> for CompilerEmitTap {
  async fn run(&self, _compilation: &mut Compilation) -> rspack_error::Result<()> {
    self.function.call_with_promise(()).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl AsyncSeries<Compilation> for CompilerAfterEmitTap {
  async fn run(&self, _compilation: &mut Compilation) -> rspack_error::Result<()> {
    self.function.call_with_promise(()).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl AsyncParallel3<Compilation, String, AssetEmittedInfo> for CompilerAssetEmittedTap {
  async fn run(
    &self,
    _compilation: &Compilation,
    filename: &String,
    info: &AssetEmittedInfo,
  ) -> rspack_error::Result<()> {
    self
      .function
      .call_with_promise(JsAssetEmittedArgs {
        filename: filename.to_string(),
        output_path: info.output_path.to_string_lossy().into_owned(),
        target_path: info.target_path.to_string_lossy().into_owned(),
      })
      .await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl AsyncSeries<BoxModule> for CompilationBuildModuleTap {
  async fn run(&self, module: &mut BoxModule) -> rspack_error::Result<()> {
    self
      .function
      .call_with_sync(module.to_js_module().expect("Convert to js_module failed."))
      .await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl AsyncSeries<BoxModule> for CompilationStillValidModuleTap {
  async fn run(&self, module: &mut BoxModule) -> rspack_error::Result<()> {
    self
      .function
      .call_with_sync(module.to_js_module().expect("Convert to js_module failed."))
      .await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl AsyncSeries<BoxModule> for CompilationSucceedModuleTap {
  async fn run(&self, module: &mut BoxModule) -> rspack_error::Result<()> {
    self
      .function
      .call_with_sync(module.to_js_module().expect("Convert to js_module failed."))
      .await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl SyncSeries4<ModuleIdentifier, IdentifierSet, CodeGenerationResults, ExecuteModuleId>
  for CompilationExecuteModuleTap
{
  fn run(
    &self,
    entry: &mut ModuleIdentifier,
    runtime_modules: &mut IdentifierSet,
    codegen_results: &mut CodeGenerationResults,
    id: &mut ExecuteModuleId,
  ) -> rspack_error::Result<()> {
    self.function.blocking_call_with_sync(JsExecuteModuleArg {
      entry: entry.to_string(),
      runtime_modules: runtime_modules.iter().map(|id| id.to_string()).collect(),
      codegen_results: codegen_results.clone().into(),
      id: *id,
    })
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl AsyncSeries<Compilation> for CompilationFinishModulesTap {
  async fn run(&self, compilation: &mut Compilation) -> rspack_error::Result<()> {
    // SAFETY:
    // 1. `Compiler` is stored on the heap and pinned in binding crate.
    // 2. `Compilation` outlives `JsCompilation` and `Compiler` outlives `Compilation`.
    // 3. `JsCompilation` was replaced everytime a new `Compilation` was created before getting accessed.
    let compilation = unsafe { JsCompilation::from_compilation(compilation) };

    self.function.call_with_promise(compilation).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl AsyncSeriesBail<Compilation, bool> for CompilationOptimizeModulesTap {
  async fn run(&self, _compilation: &mut Compilation) -> rspack_error::Result<Option<bool>> {
    self.function.call_with_sync(()).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl AsyncSeries<Compilation> for CompilationAfterOptimizeModulesTap {
  async fn run(&self, _compilation: &mut Compilation) -> rspack_error::Result<()> {
    self.function.call_with_sync(()).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl AsyncSeries<Compilation> for CompilationOptimizeTreeTap {
  async fn run(&self, _compilation: &mut Compilation) -> rspack_error::Result<()> {
    self.function.call_with_promise(()).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl AsyncSeriesBail<Compilation, bool> for CompilationOptimizeChunkModulesTap {
  async fn run(&self, _compilation: &mut Compilation) -> rspack_error::Result<Option<bool>> {
    self.function.call_with_promise(()).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl AsyncSeries3<Compilation, ModuleIdentifier, ChunkUkey> for CompilationRuntimeModuleTap {
  async fn run(
    &self,
    compilation: &mut Compilation,
    m: &mut ModuleIdentifier,
    c: &mut ChunkUkey,
  ) -> rspack_error::Result<()> {
    let Some(module) = compilation.runtime_modules.get(m) else {
      return Ok(());
    };
    let chunk = compilation.chunk_by_ukey.expect_get(c);
    let arg = JsRuntimeModuleArg {
      module: JsRuntimeModule {
        source: Some(
          module
            .generate(compilation)?
            .to_js_compat_source()
            .unwrap_or_else(|err| panic!("Failed to generate runtime module source: {err}")),
        ),
        module_identifier: module.identifier().to_string(),
        constructor_name: module.get_constructor_name(),
        name: module.name().to_string().replace("webpack/runtime/", ""),
      },
      chunk: JsChunk::from(chunk),
    };
    if let Some(module) = self.function.call_with_sync(arg).await?
      && let Some(source) = module.source
    {
      let module = compilation
        .runtime_modules
        .get_mut(m)
        .expect("should have module");
      module.set_custom_source(CompatSource::from(source).boxed())
    }
    Ok(())
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl AsyncSeries2<Chunk, String> for CompilationChunkAssetTap {
  async fn run(&self, chunk: &mut Chunk, file: &mut String) -> rspack_error::Result<()> {
    self
      .function
      .call_with_sync(JsChunkAssetArgs {
        chunk: JsChunk::from(chunk),
        filename: file.to_string(),
      })
      .await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl AsyncSeries<Compilation> for CompilationProcessAssetsTap {
  async fn run(&self, compilation: &mut Compilation) -> rspack_error::Result<()> {
    // SAFETY:
    // 1. `Compiler` is stored on the heap and pinned in binding crate.
    // 2. `Compilation` outlives `JsCompilation` and `Compiler` outlives `Compilation`.
    // 3. `JsCompilation` was replaced everytime a new `Compilation` was created before getting accessed.
    let compilation = unsafe { JsCompilation::from_compilation(compilation) };

    self.function.call_with_promise(compilation).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl AsyncSeries<Compilation> for CompilationAfterProcessAssetsTap {
  async fn run(&self, compilation: &mut Compilation) -> rspack_error::Result<()> {
    // SAFETY:
    // 1. `Compiler` is stored on the heap and pinned in binding crate.
    // 2. `Compilation` outlives `JsCompilation` and `Compiler` outlives `Compilation`.
    // 3. `JsCompilation` was replaced everytime a new `Compilation` was created before getting accessed.
    let compilation = unsafe { JsCompilation::from_compilation(compilation) };

    self.function.call_with_sync(compilation).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl AsyncSeriesBail<BeforeResolveArgs, bool> for NormalModuleFactoryBeforeResolveTap {
  async fn run(&self, args: &mut BeforeResolveArgs) -> rspack_error::Result<Option<bool>> {
    match self.function.call_with_promise(args.clone().into()).await {
      Ok((ret, resolve_data)) => {
        args.request = resolve_data.request;
        args.context = resolve_data.context;
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
impl AsyncSeriesBail4<String, ModuleFactoryCreateData, FactoryMeta, CreateData, bool>
  for NormalModuleFactoryAfterResolveTap
{
  async fn run(
    &self,
    request: &mut String,
    data: &mut ModuleFactoryCreateData,
    meta: &mut FactoryMeta,
    create_data: &mut CreateData,
  ) -> rspack_error::Result<Option<bool>> {
    match self
      .function
      .call_with_promise(JsAfterResolveData {
        request: request.to_string(),
        context: data.context.to_string(),
        file_dependencies: data
          .file_dependencies
          .clone()
          .into_iter()
          .map(|item| item.to_string_lossy().to_string())
          .collect::<Vec<_>>(),
        context_dependencies: data
          .context_dependencies
          .clone()
          .into_iter()
          .map(|item| item.to_string_lossy().to_string())
          .collect::<Vec<_>>(),
        missing_dependencies: data
          .missing_dependencies
          .clone()
          .into_iter()
          .map(|item| item.to_string_lossy().to_string())
          .collect::<Vec<_>>(),
        factory_meta: JsFactoryMeta {
          side_effect_free: meta.side_effect_free,
        },
        create_data: Some(JsCreateData {
          request: create_data.request.to_owned(),
          user_request: create_data.user_request.to_owned(),
          resource: create_data
            .resource
            .resource_path
            .to_string_lossy()
            .to_string(),
        }),
      })
      .await
    {
      Ok((ret, resolve_data)) => {
        if let Some(resolve_data) = resolve_data {
          fn override_resource(origin_data: &ResourceData, new_resource: String) -> ResourceData {
            let mut resource_data = origin_data.clone();
            let origin_resource_path = origin_data.resource_path.to_string_lossy().to_string();
            resource_data.resource_path = new_resource.clone().into();
            resource_data.resource = resource_data
              .resource
              .replace(&origin_resource_path, &new_resource);

            resource_data
          }

          let request = resolve_data.request;
          let user_request = resolve_data.user_request;
          let resource = override_resource(&create_data.resource, resolve_data.resource);

          create_data.request = request;
          create_data.user_request = user_request;
          create_data.resource = resource;
        }

        Ok(ret)
      }
      Err(err) => Err(err),
    }
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}
