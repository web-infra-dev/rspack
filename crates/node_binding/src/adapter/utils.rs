use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadSafeCallContext;
use napi::{
  threadsafe_function::{ErrorStrategy, ThreadSafeResultContext, ThreadsafeFunction},
  Env, JsExternal, JsString, JsUnknown, NapiRaw,
};

use super::common::{
  REGISTERED_BUILD_END_SENDERS, REGISTERED_BUILD_START_SENDERS, REGISTERED_LOAD_SENDERS,
  REGISTERED_RESOLVE_SENDERS,
};
use super::OnLoadResult;
use crate::PluginCallbacks;

pub fn create_node_adapter_from_plugin_callbacks(
  env: &Env,
  plugin_callbacks: Option<PluginCallbacks>,
) -> Option<super::RspackPluginNodeAdapter> {
  plugin_callbacks.map(
    |PluginCallbacks {
       build_start_callback,
       load_callback,
       resolve_callback,
       build_end_callback,
     }| {
      let mut build_start_tsfn: ThreadsafeFunction<String, ErrorStrategy::CalleeHandled> =
        build_start_callback
          .create_threadsafe_function(
            0,
            |ctx| ctx.env.create_string_from_std(ctx.value).map(|v| vec![v]),
            |ctx: ThreadSafeResultContext<Promise<String>>| {
              let return_value = ctx.return_value;

              ctx
                .env
                .execute_tokio_future(
                  async move {
                    let result = return_value.await?;

                    let result: super::RspackThreadsafeResult<()> =
                      serde_json::from_str(&result).expect("failed to evaluate build_start result");

                    tracing::debug!("build start result {:?}", result);

                    let sender = REGISTERED_BUILD_START_SENDERS.remove(&result.get_call_id());

                    if let Some((_, sender)) = sender {
                      sender.send(()).expect("unable to send");
                    } else {
                      panic!("unable to send");
                    }

                    Ok(())
                  },
                  |_, ret| Ok(ret),
                )
                .expect("failed to execute tokio future");
            },
          )
          .unwrap();

      let mut load_tsfn: ThreadsafeFunction<
        (String, tokio::sync::oneshot::Sender<Option<OnLoadResult>>),
        ErrorStrategy::CalleeHandled,
      > = load_callback
        .create_threadsafe_function(
          0,
          |ctx: ThreadSafeCallContext<(
            String,
            tokio::sync::oneshot::Sender<Option<OnLoadResult>>,
          )>| {
            let value = ctx.value.0;
            let sender = ctx.value.1;
            Ok(vec![
              ctx.env.create_string_from_std(value)?.into_unknown(),
              ctx.env.create_external(Some(sender), None)?.into_unknown(),
            ])
          },
          |ctx: ThreadSafeResultContext<Vec<JsUnknown>>| {
            let mut return_value = ctx.return_value;

            let sender = unsafe { return_value.remove(1).cast::<JsExternal>() };
            let sender = ctx
              .env
              .get_value_external::<Option<tokio::sync::oneshot::Sender<Option<OnLoadResult>>>>(
                &sender,
              )
              .expect("failed to get external value")
              .take();

            let raw_promise = return_value.remove(0);

            let load_promise = unsafe {
              Promise::from_napi_value(ctx.env.raw(), raw_promise.raw())
                .expect("expect return value to be a promise") as Promise<String>
            };

            ctx
              .env
              .execute_tokio_future(
                async move {
                  let load_result = load_promise.await?;

                  let load_result: super::RspackThreadsafeResult<Option<super::OnLoadResult>> =
                    serde_json::from_str(&load_result).expect("failed to evaluate onload result");

                  // tracing::debug!("onload result {:?}", load_result);

                  // // let sender = REGISTERED_LOAD_SENDERS.remove(&load_result.get_call_id());

                  if let Some(sender) = sender {
                    sender.send(load_result.into_inner()).map_err(|e| {
                      Error::new(
                        napi::Status::GenericFailure,
                        format!("unable to send result {:#?}", e),
                      )
                    })?;
                  }

                  Ok(())
                },
                |_, ret| Ok(ret),
              )
              .expect("failed to execute tokio future");
          },
        )
        .unwrap();

      let mut resolve_tsfn: ThreadsafeFunction<String, ErrorStrategy::CalleeHandled> =
        resolve_callback
          .create_threadsafe_function(
            0,
            |ctx| ctx.env.create_string_from_std(ctx.value).map(|v| vec![v]),
            |ctx: ThreadSafeResultContext<Promise<String>>| {
              let return_value = ctx.return_value;

              ctx
                .env
                .execute_tokio_future(
                  async move {
                    let result = return_value.await?;

                    let resolve_result: super::RspackThreadsafeResult<
                      Option<super::OnResolveResult>,
                    > = serde_json::from_str(&result).expect("failed to evaluate onresolve result");

                    tracing::debug!("[rspack:binding] onresolve result {:?}", resolve_result);

                    let sender = REGISTERED_RESOLVE_SENDERS.remove(&resolve_result.get_call_id());

                    if let Some((_, sender)) = sender {
                      sender
                        .send(resolve_result.into_inner())
                        .expect("unable to send");
                    } else {
                      panic!("unable to send");
                    }

                    Ok(())
                  },
                  |_, ret| Ok(ret),
                )
                .expect("failed to execute tokio future");
            },
          )
          .unwrap();

      let mut build_end_tsfn: ThreadsafeFunction<String, ErrorStrategy::CalleeHandled> =
        build_end_callback
          .create_threadsafe_function(
            0,
            |ctx| ctx.env.create_string_from_std(ctx.value).map(|v| vec![v]),
            |ctx: ThreadSafeResultContext<Promise<String>>| {
              let return_value = ctx.return_value;

              ctx
                .env
                .execute_tokio_future(
                  async move {
                    let result = return_value.await?;

                    let result: super::RspackThreadsafeResult<()> =
                      serde_json::from_str(&result).expect("failed to evaluate build_end result");

                    tracing::debug!("build end result {:?}", result);

                    let sender = REGISTERED_BUILD_END_SENDERS.remove(&result.get_call_id());

                    if let Some((_, sender)) = sender {
                      sender.send(()).expect("unable to send");
                    } else {
                      panic!("unable to send");
                    }

                    Ok(())
                  },
                  |_, ret| Ok(ret),
                )
                .expect("failed to execute tokio future");
            },
          )
          .unwrap();

      build_start_tsfn.unref(env).unwrap();
      load_tsfn.unref(env).unwrap();
      resolve_tsfn.unref(env).unwrap();
      build_end_tsfn.unref(env).unwrap();

      super::RspackPluginNodeAdapter {
        build_start_tsfn,
        load_tsfn,
        resolve_tsfn,
        build_end_tsfn,
      }
    },
  )
}
