use napi::bindgen_prelude::*;
use napi::{
  threadsafe_function::{ErrorStrategy, ThreadSafeResultContext, ThreadsafeFunction},
  Env,
};

use super::common::{
  REGISTERED_BUILD_END_SENDERS, REGISTERED_BUILD_START_SENDERS, REGISTERED_LOAD_SENDERS,
  REGISTERED_RESOLVE_SENDERS,
};
use crate::PluginCallbacks;

pub fn create_node_adapter_from_plugin_callbacks(
  env: &Env,
  plugin_callbacks: Option<PluginCallbacks>,
) -> Result<Option<super::RspackPluginNodeAdapter>> {
  plugin_callbacks
    .map(
      |PluginCallbacks {
         build_start_callback,
         load_callback,
         resolve_callback,
         build_end_callback,
       }| {
        let mut build_start_tsfn: ThreadsafeFunction<String, ErrorStrategy::CalleeHandled> =
          build_start_callback.create_threadsafe_function(
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
                      sender.send(()).map_err(|m| {
                        Error::new(
                          napi::Status::GenericFailure,
                          format!("failed to send result {:#?}", m),
                        )
                      })?;
                    } else {
                      return Err(Error::new(
                        napi::Status::GenericFailure,
                        "failed to get sender for plugin".to_owned(),
                      ));
                    }

                    Ok(())
                  },
                  |_, ret| Ok(ret),
                )
                .expect("failed to execute tokio future");
            },
          )?;

        let mut load_tsfn: ThreadsafeFunction<String, ErrorStrategy::CalleeHandled> = load_callback
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

                    let load_result: super::RspackThreadsafeResult<Option<super::OnLoadResult>> =
                      serde_json::from_str(&result).expect("failed to evaluate onload result");

                    tracing::debug!("onload result {:?}", load_result);

                    let sender = REGISTERED_LOAD_SENDERS.remove(&load_result.get_call_id());

                    if let Some((_, sender)) = sender {
                      sender.send(load_result.into_inner()).map_err(|m| {
                        Error::new(
                          napi::Status::GenericFailure,
                          format!("failed to send result {:#?}", m),
                        )
                      })?;
                    } else {
                      return Err(Error::new(
                        napi::Status::GenericFailure,
                        "failed to get sender for plugin".to_owned(),
                      ));
                    }

                    Ok(())
                  },
                  |_, ret| Ok(ret),
                )
                .expect("failed to execute tokio future");
            },
          )?;

        let mut resolve_tsfn: ThreadsafeFunction<String, ErrorStrategy::CalleeHandled> =
          resolve_callback.create_threadsafe_function(
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
                      sender.send(resolve_result.into_inner()).map_err(|m| {
                        Error::new(
                          napi::Status::GenericFailure,
                          format!("failed to send result {:#?}", m),
                        )
                      })?;
                    } else {
                      return Err(Error::new(
                        napi::Status::GenericFailure,
                        "failed to get sender for plugin".to_owned(),
                      ));
                    }

                    Ok(())
                  },
                  |_, ret| Ok(ret),
                )
                .expect("failed to execute tokio future");
            },
          )?;

        let mut build_end_tsfn: ThreadsafeFunction<String, ErrorStrategy::CalleeHandled> =
          build_end_callback.create_threadsafe_function(
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
                      sender.send(()).map_err(|m| {
                        Error::new(
                          napi::Status::GenericFailure,
                          format!("failed to send result {:#?}", m),
                        )
                      })?;
                    } else {
                      return Err(Error::new(
                        napi::Status::GenericFailure,
                        "failed to get sender for plugin".to_owned(),
                      ));
                    }

                    Ok(())
                  },
                  |_, ret| Ok(ret),
                )
                .expect("failed to execute tokio future");
            },
          )?;

        build_start_tsfn.unref(env)?;
        load_tsfn.unref(env)?;
        resolve_tsfn.unref(env)?;
        build_end_tsfn.unref(env)?;

        Ok(super::RspackPluginNodeAdapter {
          build_start_tsfn,
          load_tsfn,
          resolve_tsfn,
          build_end_tsfn,
        })
      },
    )
    .transpose()
}
