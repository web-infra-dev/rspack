use async_trait::async_trait;
use napi::{
  bindgen_prelude::{FromNapiValue, Promise, ToNapiValue},
  Env, JsFunction, NapiRaw,
};
use paste::paste;
use rspack_binding_values::{JsBeforeResolveArgs, JsBeforeResolveOutput, JsCompilation};
use rspack_core::{
  BeforeResolveArgs, Compilation, CompilationParams, CompilationProcessAssetsHook,
  CompilerCompilationHook, CompilerMakeHook, MakeParam, NormalModuleFactoryBeforeResolveHook,
};
use rspack_hook::{AsyncSeries, AsyncSeries2, AsyncSeriesBail, Hook, Interceptor};
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use tokio::sync::Mutex;

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

struct RegisterJsTapsInner<T: 'static, R> {
  register: ThreadsafeFunction<Vec<i32>, Vec<ThreadsafeJsTap<T, R>>>,
  cache: RegisterJsTapsCache<T, R>,
}

enum RegisterJsTapsCache<T: 'static, R> {
  NoCache,
  Cache(Mutex<Option<Vec<ThreadsafeJsTap<T, R>>>>),
}

impl<T: 'static, R> RegisterJsTapsCache<T, R> {
  pub fn new(cache: bool) -> Self {
    if cache {
      Self::Cache(Default::default())
    } else {
      Self::NoCache
    }
  }
}

impl<T: 'static + ToNapiValue, R: 'static> RegisterJsTapsInner<T, R> {
  pub fn new(
    register: ThreadsafeFunction<Vec<i32>, Vec<ThreadsafeJsTap<T, R>>>,
    cache: bool,
  ) -> Self {
    Self {
      register,
      cache: RegisterJsTapsCache::new(cache),
    }
  }

  pub async fn call_register(
    &self,
    hook: &impl Hook,
  ) -> rspack_error::Result<Vec<ThreadsafeJsTap<T, R>>> {
    if let RegisterJsTapsCache::Cache(cache) = &self.cache {
      let mut cache = cache.lock().await;
      if let Some(cache) = &*cache {
        Ok(cache.clone())
      } else {
        let js_taps = self.call_register_impl(hook).await?;
        cache.replace(js_taps.clone());
        Ok(js_taps)
      }
    } else {
      self.call_register_impl(hook).await
    }
  }

  async fn call_register_impl(
    &self,
    hook: &impl Hook,
  ) -> rspack_error::Result<Vec<ThreadsafeJsTap<T, R>>> {
    let mut used_stages = Vec::from_iter(hook.used_stages());
    used_stages.sort();
    self.register.call_with_sync(used_stages).await
  }
}

macro_rules! define_register {
  ($name:ident, tap = $tap_name:ident<$arg:ty, $ret:ty> @ $tap_hook:ty, cache = $cache:expr,) => {
    paste! {
      impl RegisterJsTaps {
        pub fn [<intercept_ $name>](&self, hooks: &mut $tap_hook) {
          hooks.intercept([<Register $tap_name s>]::new(self.$name.clone()))
        }
      }

      struct [<Register $tap_name s>] {
        inner: RegisterJsTapsInner<$arg, $ret>,
      }

      impl [<Register $tap_name s>] {
        pub fn new(
          register: ThreadsafeFunction<Vec<i32>, Vec<ThreadsafeJsTap<$arg, $ret>>>,
        ) -> Self {
          Self {
            inner: RegisterJsTapsInner::new(register, $cache),
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

      #[async_trait]
      impl Interceptor<$tap_hook> for [<Register $tap_name s>] {
        async fn call(
          &self,
          hook: &$tap_hook,
        ) -> rspack_error::Result<Vec<<$tap_hook as Hook>::Tap>> {
          let js_taps = self.inner.call_register(hook).await?;
          let js_taps = js_taps
            .into_iter()
            .map(|t| {
              Box::new($tap_name::new(t)) as <$tap_hook as Hook>::Tap
            })
            .collect();
          Ok(js_taps)
        }
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
    ts_type = "(stages: Array<number>) => Array<{ function: ((compilation: JsBeforeResolveArgs) => Promise<[boolean | undefined, JsBeforeResolveArgs]>); stage: number; }>"
  )]
  pub register_normal_module_factory_before_resolve_taps: ThreadsafeFunction<
    Vec<i32>,
    Vec<ThreadsafeJsTap<JsBeforeResolveArgs, Promise<JsBeforeResolveOutput>>>,
  >,
}

define_register!(
  register_compiler_compilation_taps,
  tap = CompilerCompilationTap<JsCompilation, ()> @ CompilerCompilationHook,
  cache = false,
);
define_register!(
  register_compiler_make_taps,
  tap = CompilerMakeTap<JsCompilation, Promise<()>> @ CompilerMakeHook,
  cache = false,
);
define_register!(
  register_compilation_process_assets_taps,
  tap = CompilationProcessAssetsTap<JsCompilation, Promise<()>> @ CompilationProcessAssetsHook,
  cache = false,
);
define_register!(
  register_normal_module_factory_before_resolve_taps,
  tap = NormalModuleFactoryBeforeResolveTap<JsBeforeResolveArgs, Promise<JsBeforeResolveOutput>> @ NormalModuleFactoryBeforeResolveHook,
  cache = true,
);

#[async_trait]
impl AsyncSeries2<Compilation, CompilationParams> for CompilerCompilationTap {
  async fn run(
    &self,
    compilation: &mut Compilation,
    _: &mut CompilationParams,
  ) -> rspack_error::Result<()> {
    // SAFETY: `Compiler` will not be moved, as it's stored on the heap.
    // The pointer to `Compilation` is valid for the lifetime of `Compiler`.
    // `Compiler` is valid through the lifetime before it's closed by calling `Compiler.close()` or gc-ed.
    // `JsCompilation` is valid through the entire lifetime of `Compilation`.
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
    // SAFETY: `Compiler` will not be moved, as it's stored on the heap.
    // The pointer to `Compilation` is valid for the lifetime of `Compiler`.
    // `Compiler` is valid through the lifetime before it's closed by calling `Compiler.close()` or gc-ed.
    // `JsCompilation` is valid through the entire lifetime of `Compilation`.
    let compilation = unsafe { JsCompilation::from_compilation(compilation) };

    self.function.call_with_promise(compilation).await
  }

  fn stage(&self) -> i32 {
    self.stage
  }
}

#[async_trait]
impl AsyncSeries<Compilation> for CompilationProcessAssetsTap {
  async fn run(&self, compilation: &mut Compilation) -> rspack_error::Result<()> {
    // SAFETY: `Compiler` will not be moved, as it's stored on the heap.
    // The pointer to `Compilation` is valid for the lifetime of `Compiler`.
    // `Compiler` is valid through the lifetime before it's closed by calling `Compiler.close()` or gc-ed.
    // `JsCompilation` is valid through the entire lifetime of `Compilation`.
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
