use napi::bindgen_prelude::*;
use napi::{
  threadsafe_function::{ErrorStrategy, ThreadSafeResultContext, ThreadsafeFunction},
  Env,
};

use super::common::REGISTERED_DONE_SENDERS;
use crate::adapter::common::REGISTERED_PROCESS_ASSETS_SENDERS;
use crate::PluginCallbacks;

pub fn create_node_adapter_from_plugin_callbacks(
  env: &Env,
  plugin_callbacks: Option<PluginCallbacks>,
) -> Result<Option<super::RspackPluginNodeAdapter>> {
  plugin_callbacks
    .map(
      |PluginCallbacks {
         done_callback,
         process_assets_callback,
       }| {
        let mut done_tsfn: ThreadsafeFunction<String, ErrorStrategy::CalleeHandled> = done_callback
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
                      serde_json::from_str(&result).expect("failed to evaluate done result");

                    tracing::debug!("build end result {:?}", result);

                    let sender = REGISTERED_DONE_SENDERS.remove(&result.get_call_id());

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
        let mut process_assets_tsfn: ThreadsafeFunction<String, ErrorStrategy::CalleeHandled> =
          process_assets_callback.create_threadsafe_function(
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
                      serde_json::from_str(&result).expect("failed to evaluate done result");

                    tracing::debug!("build end result {:?}", result);

                    let sender = REGISTERED_PROCESS_ASSETS_SENDERS.remove(&result.get_call_id());

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
        done_tsfn.unref(env)?;
        process_assets_tsfn.unref(env)?;

        Ok(super::RspackPluginNodeAdapter {
          done_tsfn,
          process_assets_tsfn,
        })
      },
    )
    .transpose()
}
