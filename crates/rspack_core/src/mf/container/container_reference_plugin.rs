use async_trait::async_trait;

use super::{remote_module::RemoteModule, remote_runtime_module::RemoteRuntimeModule};
use crate::{
  AdditionalChunkRuntimeRequirementsArgs, ExternalType, FactorizeArgs, ModuleExt,
  ModuleFactoryResult, NormalModuleFactoryContext, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, PluginFactorizeHookOutput,
  RuntimeGlobals,
};

#[derive(Debug)]
pub struct ContainerReferencePluginOptions {
  pub remote_type: ExternalType,
  pub remotes: Vec<(String, RemoteOptions)>,
  pub share_scope: Option<String>,
}

#[derive(Debug)]
pub struct RemoteOptions {
  pub external: Vec<String>,
  pub share_scope: String,
}

#[derive(Debug)]
pub struct ContainerReferencePlugin {
  options: ContainerReferencePluginOptions,
}

impl ContainerReferencePlugin {
  pub fn new(options: ContainerReferencePluginOptions) -> Self {
    Self { options }
  }
}

#[async_trait]
impl Plugin for ContainerReferencePlugin {
  fn name(&self) -> &'static str {
    "rspack.ContainerReferencePlugin"
  }

  async fn factorize(
    &self,
    _ctx: PluginContext,
    args: FactorizeArgs<'_>,
    _job_ctx: &mut NormalModuleFactoryContext,
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
          )
          .boxed();
          return Ok(Some(ModuleFactoryResult::new(remote)));
        }
      }
    }
    Ok(None)
  }

  fn runtime_requirements_in_tree(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    if args
      .runtime_requirements
      .contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
    {
      args.runtime_requirements.insert(RuntimeGlobals::MODULE);
      args
        .runtime_requirements
        .insert(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY);
      args
        .runtime_requirements
        .insert(RuntimeGlobals::HAS_OWN_PROPERTY);
      args
        .runtime_requirements
        .insert(RuntimeGlobals::INITIALIZE_SHARING);
      args
        .runtime_requirements
        .insert(RuntimeGlobals::SHARE_SCOPE_MAP);
      args
        .compilation
        .add_runtime_module(args.chunk, Box::<RemoteRuntimeModule>::default());
    }
    Ok(())
  }
}
