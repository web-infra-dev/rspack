use napi::{bindgen_prelude::*, Env, JsObject, NapiRaw, NapiValue};

use crate::{js_values::StatsCompilation, BoxedClosure, PluginCallbacks};

pub fn create_node_adapter_from_plugin_callbacks(
  env: Env,
  plugin_callbacks: Option<PluginCallbacks>,
) -> Result<Option<super::RspackPluginNodeAdapter>> {
  plugin_callbacks
    .map(
      |PluginCallbacks {
         done_callback,
         process_assets_callback,
       }| {
        let done_tsfn: crate::threadsafe_function::ThreadsafeFunction<StatsCompilation, ()> = {
          let cb = unsafe { done_callback.raw() };

          crate::threadsafe_function::ThreadsafeFunction::create(env.raw(), cb, 0, |ctx| {
            let (ctx, resolver) = ctx.split_into_parts();
            let stats_json = unsafe {
              let raw =
                napi::bindgen_prelude::ToNapiValue::to_napi_value(ctx.env.raw(), ctx.value)?;
              JsObject::from_raw_unchecked(ctx.env.raw(), raw)
            };
            let result = ctx.callback.call(None, &[stats_json])?;

            resolver.resolve::<()>(result, |_| Ok(()))
          })
        }?;

        let process_assets_tsfn: crate::threadsafe_function::ThreadsafeFunction<
          (String, BoxedClosure),
          (),
        > = {
          let cb = unsafe { process_assets_callback.raw() };

          crate::threadsafe_function::ThreadsafeFunction::create(env.raw(), cb, 0, |ctx| {
            let (ctx, resolver) = ctx.split_into_parts();

            let (s, emit_asset_cb) = ctx.value;
            let result = ctx.callback.call(
              None,
              &[
                ctx.env.create_string_from_std(s)?.into_unknown(),
                ctx
                  .env
                  .create_function_from_closure("compilation_emit_asset", emit_asset_cb)?
                  .into_unknown(),
              ],
            )?;

            resolver.resolve::<()>(result, |_| Ok(()))
          })
        }?;

        // See the comment in `threadsafe_function.rs`
        done_tsfn.unref(&env)?;
        process_assets_tsfn.unref(&env)?;

        Ok(super::RspackPluginNodeAdapter {
          done_tsfn,
          process_assets_tsfn,
        })
      },
    )
    .transpose()
}
