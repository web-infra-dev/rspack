use rspack_core::{
  ApplyContext, BoxModule, Compilation, CompilationParams, CompilerCompilation, CompilerOptions,
  Context, DependencyType, LibIdentOptions, ModuleFactoryCreateData, NormalModuleCreateData,
  NormalModuleFactoryFactorize, NormalModuleFactoryModule, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use super::delegated_module::DelegatedModule;
use crate::DllManifestContent;

#[derive(Debug)]
pub struct DelegatedPluginOptions {
  pub source: String,

  pub context: Option<Context>,

  pub content: DllManifestContent,

  pub r#type: String,

  pub extensions: Vec<String>,

  pub scope: Option<String>,

  pub compilation_context: Context,
}

#[plugin]
#[derive(Debug)]
pub struct DelegatedPlugin {
  options: DelegatedPluginOptions,
}

impl DelegatedPlugin {
  pub fn new(options: DelegatedPluginOptions) -> Self {
    Self::new_inner(options)
  }
}

impl Plugin for DelegatedPlugin {
  fn name(&self) -> &'static str {
    "rspack.DelegatedPlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
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
      .normal_module_factory_hooks
      .module
      .tap(nmf_module::new(self));

    Ok(())
  }
}

#[plugin_hook(CompilerCompilation for DelegatedPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(
    DependencyType::DelegatedSource,
    params.normal_module_factory.clone(),
  );
  Ok(())
}

#[plugin_hook(NormalModuleFactoryFactorize for DelegatedPlugin)]
async fn factorize(&self, data: &mut ModuleFactoryCreateData) -> Result<Option<BoxModule>> {
  if let Some(scope) = &self.options.scope {
    if let Some(dependency) = data.dependencies[0].as_module_dependency() {
      let scope_prefix = format!("{scope}/");
      let request = dependency.request();
      if request.starts_with(&scope_prefix) {
        let inner_request = format!(
          ".{}",
          &request.chars().skip(scope.len()).collect::<String>()
        );

        if let Some(resolved) = self.options.content.get(&inner_request) {
          return Ok(Some(Box::new(DelegatedModule::new(
            self.options.source.clone(),
            resolved.clone(),
            self.options.r#type.clone(),
            inner_request,
            Some(request.to_owned()),
          ))));
        }

        for extension in self.options.extensions.iter() {
          let request_plus_ext = format!("{inner_request}{extension}");

          if let Some(resolved) = self.options.content.get(&request_plus_ext) {
            return Ok(Some(Box::new(DelegatedModule::new(
              self.options.source.clone(),
              resolved.clone(),
              self.options.r#type.clone(),
              request_plus_ext,
              format!("{request}{extension}").into(),
            ))));
          }
        }
      }
    }
  }

  Ok(None)
}

#[plugin_hook(NormalModuleFactoryModule for DelegatedPlugin)]
async fn nmf_module(
  &self,
  _data: &mut ModuleFactoryCreateData,
  _create_data: &mut NormalModuleCreateData,
  module: &mut BoxModule,
) -> Result<()> {
  if self.options.scope.is_none() {
    if let Some(request) = module.lib_ident(LibIdentOptions {
      context: self.options.context.as_ref().unwrap_or(&Context::from("")),
    }) {
      if let Some(resolved) = self.options.content.get(request.as_ref()) {
        let original_request = module.lib_ident(LibIdentOptions {
          context: &self.options.compilation_context,
        });

        *module = Box::new(DelegatedModule::new(
          self.options.source.clone(),
          resolved.clone(),
          self.options.r#type.clone(),
          request.to_string(),
          original_request.map(|request| request.to_string()),
        ));
      }
    };
  }

  Ok(())
}
