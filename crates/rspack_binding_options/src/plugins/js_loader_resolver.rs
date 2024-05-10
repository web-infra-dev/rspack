use std::{fmt::Debug, path::Path, sync::Arc};

use rspack_core::{
  ApplyContext, BoxLoader, CompilerOptions, Context, ModuleRuleUseLoader, NormalModule,
  NormalModuleBeforeLoaders, NormalModuleFactoryResolveLoader, Plugin, PluginContext,
  ResolveResult, Resolver, BUILTIN_LOADER_PREFIX,
};
use rspack_error::{error, Result};
use rspack_hook::{plugin, plugin_hook};

use crate::{get_builtin_loader, JsLoaderAdapter, JsLoaderRunner};

#[plugin]
pub(crate) struct JsLoaderResolverPlugin {
  runner: JsLoaderRunner,
}

impl JsLoaderResolverPlugin {
  pub fn new(runner: JsLoaderRunner) -> Self {
    Self::new_inner(runner)
  }
}

impl Debug for JsLoaderResolverPlugin {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("JsLoaderResolver").finish()
  }
}

#[plugin_hook(NormalModuleBeforeLoaders for JsLoaderResolverPlugin)]
fn before_loaders(&self, module: &mut NormalModule) -> Result<()> {
  let contains_inline = module.contains_inline_loader();
  let contains_js_loader = module
    .loaders()
    .iter()
    .any(|l| !l.identifier().starts_with(BUILTIN_LOADER_PREFIX));

  // If there's any JS loader, then we switch to the JS loader runner.
  // Else, we run loader on the Rust side using the Rust loader runner.
  // Note: If the loaders list contains inline loaders,
  // fallback to JS loader runner for passing builtin options(reuse `Compiler.ruleSet`).
  if contains_inline || contains_js_loader {
    *module.loaders_mut_vec() = vec![Arc::new(JsLoaderAdapter {
      runner: self.runner.clone(),
      identifier: module
        .loaders()
        .iter()
        .map(|l| l.identifier().as_str())
        .collect::<Vec<_>>()
        .join("$")
        .into(),
    })];
  }

  Ok(())
}

#[plugin_hook(NormalModuleFactoryResolveLoader for JsLoaderResolverPlugin)]
async fn resolve_loader(
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
      // TODO: Should move this logic to `resolver`, since `resolve.alias` may contain query or fragment too.
      let resource = resource.path.to_string_lossy().to_string() + rest.unwrap_or_default();
      Ok(Some(Arc::new(JsLoaderAdapter {
        identifier: resource.into(),
        runner: self.runner.clone(),
      })))
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

#[async_trait::async_trait]
impl Plugin for JsLoaderResolverPlugin {
  fn name(&self) -> &'static str {
    "JsLoaderResolverPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .normal_module_hooks
      .before_loaders
      .tap(before_loaders::new(self));
    ctx
      .context
      .normal_module_factory_hooks
      .resolve_loader
      .tap(resolve_loader::new(self));
    Ok(())
  }
}
