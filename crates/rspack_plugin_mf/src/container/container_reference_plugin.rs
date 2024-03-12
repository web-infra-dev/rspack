use std::sync::Arc;

use async_trait::async_trait;
use rspack_core::{
  ApplyContext, Compilation, CompilationParams, CompilerOptions, DependencyType, ExternalType,
  FactorizeArgs, ModuleExt, ModuleFactoryResult, Plugin, PluginContext, PluginFactorizeHookOutput,
  PluginRuntimeRequirementsInTreeOutput, RuntimeGlobals, RuntimeRequirementsInTreeArgs,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook, AsyncSeries2};

use super::{
  fallback_module_factory::FallbackModuleFactory, remote_module::RemoteModule,
  remote_runtime_module::RemoteRuntimeModule,
};

#[derive(Debug)]
pub struct ContainerReferencePluginOptions {
  pub remote_type: ExternalType,
  pub remotes: Vec<(String, RemoteOptions)>,
  pub share_scope: Option<String>,
  pub enhanced: bool,
}

#[derive(Debug)]
pub struct RemoteOptions {
  pub external: Vec<String>,
  pub share_scope: String,
}

#[plugin]
#[derive(Debug)]
pub struct ContainerReferencePlugin {
  options: ContainerReferencePluginOptions,
}

impl ContainerReferencePlugin {
  pub fn new(options: ContainerReferencePluginOptions) -> Self {
    Self::new_inner(options)
  }
}

#[plugin_hook(AsyncSeries2<Compilation, CompilationParams> for ContainerReferencePlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(
    DependencyType::RemoteToExternal,
    params.normal_module_factory.clone(),
  );
  compilation.set_dependency_factory(
    DependencyType::RemoteToFallbackItem,
    params.normal_module_factory.clone(),
  );
  compilation.set_dependency_factory(
    DependencyType::RemoteToFallback,
    Arc::new(FallbackModuleFactory),
  );
  Ok(())
}

#[async_trait]
impl Plugin for ContainerReferencePlugin {
  fn name(&self) -> &'static str {
    "rspack.ContainerReferencePlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));
    Ok(())
  }

  async fn factorize(
    &self,
    _ctx: PluginContext,
    args: &mut FactorizeArgs<'_>,
  ) -> PluginFactorizeHookOutput {
    let request = args.dependency.request();
    if !request.contains('!') {
      for (key, config) in &self.options.remotes {
        let key_len = key.len();
        if request.starts_with(key)
          && (request.len() == key_len || request[key_len..].starts_with('/'))
        {
          let internal_request = &request[key_len..];
          let remote = RemoteModule::new(
            request.to_owned(),
            config
              .external
              .iter()
              .enumerate()
              .map(|(i, e)| {
                if let Some(stripped) = e.strip_prefix("internal ") {
                  stripped.to_string()
                } else {
                  format!(
                    "webpack/container/reference/{}{}",
                    key,
                    (i > 0)
                      .then(|| format!("/fallback-{}", i))
                      .unwrap_or_default()
                  )
                }
              })
              .collect(),
            format!(".{}", internal_request),
            config.share_scope.clone(),
            key.to_string(),
          )
          .boxed();
          return Ok(Some(ModuleFactoryResult::new_with_module(remote)));
        }
      }
    }
    Ok(None)
  }

  async fn runtime_requirements_in_tree(
    &self,
    _ctx: PluginContext,
    args: &mut RuntimeRequirementsInTreeArgs,
  ) -> PluginRuntimeRequirementsInTreeOutput {
    if args
      .runtime_requirements
      .contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
    {
      args.runtime_requirements_mut.insert(RuntimeGlobals::MODULE);
      args
        .runtime_requirements_mut
        .insert(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY);
      args
        .runtime_requirements_mut
        .insert(RuntimeGlobals::HAS_OWN_PROPERTY);
      args
        .runtime_requirements_mut
        .insert(RuntimeGlobals::INITIALIZE_SHARING);
      args
        .runtime_requirements_mut
        .insert(RuntimeGlobals::SHARE_SCOPE_MAP);
      args
        .compilation
        .add_runtime_module(
          args.chunk,
          Box::new(RemoteRuntimeModule::new(self.options.enhanced)),
        )
        .await?;
    }
    Ok(())
  }
}
