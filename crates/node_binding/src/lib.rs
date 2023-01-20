#![recursion_limit = "256"]
#[macro_use]
extern crate napi_derive;

#[macro_use]
extern crate rspack_binding_macros;

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use napi::bindgen_prelude::*;

mod js_values;
mod plugins;
mod utils;

use js_values::*;
use plugins::*;
use rspack_binding_options::*;
use utils::*;

#[cfg(all(not(all(target_os = "linux", target_arch = "aarch64", target_env = "musl"))))]
#[global_allocator]
static ALLOC: mimalloc_rust::GlobalMiMalloc = mimalloc_rust::GlobalMiMalloc;

#[napi]
pub struct Rspack {
  compiler: rspack::Compiler,
  lock: Arc<AtomicBool>,
}

#[napi]
impl Rspack {
  #[napi(constructor)]
  pub fn new(env: Env, mut options: RawOptions, js_hooks: Option<JsHooks>) -> Result<Self> {
    init_custom_trace_subscriber(env)?;
    // rspack_tracing::enable_tracing_by_env();
    Self::prepare_environment(&env, &mut options);
    tracing::info!("raw_options: {:#?}", &options);

    let compiler_options = {
      let mut options =
        normalize_bundle_options(options).map_err(|e| Error::from_reason(format!("{e}")))?;

      if let Some(hooks_adapter) = js_hooks
        .map(|js_hooks| JsHooksAdapter::from_js_hooks(env, js_hooks))
        .transpose()?
      {
        options
          .plugins
          .push(Box::new(hooks_adapter) as Box<dyn rspack_core::Plugin>);
      };

      options
        .module
        .rules
        .iter_mut()
        .try_for_each(|rule| {
          rule.r#use.iter_mut().try_for_each(|loader| {
            let casted = loader.as_any_mut();
            if let Some(adapter) = casted.downcast_mut::<JsLoaderAdapter>() {
              adapter.unref(&env)
            } else {
              Ok(())
            }
          })
        })
        .map_err(|e| Error::from_reason(format!("failed to unref tsfn {e:?}")))?;

      options
    };

    tracing::info!("normalized_options: {:#?}", &compiler_options);

    let rspack = rspack::rspack(compiler_options, vec![]);

    Ok(Self {
      compiler: rspack,
      lock: Arc::new(AtomicBool::new(false)),
    })
  }

  /// Build with the given option passed to the constructor
  #[napi(catch_unwind, ts_args_type = "callback: (err: null | Error) => void")]
  pub fn build(&mut self, env: Env, f: JsFunction) -> Result<()> {
    if self
      .lock
      .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
      .is_err()
    {
      return callbackify::<(), _>(env, f, async {
        Err(Error::from_reason("ConcurrentCompilationError: You ran Rspack twice. Each instance only supports a single concurrent compilation at a time."))
      });
    }

    let compiler = unsafe {
      std::mem::transmute::<&'_ mut rspack::Compiler, &'static mut rspack::Compiler>(
        &mut self.compiler,
      )
    };

    callbackify(env, f, {
      let lock = self.lock.clone();
      async move {
        compiler
          .build()
          .await
          .map_err(|e| Error::new(napi::Status::GenericFailure, format!("{e}")))?;
        tracing::info!("build ok");
        lock.store(false, Ordering::Release);
        Ok(())
      }
    })
  }

  /// Rebuild with the given option passed to the constructor
  #[napi(
    catch_unwind,
    ts_args_type = "changed_files: string[], removed_files: string[], callback: (err: null | Error) => void"
  )]
  pub fn rebuild(
    &mut self,
    env: Env,
    changed_files: Vec<String>,
    removed_files: Vec<String>,
    f: JsFunction,
  ) -> Result<()> {
    if self
      .lock
      .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
      .is_err()
    {
      return callbackify::<(), _>(env, f, async {
        Err(Error::from_reason("ConcurrentCompilationError: You ran Rspack twice. Each instance only supports a single concurrent compilation at a time."))
      });
    }

    let compiler = unsafe {
      std::mem::transmute::<&'_ mut rspack::Compiler, &'static mut rspack::Compiler>(
        &mut self.compiler,
      )
    };

    callbackify(env, f, {
      let lock = self.lock.clone();
      async move {
        compiler
          .rebuild(
            HashSet::from_iter(changed_files.into_iter()),
            HashSet::from_iter(removed_files.into_iter()),
          )
          .await
          .map_err(|e| Error::new(napi::Status::GenericFailure, format!("{e:?}")))?;
        tracing::info!("rebuild ok");
        lock.store(false, Ordering::Release);
        Ok(())
      }
    })
  }

  /// Get the last compilation
  ///
  /// Warning:
  ///
  /// **Note** that this method is not safe if you cache the _JsCompilation_ on the Node side, as it will be invalidated by the next build and accessing a dangling ptr is a UB.
  #[napi(catch_unwind, js_name = "unsafe_last_compilation")]
  pub fn unsafe_last_compilation<F: Fn(JsCompilation) -> Result<()>>(
    &mut self,
    f: F,
  ) -> Result<()> {
    let compiler = unsafe {
      std::mem::transmute::<&'_ mut rspack::Compiler, &'static mut rspack::Compiler>(
        &mut self.compiler,
      )
    };

    f(JsCompilation::from_compilation(unsafe {
      Pin::new_unchecked(&mut compiler.compilation)
    }))
  }
}

impl Rspack {
  fn prepare_environment(env: &Env, options: &mut RawOptions) {
    NAPI_ENV.with(|napi_env| *napi_env.borrow_mut() = Some(env.raw()));

    if let Some(module) = options.module.as_mut() {
      for rule in &mut module.rules {
        if let Some(uses) = rule.r#use.as_mut() {
          for item in uses {
            if let Some(loader) = item.loader.as_ref() {
              // let (env_ptr, loader_ptr) = unsafe { (env.raw(), loader.raw()) };
              if let Ok(display_name) = get_named_property_value_string(*env, loader, "displayName")
              {
                item.__loader_name = Some(display_name);
              } else if let Ok(name) = get_named_property_value_string(*env, loader, "name") {
                item.__loader_name = Some(name);
              }
            }
          }
        }
      }
    }
  }
}

#[napi::module_init]
fn init() {
  use std::panic::set_hook;

  use backtrace::Backtrace;

  set_hook(Box::new(|panic_info| {
    let backtrace = Backtrace::new();
    println!("Panic: {panic_info:?}\nBacktrace: \n{backtrace:?}");
  }));
}
