use async_trait::async_trait;
use napi::{
  bindgen_prelude::{FromNapiValue, Promise, ToNapiValue},
  Either, Env, JsFunction, NapiRaw,
};
use rspack_binding_values::JsCompilation;
use rspack_core::{
  Compilation, CompilationParams, CompilationProcessAssetsHook, CompilerCompilationHook,
  CompilerMakeHook, MakeParam,
};
use rspack_hook::{AsyncSeries, AsyncSeries2, Hook, Interceptor};
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

type MaybePromiseUndefined = Either<(), Promise<()>>;

#[derive(Clone)]
#[napi(object, object_to_js = false)]
pub struct RegisterJsTaps {
  #[napi(ts_type = "(arg: Array<number>) => any")]
  pub register_compiler_compilation_taps:
    ThreadsafeFunction<Vec<i32>, Vec<ThreadsafeJsTap<JsCompilation, MaybePromiseUndefined>>>,
  #[napi(ts_type = "(arg: Array<number>) => any")]
  pub register_compiler_make_taps:
    ThreadsafeFunction<Vec<i32>, Vec<ThreadsafeJsTap<JsCompilation, Promise<()>>>>,
  #[napi(ts_type = "(arg: Array<number>) => any")]
  pub register_compilation_process_assets_taps:
    ThreadsafeFunction<Vec<i32>, Vec<ThreadsafeJsTap<JsCompilation, Promise<()>>>>,
}

impl<R> FromNapiValue for ThreadsafeJsTap<JsCompilation, R> {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    let t = JsTap::from_napi_value(env, napi_val)?;
    ThreadsafeJsTap::from_js_tap(t, Env::from_raw(env))
  }
}

#[derive(Clone)]
struct CompilerCompilationTap {
  function: ThreadsafeFunction<JsCompilation, Either<(), Promise<()>>>,
  stage: i32,
}

impl CompilerCompilationTap {
  pub fn new(tap: ThreadsafeJsTap<JsCompilation, Either<(), Promise<()>>>) -> Self {
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
    self.function.call_with_auto(compilation).await
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
