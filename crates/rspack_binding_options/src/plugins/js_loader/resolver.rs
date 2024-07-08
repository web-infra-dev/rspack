use std::{path::Path, sync::Arc};

use rspack_core::{
  BoxLoader, Context, Loader, ModuleRuleUseLoader, NormalModuleFactoryResolveLoader, ResolveResult,
  Resolver, RunnerContext, BUILTIN_LOADER_PREFIX,
};
use rspack_error::{error, Result};
use rspack_hook::plugin_hook;
use rspack_identifier::{Identifiable, Identifier};
use rspack_loader_preact_refresh::PREACT_REFRESH_LOADER_IDENTIFIER;
use rspack_loader_react_refresh::REACT_REFRESH_LOADER_IDENTIFIER;
use rspack_loader_swc::SWC_LOADER_IDENTIFIER;

use super::{JsLoaderRspackPlugin, JsLoaderRspackPluginInner};

#[derive(Debug)]
pub struct JsLoader(pub Identifier);

impl Loader<RunnerContext> for JsLoader {}

impl Identifiable for JsLoader {
  fn identifier(&self) -> Identifier {
    self.0
  }
}

pub fn get_builtin_loader(builtin: &str, options: Option<&str>) -> BoxLoader {
  if builtin.starts_with(SWC_LOADER_IDENTIFIER) {
    return Arc::new(
      rspack_loader_swc::SwcLoader::new(
        serde_json::from_str(options.unwrap_or("{}")).unwrap_or_else(|e| {
          panic!("Could not parse builtin:swc-loader options:{options:?},error: {e:?}")
        }),
      )
      .with_identifier(builtin.into()),
    );
  }
  if builtin.starts_with(REACT_REFRESH_LOADER_IDENTIFIER) {
    return Arc::new(
      rspack_loader_react_refresh::ReactRefreshLoader::default().with_identifier(builtin.into()),
    );
  }
  if builtin.starts_with(PREACT_REFRESH_LOADER_IDENTIFIER) {
    return Arc::new(
      rspack_loader_preact_refresh::PreactRefreshLoader::default().with_identifier(builtin.into()),
    );
  }
  if builtin.starts_with(rspack_loader_testing::SIMPLE_ASYNC_LOADER_IDENTIFIER) {
    return Arc::new(rspack_loader_testing::SimpleAsyncLoader);
  }
  if builtin.starts_with(rspack_loader_testing::SIMPLE_LOADER_IDENTIFIER) {
    return Arc::new(rspack_loader_testing::SimpleLoader);
  }
  if builtin.starts_with(rspack_loader_testing::PITCHING_LOADER_IDENTIFIER) {
    return Arc::new(rspack_loader_testing::PitchingLoader);
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
  let context = context.as_ref();
  let loader_request = &l.loader;
  let loader_options = l.options.as_deref();
  let mut rest = None;
  let prev = if let Some(index) = loader_request.find('?') {
    rest = Some(&loader_request[index..]);
    Path::new(&loader_request[0..index])
  } else {
    Path::new(loader_request)
  };

  // FIXME: not belong to napi
  if loader_request.starts_with(BUILTIN_LOADER_PREFIX) {
    return Ok(Some(get_builtin_loader(loader_request, loader_options)));
  }

  let resolve_result = resolver
    .resolve(context, &prev.to_string_lossy())
    .map_err(|err| {
      let loader_request = prev.display();
      let context = context.display();
      error!("Failed to resolve loader: {loader_request} in {context} {err:?}")
    })?;

  match resolve_result {
    ResolveResult::Resource(resource) => {
      let path = resource.path.to_string_lossy().to_ascii_lowercase();
      let r#type = if path.ends_with(".mjs") {
        Some("module")
      } else if path.ends_with(".cjs") {
        Some("commonjs")
      } else {
        resource
          .description_data
          .as_ref()
          .and_then(|data| data.json().get("type").and_then(|t| t.as_str()))
      };
      // TODO: Should move this logic to `resolver`, since `resolve.alias` may contain query or fragment too.
      let resource = resource.path.to_string_lossy().to_string() + rest.unwrap_or_default();
      let ident = if let Some(ty) = r#type {
        format!("{ty}|{resource}")
      } else {
        resource
      };
      Ok(Some(Arc::new(JsLoader(ident.into()))))
    }
    ResolveResult::Ignored => {
      let loader_request = prev.display();
      let context = context.to_string_lossy();
      Err(error!(
        "Failed to resolve loader: loader_request={loader_request}, context={context}"
      ))
    }
  }
}
