use napi::Env;
use napi::{bindgen_prelude::*, NapiRaw};

use super::BoxedClosure;
use crate::PluginCallbacks;

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
        let done_tsfn: crate::threadsafe_function::ThreadsafeFunction<(), ()> = {
          let cb = unsafe { done_callback.raw() };

          crate::threadsafe_function::ThreadsafeFunction::create(env.raw(), cb, 0, |ctx| {
            let result = ctx.callback.call_without_args(None)?;

            // TODO: use deferred value

            assert!(result.is_promise()?);

            // TODO: enable feature anyhow
            let promise = unsafe { Promise::<()>::from_napi_value(ctx.env.raw(), result.raw()) }?;

            ctx.env.execute_tokio_future(
              async move {
                let _ = promise.await?;
                ctx.tx.send(()).map_err(|err| {
                  napi::Error::from_reason(format!("Failed to send result: {:?}", err))
                })
              },
              |_, res| Ok(res),
            )?;

            Ok(())
          })
        }?;

        let mut process_assets_tsfn: crate::threadsafe_function::ThreadsafeFunction<
          (String, BoxedClosure),
          (),
        > = {
          let cb = unsafe { process_assets_callback.raw() };

          crate::threadsafe_function::ThreadsafeFunction::create(env.raw(), cb, 0, |ctx| {
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

            assert!(result.is_promise()?);

            // TODO: use deferred value

            // TODO: enable feature anyhow
            let promise = unsafe { Promise::<()>::from_napi_value(ctx.env.raw(), result.raw()) }?;

            ctx.env.execute_tokio_future(
              async move {
                let _ = promise.await?;
                ctx.tx.send(()).map_err(|err| {
                  napi::Error::from_reason(format!("Failed to send result: {:?}", err))
                })
              },
              |_, res| Ok(res),
            )?;

            Ok(())
          })
        }?;

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
