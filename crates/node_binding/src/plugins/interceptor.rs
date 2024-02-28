use std::sync::Arc;

use async_trait::async_trait;
use napi::{Env, JsFunction, NapiRaw};
use rspack_binding_macros::{call_js_function_with_napi_objects, js_fn_into_threadsafe_fn};
use rspack_binding_values::JsCompilation;
use rspack_core::{
  Compilation, CompilationParams, CompilerCompilationHook, CompilerMakeHook, MakeParam,
};
use rspack_hook::{AsyncSeries2, Interceptor};
use rspack_napi_shared::{
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
  NapiResultExt,
};

#[napi(string_enum)]
#[derive(Debug)]
pub enum JsTapType {
  CompilerCompilation,
  CompilerMake,
}

#[napi(object)]
pub struct JsTap {
  pub function: JsFunction,
  pub stage: i32,
}

pub struct ThreadsafeJsTap<T: 'static, R> {
  pub function: ThreadsafeFunction<T, R>,
  pub stage: i32,
}

impl<
    T: napi::bindgen_prelude::ToNapiValue,
    R: 'static + Send + napi::bindgen_prelude::FromNapiValue,
  > ThreadsafeJsTap<T, R>
{
  pub fn from_js_tap(js_tap: JsTap, env: Env) -> napi::Result<Self> {
    Ok(Self {
      function: js_fn_into_threadsafe_fn!(js_tap.function, env),
      stage: js_tap.stage,
    })
  }
}

fn register_taps_fn_into_threadsafe_fn<
  S: napi::bindgen_prelude::ToNapiValue,
  T: napi::bindgen_prelude::ToNapiValue,
  R: 'static + Send + napi::bindgen_prelude::FromNapiValue,
>(
  js_cb: JsFunction,
  env: Env,
) -> napi::Result<ThreadsafeFunction<S, Vec<ThreadsafeJsTap<T, R>>>> {
  let cb = unsafe { js_cb.raw() };
  let mut tsfn = ThreadsafeFunction::create(env.raw(), cb, 0, |ctx| {
    let (ctx, resolver) = ctx.split_into_parts();

    let env = ctx.env;
    let cb = ctx.callback;
    let result = unsafe { call_js_function_with_napi_objects!(env, cb, ctx.value) };

    resolver.resolve_non_promise(result, |env, result: Vec<JsTap>| {
      Ok(
        result
          .into_iter()
          .map(|t| ThreadsafeJsTap::from_js_tap(t, *env))
          .collect::<napi::Result<Vec<_>>>()?,
      )
    })
  })?;

  // See the comment in `threadsafe_function.rs`
  tsfn.unref(&env)?;
  Ok(tsfn)
}

#[napi(object)]
pub struct RegisterJsTaps {
  pub register_compiler_compilation_taps: JsFunction,
  pub register_compiler_make_taps: JsFunction,
}

#[derive(Clone)]
pub struct ThreadsafeRegisterJsTaps(Arc<ThreadsafeRegisterJsTapsInner>);

struct ThreadsafeRegisterJsTapsInner {
  register_compiler_compilation_taps:
    ThreadsafeFunction<Vec<i32>, Vec<ThreadsafeJsTap<JsCompilation, ()>>>,
  register_compiler_make_taps:
    ThreadsafeFunction<Vec<i32>, Vec<ThreadsafeJsTap<JsCompilation, ()>>>,
}

impl ThreadsafeRegisterJsTaps {
  pub fn from_js_taps(register_js_taps: RegisterJsTaps, env: Env) -> napi::Result<Self> {
    Ok(Self(Arc::new(ThreadsafeRegisterJsTapsInner {
      register_compiler_compilation_taps: register_taps_fn_into_threadsafe_fn(
        register_js_taps.register_compiler_compilation_taps,
        env,
      )?,
      register_compiler_make_taps: register_taps_fn_into_threadsafe_fn(
        register_js_taps.register_compiler_make_taps,
        env,
      )?,
    })))
  }
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

    self
      .function
      .call(compilation, ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .unwrap_or_else(|err| panic!("Failed to call compilation: {err}"))
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl Interceptor<CompilerCompilationHook> for ThreadsafeRegisterJsTaps {
  async fn call(&self, hook: &CompilerCompilationHook) -> rspack_error::Result<()> {
    let mut used_stages = Vec::from_iter(hook.used_stages());
    used_stages.sort();
    let js_taps: Vec<ThreadsafeJsTap<JsCompilation, ()>> = self
      .0
      .register_compiler_compilation_taps
      .call(used_stages, ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .unwrap_or_else(|err| panic!("Failed to call compiler.hooks.compilation: {err}"))?;
    for js_tap in js_taps {
      hook.tap(Box::new(CompilerCompilationTap::new(js_tap)));
    }
    Ok(())
  }
}

#[derive(Clone)]
struct CompilerMakeTap {
  function: ThreadsafeFunction<JsCompilation, ()>,
  stage: i32,
}

impl CompilerMakeTap {
  pub fn new(tap: ThreadsafeJsTap<JsCompilation, ()>) -> Self {
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

    self
      .function
      .call(compilation, ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .unwrap_or_else(|err| panic!("Failed to call compilation: {err}"))
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl Interceptor<CompilerMakeHook> for ThreadsafeRegisterJsTaps {
  async fn call(&self, hook: &CompilerMakeHook) -> rspack_error::Result<()> {
    let mut used_stages = Vec::from_iter(hook.used_stages());
    used_stages.sort();
    let js_taps: Vec<ThreadsafeJsTap<JsCompilation, ()>> = self
      .0
      .register_compiler_make_taps
      .call(used_stages, ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .unwrap_or_else(|err| panic!("Failed to call compiler.hooks.make: {err}"))?;
    for js_tap in js_taps {
      hook.tap(Box::new(CompilerMakeTap::new(js_tap)));
    }
    Ok(())
  }
}
