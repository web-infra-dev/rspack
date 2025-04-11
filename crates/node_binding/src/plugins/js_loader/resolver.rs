use std::{
  borrow::Cow,
  sync::{Arc, LazyLock},
};

use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  BoxLoader, Context, Loader, ModuleRuleUseLoader, NormalModuleFactoryResolveLoader, ResolveResult,
  Resolver, Resource, RunnerContext, BUILTIN_LOADER_PREFIX,
};
use rspack_error::{error, Result, SerdeResultToRspackResultExt, ToStringResultToRspackResultExt};
use rspack_hook::plugin_hook;
use rspack_loader_lightningcss::{config::Config, LIGHTNINGCSS_LOADER_IDENTIFIER};
use rspack_loader_preact_refresh::PREACT_REFRESH_LOADER_IDENTIFIER;
use rspack_loader_react_refresh::REACT_REFRESH_LOADER_IDENTIFIER;
use rspack_loader_swc::{SwcLoader, SWC_LOADER_IDENTIFIER};
use rspack_paths::Utf8Path;
use rustc_hash::FxHashMap;
use tokio::sync::RwLock;

use super::{JsLoaderRspackPlugin, JsLoaderRspackPluginInner};

#[cacheable]
#[derive(Debug)]
pub struct JsLoader(pub Identifier);

#[cacheable_dyn]
impl Loader<RunnerContext> for JsLoader {}

impl Identifiable for JsLoader {
  fn identifier(&self) -> Identifier {
    self.0
  }
}

type SwcLoaderCache<'a> = LazyLock<RwLock<FxHashMap<(Cow<'a, str>, Arc<str>), Arc<SwcLoader>>>>;
static SWC_LOADER_CACHE: SwcLoaderCache = LazyLock::new(|| RwLock::new(FxHashMap::default()));

pub async fn get_builtin_loader(builtin: &str, options: Option<&str>) -> Result<BoxLoader> {
  let options: Arc<str> = options.unwrap_or("{}").into();
  if builtin.starts_with(SWC_LOADER_IDENTIFIER) {
    if let Some(loader) = SWC_LOADER_CACHE
      .read()
      .await
      .get(&(Cow::Borrowed(builtin), options.clone()))
    {
      return Ok(loader.clone());
    }

    let loader = Arc::new(
      rspack_loader_swc::SwcLoader::new(options.as_ref())
        .to_rspack_result_with_detail(
          options.as_ref(),
          "failed to parse builtin:swc-loader options",
        )?
        .with_identifier(builtin.into()),
    );

    SWC_LOADER_CACHE.write().await.insert(
      (Cow::Owned(builtin.to_owned()), options.clone()),
      loader.clone(),
    );
    return Ok(loader);
  }

  if builtin.starts_with(LIGHTNINGCSS_LOADER_IDENTIFIER) {
    let config: rspack_loader_lightningcss::config::RawConfig =
      serde_json::from_str(options.as_ref()).to_rspack_result_with_detail(
        options.as_ref(),
        "Could not parse builtin:lightningcss-loader options",
      )?;
    // TODO: builtin-loader supports function
    return Ok(Arc::new(
      rspack_loader_lightningcss::LightningCssLoader::new(None, Config::try_from(config)?, builtin),
    ));
  }

  if builtin.starts_with(REACT_REFRESH_LOADER_IDENTIFIER) {
    return Ok(Arc::new(
      rspack_loader_react_refresh::ReactRefreshLoader::default().with_identifier(builtin.into()),
    ));
  }
  if builtin.starts_with(PREACT_REFRESH_LOADER_IDENTIFIER) {
    return Ok(Arc::new(
      rspack_loader_preact_refresh::PreactRefreshLoader::default().with_identifier(builtin.into()),
    ));
  }

  // TODO: should be compiled with a different cfg
  if builtin.starts_with(rspack_loader_testing::SIMPLE_ASYNC_LOADER_IDENTIFIER) {
    return Ok(Arc::new(rspack_loader_testing::SimpleAsyncLoader));
  }
  if builtin.starts_with(rspack_loader_testing::SIMPLE_LOADER_IDENTIFIER) {
    return Ok(Arc::new(rspack_loader_testing::SimpleLoader));
  }
  if builtin.starts_with(rspack_loader_testing::PITCHING_LOADER_IDENTIFIER) {
    return Ok(Arc::new(rspack_loader_testing::PitchingLoader));
  }
  if builtin.starts_with(rspack_loader_testing::PASS_THROUGH_LOADER_IDENTIFIER) {
    return Ok(Arc::new(rspack_loader_testing::PassthroughLoader));
  }
  if builtin.starts_with(rspack_loader_testing::NO_PASS_THROUGH_LOADER_IDENTIFIER) {
    return Ok(Arc::new(rspack_loader_testing::NoPassthroughLoader));
  }
  unreachable!("Unexpected builtin loader: {builtin}")
}

#[plugin_hook(NormalModuleFactoryResolveLoader for JsLoaderRspackPlugin)]
pub(crate) async fn resolve_loader(
  &self,
  context: &Context,
  resolver: &Resolver,
  l: &ModuleRuleUseLoader,
) -> Result<Option<BoxLoader>> {
  let context = context.as_path();
  let loader_request = &l.loader;
  let loader_options = l.options.as_deref();
  let mut rest = None;
  let prev = if let Some(index) = loader_request.find('?') {
    rest = Some(&loader_request[index..]);
    Utf8Path::new(&loader_request[0..index])
  } else {
    Utf8Path::new(loader_request)
  };

  // FIXME: not belong to napi
  if loader_request.starts_with(BUILTIN_LOADER_PREFIX) {
    return get_builtin_loader(loader_request, loader_options)
      .await
      .map(Some);
  }

  let resolve_result = resolver
    .resolve(context.as_std_path(), prev.as_str())
    .await
    .to_rspack_result_with_message(|e| {
      format!("Failed to resolve loader: {prev} in {context}, error: {e}")
    })?;

  match resolve_result {
    ResolveResult::Resource(resource) => {
      let Resource {
        path,
        query,
        description_data,
        ..
      } = resource;
      // Pitfall: `Path::ends_with` is different from `str::ends_with`
      // So we need to convert `PathBuf` to `&str`
      // Use `str::ends_with` instead of `Path::extension` to avoid unnecessary allocation
      let path = path.as_str();

      let r#type = if path.ends_with(".mjs") {
        Some("module")
      } else if path.ends_with(".cjs") {
        Some("commonjs")
      } else {
        description_data
          .as_ref()
          .and_then(|data| data.json().get("type").and_then(|t| t.as_str()))
      };
      // favor explicit loader query over aliased query, see webpack issue-3320
      let resource = if let Some(rest) = rest
        && !rest.is_empty()
      {
        format!("{path}{rest}")
      } else {
        format!("{path}{query}")
      };
      let ident = if let Some(ty) = r#type {
        format!("{ty}|{resource}")
      } else {
        resource
      };
      Ok(Some(Arc::new(JsLoader(ident.into()))))
    }
    ResolveResult::Ignored => Err(error!(
      "Failed to resolve loader: loader_request={prev}, context={context}"
    )),
  }
}
