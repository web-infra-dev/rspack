use napi::bindgen_prelude::*;
use napi_derive::napi;
#[cfg(feature = "node-api")]
use {
  napi::NapiRaw,
  rspack_binding_macros::call_js_function_with_napi_objects,
  rspack_error::internal_error,
  rspack_identifier::{Identifiable, Identifier},
  rspack_napi_shared::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
  rspack_napi_shared::{NapiResultExt, NAPI_ENV},
};

#[napi(object)]
pub struct JsLoader {
  /// composed loader name, xx-loader!yy-loader!zz-loader
  pub name: String,
  pub func: JsFunction,
}

#[cfg(feature = "node-api")]
pub struct JsLoaderAdapter {
  pub func: ThreadsafeFunction<JsLoaderContext, LoaderThreadsafeLoaderResult>,
  pub name: Identifier,
}

#[cfg(feature = "node-api")]
impl TryFrom<JsLoader> for JsLoaderAdapter {
  type Error = anyhow::Error;
  fn try_from(js_loader: JsLoader) -> anyhow::Result<Self> {
    let js_loader_func = unsafe { js_loader.func.raw() };

    let func = NAPI_ENV.with(|env| -> anyhow::Result<_> {
      let env = env
        .borrow()
        .expect("Failed to get env, did you forget to call it from node?");
      let mut func = ThreadsafeFunction::<JsLoaderContext, LoaderThreadsafeLoaderResult>::create(
        env,
        js_loader_func,
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
    Ok(JsLoaderAdapter {
      func,
      name: js_loader.name.into(),
    })
  }
}

#[cfg(feature = "node-api")]
impl std::fmt::Debug for JsLoaderAdapter {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("JsLoaderAdapter")
      .field("loaders", &self.name)
      .finish()
  }
}

#[cfg(feature = "node-api")]
impl Identifiable for JsLoaderAdapter {
  fn identifier(&self) -> Identifier {
    self.name
  }
}

#[cfg(feature = "node-api")]
#[async_trait::async_trait]
impl rspack_core::Loader<rspack_core::LoaderRunnerContext> for JsLoaderAdapter {
  async fn run(
    &self,
    loader_context: &mut rspack_core::LoaderContext<'_, rspack_core::LoaderRunnerContext>,
  ) -> rspack_error::Result<()> {
    let js_loader_context = (&*loader_context).try_into()?;

    let loader_result = self
      .func
      .call(js_loader_context, ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call loader: {err}"))??;

    let source_map = loader_result
      .as_ref()
      .and_then(|r| r.source_map.as_ref())
      .map(|s| rspack_core::rspack_sources::SourceMap::from_slice(s))
      .transpose()
      .map_err(|e| internal_error!(e.to_string()))?;

    if let Some(loader_result) = loader_result {
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
      loader_context.content = Some(rspack_core::Content::from(Into::<Vec<u8>>::into(
        loader_result.content,
      )));
      loader_context.source_map = source_map;
      loader_context.additional_data = loader_result
        .additional_data
        .map(|item| String::from_utf8_lossy(&item).to_string());
    }

    Ok(())
  }
}

#[cfg(feature = "node-api")]
#[napi(object)]
pub struct JsLoaderContext {
  /// Content maybe empty in pitching stage
  pub content: Option<Buffer>,
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
}

#[cfg(feature = "node-api")]
impl<'c> TryFrom<&rspack_core::LoaderContext<'c, rspack_core::LoaderRunnerContext>>
  for JsLoaderContext
{
  type Error = rspack_error::Error;

  fn try_from(
    cx: &rspack_core::LoaderContext<'c, rspack_core::LoaderRunnerContext>,
  ) -> rspack_error::Result<Self> {
    Ok(JsLoaderContext {
      content: cx
        .content
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
    })
  }
}

#[cfg(feature = "node-api")]
#[napi(object)]
pub struct JsLoaderResult {
  pub content: Buffer,
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
