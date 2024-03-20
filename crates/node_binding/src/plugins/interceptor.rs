use std::{borrow::Cow, sync::Arc};

use async_trait::async_trait;
use napi::{
  bindgen_prelude::{FromNapiValue, Promise, ToNapiValue},
  Env, JsFunction, NapiRaw,
};
use rspack_binding_values::{
  CompatSource, JsBeforeResolveArgs, JsBeforeResolveOutput, JsChunk, JsCompilation,
  JsExecuteModuleArg, JsRuntimeModule, JsRuntimeModuleArg, ToJsCompatSource,
};
use rspack_core::{
  rspack_sources::SourceExt, BeforeResolveArgs, ChunkUkey, CodeGenerationResults, Compilation,
  CompilationExecuteModuleHook, CompilationParams, CompilationProcessAssetsHook,
  CompilationRuntimeModuleHook, CompilerCompilationHook, CompilerMakeHook, CompilerShouldEmitHook,
  CompilerThisCompilationHook, ExecuteModuleId, MakeParam, ModuleIdentifier,
  NormalModuleFactoryBeforeResolveHook,
};
use rspack_hook::{
  AsyncSeries, AsyncSeries2, AsyncSeries3, AsyncSeriesBail, Hook, Interceptor, SyncSeries4,
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
    ts_type = "(stages: Array<number>) => Array<{ function: ((compilation: JsCompilation) => void); stage: number; }>"
  )]
  pub register_compiler_this_compilation_taps: RegisterFunction<JsCompilation, ()>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((compilation: JsCompilation) => void); stage: number; }>"
  )]
  pub register_compiler_compilation_taps: RegisterFunction<JsCompilation, ()>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((compilation: JsCompilation) => Promise<void>); stage: number; }>"
  )]
  pub register_compiler_make_taps: RegisterFunction<JsCompilation, Promise<()>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((compilation: JsCompilation) => boolean | undefined); stage: number; }>"
  )]
  pub register_compiler_should_emit_taps: RegisterFunction<JsCompilation, Option<bool>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((compilation: JsExecuteModuleArg) => void); stage: number; }>"
  )]
  pub register_compilation_execute_module_taps: RegisterFunction<JsExecuteModuleArg, ()>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((compilation: JsRuntimeModuleArg) => JsRuntimeModule | undefined); stage: number; }>"
  )]
  pub register_compilation_runtime_module_taps:
    RegisterFunction<JsRuntimeModuleArg, Option<JsRuntimeModule>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((compilation: JsCompilation) => Promise<void>); stage: number; }>"
  )]
  pub register_compilation_process_assets_taps: RegisterFunction<JsCompilation, Promise<()>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((compilation: JsBeforeResolveArgs) => Promise<[boolean | undefined, JsBeforeResolveArgs]>); stage: number; }>"
  )]
  pub register_normal_module_factory_before_resolve_taps:
    RegisterFunction<JsBeforeResolveArgs, Promise<JsBeforeResolveOutput>>,
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
  RegisterCompilerShouldEmitTaps,
  tap = CompilerShouldEmitTap<JsCompilation, Option<bool>> @ CompilerShouldEmitHook,
  cache = false,
  sync = false,
);

/* Compilation Hooks */
define_register!(
  RegisterCompilationExecuteModuleTaps,
  tap = CompilationExecuteModuleTap<JsExecuteModuleArg, ()> @ CompilationExecuteModuleHook,
  cache = false,
  sync = true,
);
define_register!(
  RegisterCompilationRuntimeModuleTaps,
  tap = CompilationRuntimeModuleTap<JsRuntimeModuleArg, Option<JsRuntimeModule>> @ CompilationRuntimeModuleHook,
  cache = true,
  sync = false,
);
define_register!(
  RegisterCompilationProcessAssetsTaps,
  tap = CompilationProcessAssetsTap<JsCompilation, Promise<()>> @ CompilationProcessAssetsHook,
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
    tokio::runtime::Handle::current().block_on(self.function.call(JsExecuteModuleArg {
      entry: entry.to_string(),
      runtime_modules: runtime_modules.iter().map(|id| id.to_string()).collect(),
      codegen_results: codegen_results.clone().into(),
      id: *id,
    }))
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
            .generate(compilation)
            .to_js_compat_source()
            .unwrap_or_else(|err| panic!("Failed to generate runtime module source: {err}")),
        ),
        module_identifier: module.identifier().to_string(),
        constructor_name: module.get_constructor_name(),
        name: module.name().to_string().replace("webpack/runtime/", ""),
      },
      chunk: JsChunk::from(chunk),
    };
    if let Some(module) = self.function.call(arg).await?
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
