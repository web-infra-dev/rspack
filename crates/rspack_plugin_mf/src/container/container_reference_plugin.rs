use std::sync::Arc;

use async_trait::async_trait;
use rspack_core::{
  ApplyContext, BoxModule, ChunkUkey, Compilation, CompilationParams,
  CompilationRuntimeRequirementInTree, CompilerCompilation, CompilerOptions, DependencyType,
  ExternalType, ModuleExt, ModuleFactoryCreateData, NormalModuleFactoryFactorize, Plugin,
  PluginContext, RuntimeGlobals,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use super::{
  fallback_module_factory::FallbackModuleFactory, federation_runtime::FederationRuntimePlugin,
  federation_runtime::FederationRuntimePluginOptions, remote_module::RemoteModule,
  remote_runtime_module::RemoteRuntimeModule,
};

#[derive(Debug)]
pub struct ContainerReferencePluginOptions {
  pub remote_type: ExternalType,
  pub remotes: Vec<(String, RemoteOptions)>,
  pub share_scope: Option<String>,
  pub enhanced: bool,
}

#[derive(Debug, Clone)]
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

#[plugin_hook(CompilerCompilation for ContainerReferencePlugin)]
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

#[plugin_hook(NormalModuleFactoryFactorize for ContainerReferencePlugin)]
async fn factorize(&self, data: &mut ModuleFactoryCreateData) -> Result<Option<BoxModule>> {
  let dependency = data
    .dependency
    .as_module_dependency()
    .expect("should be module dependency");
  let request = dependency.request();
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
        return Ok(Some(remote));
      }
    }
  }
  Ok(None)
}

#[plugin_hook(CompilationRuntimeRequirementInTree for ContainerReferencePlugin)]
fn runtime_requirements_in_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
) -> Result<Option<()>> {
  if runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS) {
    runtime_requirements_mut.insert(RuntimeGlobals::MODULE);
    runtime_requirements_mut.insert(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY);
    runtime_requirements_mut.insert(RuntimeGlobals::HAS_OWN_PROPERTY);
    runtime_requirements_mut.insert(RuntimeGlobals::INITIALIZE_SHARING);
    runtime_requirements_mut.insert(RuntimeGlobals::SHARE_SCOPE_MAP);
    compilation.add_runtime_module(
      chunk_ukey,
      Box::new(RemoteRuntimeModule::new(self.options.enhanced)),
    )?;
  }
  Ok(None)
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
    ctx
      .context
      .normal_module_factory_hooks
      .factorize
      .tap(factorize::new(self));
    ctx
      .context
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirements_in_tree::new(self));

    let federation_options = federation_runtime::FederationRuntimePluginOptions {
      remote_type: self.options.remote_type.clone(),
      remotes: self.options.remotes.clone(),
      share_scope: self.options.share_scope.clone(),
      enhanced: self.options.enhanced,
    };

    federation_runtime::apply(ctx, federation_options)?;

    Ok(())
  }
}
