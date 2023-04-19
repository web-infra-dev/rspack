use std::ops::Deref;

use napi_derive::napi;
#[cfg(feature = "node-api")]
use {
  napi::bindgen_prelude::*,
  napi::NapiRaw,
  rspack_binding_macros::call_js_function_with_napi_objects,
  rspack_core::{Loader, LoaderContext, LoaderRunnerContext},
  rspack_error::internal_error,
  rspack_identifier::{Identifiable, Identifier},
  rspack_napi_shared::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
  rspack_napi_shared::{NapiResultExt, NAPI_ENV},
};

#[napi(object)]
pub struct JsLoader {
  /// composed loader name, xx-loader$yy-loader$zz-loader
  pub identifier: String,
}

#[cfg(feature = "node-api")]
#[derive(Clone)]
pub struct JsLoaderRunner(ThreadsafeFunction<JsLoaderContext, LoaderThreadsafeLoaderResult>);

#[cfg(feature = "node-api")]
impl Deref for JsLoaderRunner {
  type Target = ThreadsafeFunction<JsLoaderContext, LoaderThreadsafeLoaderResult>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[cfg(feature = "node-api")]
impl TryFrom<JsFunction> for JsLoaderRunner {
  type Error = napi::Error;

  fn try_from(value: JsFunction) -> std::result::Result<Self, Self::Error> {
    let loader_runner = unsafe { value.raw() };

    let func = NAPI_ENV.with(|env| -> anyhow::Result<_> {
      let env = env
        .borrow()
        .expect("Failed to get env, did you forget to call it from node?");

      let mut func = ThreadsafeFunction::<JsLoaderContext, LoaderThreadsafeLoaderResult>::create(
        env,
        loader_runner,
        0,
        |ctx| {
          let (ctx, resolver) = ctx.split_into_parts();

          let env = ctx.env;
          let cb = ctx.callback;
          let resource = ctx.value.resource.clone();

          let result = tracing::span!(
            tracing::Level::INFO,
            "loader_sync_call",
            resource = &resource
          )
          .in_scope(|| unsafe { call_js_function_with_napi_objects!(env, cb, ctx.value) });

          let resolve_start = std::time::Instant::now();
          resolver.resolve::<Option<JsLoaderResult>>(result, move |_, r| {
            tracing::trace!(
              "Finish resolving loader result for {}, took {}ms",
              resource,
              resolve_start.elapsed().as_millis()
            );
            Ok(r)
          })
        },
      )?;
      func.unref(&Env::from(env))?;
      Ok(func)
    })?;

    Ok(JsLoaderRunner(func))
  }
}

#[cfg(feature = "node-api")]
pub struct JsLoaderAdapter {
  // pub func: ThreadsafeFunction<JsLoaderContext, LoaderThreadsafeLoaderResult>,
  pub runner: JsLoaderRunner,
  pub identifier: Identifier,
}

#[cfg(feature = "node-api")]
impl std::fmt::Debug for JsLoaderAdapter {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("JsLoaderAdapter")
      .field("loaders", &self.identifier)
      .finish()
  }
}

#[cfg(feature = "node-api")]
impl Identifiable for JsLoaderAdapter {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}

#[cfg(feature = "node-api")]
#[async_trait::async_trait]
impl Loader<LoaderRunnerContext> for JsLoaderAdapter {
  async fn pitch(
    &self,
    loader_context: &mut LoaderContext<'_, LoaderRunnerContext>,
  ) -> rspack_error::Result<()> {
    dbg!("pitching", loader_context.request().to_string());
    let mut js_loader_context: JsLoaderContext =
      <JsLoaderContext as AsyncTryFrom<_>>::try_from(loader_context).await?;
    js_loader_context.is_pitching = true;

    let loader_result = self
      .runner
      .call(js_loader_context, ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call loader: {err}"))??;

    if let Some(loader_result) = loader_result {
      sync_loader_context(loader_result, loader_context)?;
    }

    Ok(())
  }
  async fn run(
    &self,
    loader_context: &mut LoaderContext<'_, LoaderRunnerContext>,
  ) -> rspack_error::Result<()> {
    let mut js_loader_context: JsLoaderContext =
      <JsLoaderContext as AsyncTryFrom<_>>::try_from(loader_context).await?;
    js_loader_context.is_pitching = false;

    let loader_result = self
      .runner
      .call(js_loader_context, ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call loader: {err}"))??;

    if let Some(loader_result) = loader_result {
      sync_loader_context(loader_result, loader_context)?;
    }

    Ok(())
  }
}

#[cfg(feature = "node-api")]
fn sync_loader_context(
  loader_result: JsLoaderResult,
  loader_context: &mut LoaderContext<'_, LoaderRunnerContext>,
) -> rspack_error::Result<()> {
  loader_context.cacheable = loader_result.cacheable;
  loader_context.file_dependencies = loader_result
    .file_dependencies
    .into_iter()
    .map(std::path::PathBuf::from)
    .collect();
  loader_context.context_dependencies = loader_result
    .context_dependencies
    .into_iter()
    .map(std::path::PathBuf::from)
    .collect();
  loader_context.missing_dependencies = loader_result
    .missing_dependencies
    .into_iter()
    .map(std::path::PathBuf::from)
    .collect();
  loader_context.build_dependencies = loader_result
    .build_dependencies
    .into_iter()
    .map(std::path::PathBuf::from)
    .collect();
  loader_context.content = loader_result
    .content
    .map(|c| rspack_core::Content::from(Into::<Vec<u8>>::into(c)));
  loader_context.source_map = loader_result
    .source_map
    .as_ref()
    .map(|s| rspack_core::rspack_sources::SourceMap::from_slice(s))
    .transpose()
    .map_err(|e| internal_error!(e.to_string()))?;
  loader_context.additional_data = loader_result
    .additional_data
    .map(|item| String::from_utf8_lossy(&item).to_string());

  Ok(())
}

#[cfg(feature = "node-api")]
#[napi(object)]
pub struct JsLoaderContext {
  /// Content maybe empty in pitching stage
  pub content: Option<Buffer>,
  /// Original content will always be available for pitching stage
  pub original_content: Option<Buffer>,
  pub additional_data: Option<Buffer>,
  pub source_map: Option<Buffer>,
  pub resource: String,
  pub resource_path: String,
  pub resource_query: Option<String>,
  pub resource_fragment: Option<String>,
  pub cacheable: bool,
  pub file_dependencies: Vec<String>,
  pub context_dependencies: Vec<String>,
  pub missing_dependencies: Vec<String>,
  pub build_dependencies: Vec<String>,

  pub current_loader: String,
  pub is_pitching: bool,
}

trait AsyncTryFrom<T>: Sized {
  async fn try_from(value: T) -> rspack_error::Result<Self>;
}

#[cfg(feature = "node-api")]
impl AsyncTryFrom<&mut rspack_core::LoaderContext<'_, rspack_core::LoaderRunnerContext>>
  for JsLoaderContext
{
  async fn try_from(
    cx: &mut rspack_core::LoaderContext<'_, rspack_core::LoaderRunnerContext>,
  ) -> rspack_error::Result<Self> {
    // Fetch original content for pitching stage executing Js loaders in the normal stage.
    cx.fetch_original_content().await?;
    assert!(cx.original_content.is_some());

    Ok(JsLoaderContext {
      content: cx
        .content
        .as_ref()
        .map(|c| c.to_owned().into_bytes().into()),
      original_content: cx
        .original_content
        .as_ref()
        .map(|c| c.to_owned().into_bytes().into()),
      additional_data: cx.additional_data.to_owned().map(|v| v.into_bytes().into()),
      source_map: cx
        .source_map
        .clone()
        .map(|v| v.to_json())
        .transpose()
        .map_err(|e| internal_error!(e.to_string()))?
        .map(|v| v.into_bytes().into()),
      resource: cx.resource.to_owned(),
      resource_path: cx.resource_path.to_string_lossy().to_string(),
      resource_fragment: cx.resource_fragment.map(|r| r.to_owned()),
      resource_query: cx.resource_query.map(|r| r.to_owned()),
      cacheable: cx.cacheable,
      file_dependencies: cx
        .file_dependencies
        .iter()
        .map(|i| i.to_string_lossy().to_string())
        .collect(),
      context_dependencies: cx
        .context_dependencies
        .iter()
        .map(|i| i.to_string_lossy().to_string())
        .collect(),
      missing_dependencies: cx
        .missing_dependencies
        .iter()
        .map(|i| i.to_string_lossy().to_string())
        .collect(),
      build_dependencies: cx
        .build_dependencies
        .iter()
        .map(|i| i.to_string_lossy().to_string())
        .collect(),

      current_loader: cx.current_loader().to_string(),
      is_pitching: true,
    })
  }
}

// #[cfg(feature = "node-api")]
// impl TryFrom<&rspack_core::LoaderContext<'_, rspack_core::LoaderRunnerContext>>
//   for JsLoaderContext
// {
//   type Error = rspack_error::Error;

//   fn try_from(
//     cx: &rspack_core::LoaderContext<'_, rspack_core::LoaderRunnerContext>,
//   ) -> rspack_error::Result<Self> {
//     Ok(JsLoaderContext {
//       content: cx
//         .content
//         .as_ref()
//         .map(|c| c.to_owned().into_bytes().into()),
//       original_content: cx.original_content().await,
//       additional_data: cx.additional_data.to_owned().map(|v| v.into_bytes().into()),
//       source_map: cx
//         .source_map
//         .clone()
//         .map(|v| v.to_json())
//         .transpose()
//         .map_err(|e| internal_error!(e.to_string()))?
//         .map(|v| v.into_bytes().into()),
//       resource: cx.resource.to_owned(),
//       resource_path: cx.resource_path.to_string_lossy().to_string(),
//       resource_fragment: cx.resource_fragment.map(|r| r.to_owned()),
//       resource_query: cx.resource_query.map(|r| r.to_owned()),
//       cacheable: cx.cacheable,
//       file_dependencies: cx
//         .file_dependencies
//         .iter()
//         .map(|i| i.to_string_lossy().to_string())
//         .collect(),
//       context_dependencies: cx
//         .context_dependencies
//         .iter()
//         .map(|i| i.to_string_lossy().to_string())
//         .collect(),
//       missing_dependencies: cx
//         .missing_dependencies
//         .iter()
//         .map(|i| i.to_string_lossy().to_string())
//         .collect(),
//       build_dependencies: cx
//         .build_dependencies
//         .iter()
//         .map(|i| i.to_string_lossy().to_string())
//         .collect(),

//       current_loader: cx.current_loader().to_string(),
//       is_pitching: true,
//     })
//   }
// }

#[cfg(feature = "node-api")]
#[napi(object)]
pub struct JsLoaderResult {
  /// Content in pitching stage can be empty
  pub content: Option<Buffer>,
  pub file_dependencies: Vec<String>,
  pub context_dependencies: Vec<String>,
  pub missing_dependencies: Vec<String>,
  pub build_dependencies: Vec<String>,
  pub source_map: Option<Buffer>,
  pub additional_data: Option<Buffer>,
  pub cacheable: bool,
}

#[cfg(feature = "node-api")]
pub type LoaderThreadsafeLoaderResult = Option<JsLoaderResult>;
