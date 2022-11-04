use std::collections::HashMap;

use napi::{bindgen_prelude::*, Env, JsUnknown, NapiRaw};

use crate::{
  js_values::JsCompilation, threadsafe_function::*, JsCompatSource, PluginCallbacks,
  StatsCompilation,
};

pub fn create_node_adapter_from_plugin_callbacks(
  env: Env,
  plugin_callbacks: Option<PluginCallbacks>,
) -> Result<Option<super::RspackPluginNodeAdapter>> {
  plugin_callbacks
    .map(
      |PluginCallbacks {
         done_callback,
         process_assets_callback,
         this_compilation_callback,
         compilation_callback,
       }| {
        // *Note* that the order of the creation of threadsafe function is important. There is a queue of threadsafe calls for each tsfn:
        // For example:
        // tsfn1: [call-in-js-task1, call-in-js-task2]
        // tsfn2: [call-in-js-task3, call-in-js-task4]
        // If the tsfn1 is created before tsfn2, and task1 is created(via `tsfn.call`) before task2(single tsfn level),
        // and *if these tasks are created in the same tick*, tasks will be called on main thread in the order of `task1` `task2` `task3` `task4`
        //
        // In practice:
        // The creation of callback `this_compilation` is placed before the callback `compilation` because we want the JS hooks `this_compilation` to be called before the JS hooks `compilation`.

        let mut done_tsfn: ThreadsafeFunction<StatsCompilation, ()> = {
          let cb = unsafe { done_callback.raw() };

          ThreadsafeFunction::create(env.raw(), cb, 0, |ctx| {
            let (ctx, resolver) = ctx.split_into_parts();

            let env = ctx.env;
            let cb = ctx.callback;
            let result = unsafe { call_js_function_with_napi_objects!(env, cb, ctx.value) }?;

            resolver.resolve::<()>(result, |_| Ok(()))
          })
        }?;

        let mut process_assets_tsfn: ThreadsafeFunction<HashMap<String, JsCompatSource>, ()> = {
          let cb = unsafe { process_assets_callback.raw() };

          ThreadsafeFunction::create(env.raw(), cb, 0, |ctx| {
            let (ctx, resolver) = ctx.split_into_parts();

            let env = ctx.env;
            let cb = ctx.callback;
            let result = unsafe { call_js_function_with_napi_objects!(env, cb, ctx.value) }?;

            resolver.resolve::<()>(result, |_| Ok(()))
          })
        }?;

        let mut this_compilation_tsfn: ThreadsafeFunction<JsCompilation, ()> = {
          let cb = unsafe { this_compilation_callback.raw() };

          ThreadsafeFunction::create(env.raw(), cb, 0, |ctx| {
            let ThreadSafeContext {
              env,
              value,
              callback,
              ..
            } = ctx;

            let value = unsafe {
              JsUnknown::from_napi_value(
                env.raw(),
                JsCompilation::to_napi_value(env.raw(), value)?,
              )?
            };

            callback.call(None, &[value])?;

            Ok(())
          })
        }?;

        let mut compilation_tsfn: ThreadsafeFunction<JsCompilation, ()> = {
          let cb = unsafe { compilation_callback.raw() };

          ThreadsafeFunction::create(env.raw(), cb, 0, |ctx| {
            let ThreadSafeContext {
              env,
              value,
              callback,
              ..
            } = ctx;

            let value = unsafe {
              JsUnknown::from_napi_value(
                env.raw(),
                JsCompilation::to_napi_value(env.raw(), value)?,
              )?
            };

            callback.call(None, &[value])?;

            Ok(())
          })
        }?;

        // See the comment in `threadsafe_function.rs`
        done_tsfn.unref(&env)?;
        process_assets_tsfn.unref(&env)?;
        compilation_tsfn.unref(&env)?;
        this_compilation_tsfn.unref(&env)?;

        Ok(super::RspackPluginNodeAdapter {
          done_tsfn,
          process_assets_tsfn,
          compilation_tsfn,
          this_compilation_tsfn,
        })
      },
    )
    .transpose()
}
