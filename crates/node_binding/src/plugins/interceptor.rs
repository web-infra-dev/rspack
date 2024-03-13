use async_trait::async_trait;
use napi::{
  bindgen_prelude::{FromNapiValue, Promise, ToNapiValue},
  Env, JsFunction, NapiRaw,
};
use rspack_binding_values::{JsBeforeResolveArgs, JsBeforeResolveOutput, JsCompilation};
use rspack_core::{
  BeforeResolveArgs, Compilation, CompilationParams, CompilationProcessAssetsHook,
  CompilerCompilationHook, CompilerMakeHook, MakeParam, NormalModuleFactoryBeforeResolveHook,
};
use rspack_hook::{AsyncSeries, AsyncSeries2, AsyncSeriesBail, Hook, Interceptor};
use rspack_napi_shared::new_tsfn::ThreadsafeFunction;

#[napi(object)]
pub struct JsTap {
  pub function: JsFunction,
  pub stage: i32,
}

pub struct ThreadsafeJsTap<T: 'static + ToNapiValue, R> {
  pub function: ThreadsafeFunction<T, R>,
  pub stage: i32,
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

#[derive(Clone)]
#[napi(object, object_to_js = false)]
pub struct RegisterJsTaps {
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((compilation: JsCompilation) => void); stage: number; }>"
  )]
  pub register_compiler_compilation_taps:
    ThreadsafeFunction<Vec<i32>, Vec<ThreadsafeJsTap<JsCompilation, ()>>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((compilation: JsCompilation) => Promise<void>); stage: number; }>"
  )]
  pub register_compiler_make_taps:
    ThreadsafeFunction<Vec<i32>, Vec<ThreadsafeJsTap<JsCompilation, Promise<()>>>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((compilation: JsCompilation) => Promise<void>); stage: number; }>"
  )]
  pub register_compilation_process_assets_taps:
    ThreadsafeFunction<Vec<i32>, Vec<ThreadsafeJsTap<JsCompilation, Promise<()>>>>,
  #[napi(
    ts_type = "(stages: Array<number>) => Array<{ function: ((compilation: JsBeforeResolveArgs) => Promise<void>); stage: number; }>"
  )]
  pub register_normal_module_factory_before_resolve_taps: ThreadsafeFunction<
    Vec<i32>,
    Vec<ThreadsafeJsTap<JsBeforeResolveArgs, Promise<JsBeforeResolveOutput>>>,
  >,
}

#[derive(Clone)]
struct CompilerCompilationTap {
  function: ThreadsafeFunction<JsCompilation, ()>,
  stage: i32,
}

impl CompilerCompilationTap {
  pub fn new(tap: ThreadsafeJsTap<JsCompilation, ()>) -> Self {
    Self {
      function: tap.function,
      stage: tap.stage,
    }
  }
}

#[async_trait]
impl AsyncSeries2<Compilation, CompilationParams> for CompilerCompilationTap {
  async fn run(
    &self,
    compilation: &mut Compilation,
    _: &mut CompilationParams,
  ) -> rspack_error::Result<()> {
    let compilation = JsCompilation::from_compilation(unsafe {
      std::mem::transmute::<&'_ mut rspack_core::Compilation, &'static mut rspack_core::Compilation>(
        compilation,
      )
    });
    self.function.call_with_sync(compilation).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl Interceptor<CompilerCompilationHook> for RegisterJsTaps {
  async fn call(
    &self,
    hook: &CompilerCompilationHook,
  ) -> rspack_error::Result<Vec<<CompilerCompilationHook as Hook>::Tap>> {
    let mut used_stages = Vec::from_iter(hook.used_stages());
    used_stages.sort();
    let js_taps = self
      .register_compiler_compilation_taps
      .call_with_sync(used_stages)
      .await?;
    let js_taps = js_taps
      .into_iter()
      .map(|t| Box::new(CompilerCompilationTap::new(t)) as <CompilerCompilationHook as Hook>::Tap)
      .collect();
    Ok(js_taps)
  }
}

#[derive(Clone)]
struct CompilerMakeTap {
  function: ThreadsafeFunction<JsCompilation, Promise<()>>,
  stage: i32,
}

impl CompilerMakeTap {
  pub fn new(tap: ThreadsafeJsTap<JsCompilation, Promise<()>>) -> Self {
    Self {
      function: tap.function,
      stage: tap.stage,
    }
  }
}

#[async_trait]
impl AsyncSeries2<Compilation, Vec<MakeParam>> for CompilerMakeTap {
  async fn run(
    &self,
    compilation: &mut Compilation,
    _: &mut Vec<MakeParam>,
  ) -> rspack_error::Result<()> {
    let compilation = JsCompilation::from_compilation(unsafe {
      std::mem::transmute::<&'_ mut rspack_core::Compilation, &'static mut rspack_core::Compilation>(
        compilation,
      )
    });

    self.function.call_with_promise(compilation).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl Interceptor<CompilerMakeHook> for RegisterJsTaps {
  async fn call(
    &self,
    hook: &CompilerMakeHook,
  ) -> rspack_error::Result<Vec<<CompilerMakeHook as Hook>::Tap>> {
    let mut used_stages = Vec::from_iter(hook.used_stages());
    used_stages.sort();
    let js_taps = self
      .register_compiler_make_taps
      .call_with_sync(used_stages)
      .await?;
    let js_taps = js_taps
      .into_iter()
      .map(|t| Box::new(CompilerMakeTap::new(t)) as <CompilerMakeHook as Hook>::Tap)
      .collect();
    Ok(js_taps)
  }
}

#[derive(Clone)]
struct CompilationProcessAssetsTap {
  function: ThreadsafeFunction<JsCompilation, Promise<()>>,
  stage: i32,
}

impl CompilationProcessAssetsTap {
  pub fn new(tap: ThreadsafeJsTap<JsCompilation, Promise<()>>) -> Self {
    Self {
      function: tap.function,
      stage: tap.stage,
    }
  }
}

#[async_trait]
impl AsyncSeries<Compilation> for CompilationProcessAssetsTap {
  async fn run(&self, compilation: &mut Compilation) -> rspack_error::Result<()> {
    let compilation = JsCompilation::from_compilation(unsafe {
      std::mem::transmute::<&'_ mut rspack_core::Compilation, &'static mut rspack_core::Compilation>(
        compilation,
      )
    });

    self.function.call_with_promise(compilation).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl Interceptor<CompilationProcessAssetsHook> for RegisterJsTaps {
  async fn call(
    &self,
    hook: &CompilationProcessAssetsHook,
  ) -> rspack_error::Result<Vec<<CompilationProcessAssetsHook as Hook>::Tap>> {
    let mut used_stages = Vec::from_iter(hook.used_stages());
    used_stages.sort();
    let js_taps = self
      .register_compilation_process_assets_taps
      .call_with_sync(used_stages)
      .await?;
    let js_taps = js_taps
      .into_iter()
      .map(|t| {
        Box::new(CompilationProcessAssetsTap::new(t)) as <CompilationProcessAssetsHook as Hook>::Tap
      })
      .collect();
    Ok(js_taps)
  }
}

#[derive(Clone)]
struct NormalModuleFactoryBeforeResolveTap {
  function: ThreadsafeFunction<JsBeforeResolveArgs, Promise<(Option<bool>, JsBeforeResolveArgs)>>,
  stage: i32,
}

impl NormalModuleFactoryBeforeResolveTap {
  pub fn new(
    tap: ThreadsafeJsTap<JsBeforeResolveArgs, Promise<(Option<bool>, JsBeforeResolveArgs)>>,
  ) -> Self {
    Self {
      function: tap.function,
      stage: tap.stage,
    }
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
impl Interceptor<NormalModuleFactoryBeforeResolveHook> for RegisterJsTaps {
  async fn call(
    &self,
    hook: &NormalModuleFactoryBeforeResolveHook,
  ) -> rspack_error::Result<Vec<<NormalModuleFactoryBeforeResolveHook as Hook>::Tap>> {
    let mut used_stages = Vec::from_iter(hook.used_stages());
    used_stages.sort();
    let js_taps = self
      .register_normal_module_factory_before_resolve_taps
      .call_with_sync(used_stages)
      .await?;
    let js_taps = js_taps
      .into_iter()
      .map(|t| {
        Box::new(NormalModuleFactoryBeforeResolveTap::new(t))
          as <NormalModuleFactoryBeforeResolveHook as Hook>::Tap
      })
      .collect();
    Ok(js_taps)
  }
}
