use std::sync::Arc;

use rspack_core::{
  BoxModule, ChunkUkey, Compilation, CompilationParams, CompilationRuntimeRequirementInTree,
  CompilerCompilation, DependencyType, ExternalType, ModuleExt, ModuleFactoryCreateData,
  NormalModuleFactoryFactorize, Plugin, RuntimeGlobals, RuntimeModule,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::itoa;

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
  let request = &data.request;
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
                let fallback_suffix = if i > 0 {
                  let mut i_buffer = itoa::Buffer::new();
                  let i_str = i_buffer.format(i);
                  format!("/fallback-{i_str}")
                } else {
                  Default::default()
                };
                format!("webpack/container/reference/{key}{fallback_suffix}")
              }
            })
            .collect(),
          format!(".{internal_request}"),
          config.share_scope.clone(),
          key.clone(),
        )
        .boxed();
        return Ok(Some(remote));
      }
    }
  }
  Ok(None)
}

#[plugin_hook(CompilationRuntimeRequirementInTree for ContainerReferencePlugin)]
async fn runtime_requirements_in_tree(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  _all_runtime_requirements: &RuntimeGlobals,
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
  runtime_modules_to_add: &mut Vec<(ChunkUkey, Box<dyn RuntimeModule>)>,
) -> Result<Option<()>> {
  if runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS) {
    runtime_requirements_mut.insert(RuntimeGlobals::MODULE);
    runtime_requirements_mut.insert(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY);
    runtime_requirements_mut.insert(RuntimeGlobals::HAS_OWN_PROPERTY);
    runtime_requirements_mut.insert(RuntimeGlobals::INITIALIZE_SHARING);
    runtime_requirements_mut.insert(RuntimeGlobals::SHARE_SCOPE_MAP);
    runtime_modules_to_add.push((
      *chunk_ukey,
      Box::new(RemoteRuntimeModule::new(
        &compilation.runtime_template,
        self.options.enhanced,
      )),
    ));
  }
  Ok(None)
}

impl Plugin for ContainerReferencePlugin {
  fn name(&self) -> &'static str {
    "rspack.ContainerReferencePlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .normal_module_factory_hooks
      .factorize
      .tap(factorize::new(self));
    ctx
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirements_in_tree::new(self));
    Ok(())
  }
}
