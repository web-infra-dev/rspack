use std::sync::Arc;

use rspack_error::Result;

pub use rspack_loader_runner::{
  Content, Loader, LoaderContext, LoaderResult, LoaderRunner, LoaderRunnerAdditionalContext,
  ResourceData,
};

use crate::{CompilerOptions, ModuleRule, ModuleType, PluginDriver};

#[derive(Debug)]
pub struct CompilerContext {
  pub options: Arc<CompilerOptions>,
}

pub type CompilationContext = ();

pub type BoxedLoader = rspack_loader_runner::BoxedLoader<CompilerContext, CompilationContext>;

pub struct LoaderRunnerRunner {
  pub options: Arc<CompilerOptions>,
  pub plugin_driver: Arc<PluginDriver>,
  pub compiler_context: CompilerContext,
}

type ResolvedModuleType = Option<ModuleType>;

impl LoaderRunnerRunner {
  pub fn new(options: Arc<CompilerOptions>, plugin_driver: Arc<PluginDriver>) -> Self {
    let compiler_context = CompilerContext {
      options: options.clone(),
    };

    Self {
      options,
      plugin_driver,
      compiler_context,
    }
  }

  pub async fn run(
    &self,
    resource_data: ResourceData,
  ) -> Result<(LoaderResult, ResolvedModuleType)> {
    // Progressive module type resolution:
    // Stage 1: maintain the resolution logic via file extension
    // TODO: Stage 2:
    //           1. remove all extension based module type resolution, and let `module.rules[number].type` to handle this(everything is based on its config)
    //           2. set default module type to `Js`, it equals to `javascript/auto` in webpack.
    let mut resolved_module_type: ResolvedModuleType = None;

    let loaders = self
      .options
      .module
      .rules
      .iter()
      .filter_map(|module_rule| -> Option<Result<&ModuleRule>> {
        if let Some(func) = &module_rule.func__ {
          match func(&resource_data) {
            Ok(result) => {
              if result {
                return Some(Ok(module_rule));
              }

              return None
            },
            Err(e) => {
              return Some(Err(e.into()))
            }
          }
        }

        // Include all modules that pass test assertion. If you supply a Rule.test option, you cannot also supply a `Rule.resource`.
        // See: https://webpack.js.org/configuration/module/#ruletest
        if let Some(test_rule) = &module_rule.test && test_rule.is_match(&resource_data.resource) {
          return Some(Ok(module_rule));
        } else if let Some(resource_rule) = &module_rule.resource && resource_rule.is_match(&resource_data.resource) {
          return Some(Ok(module_rule));
        }

        if let Some(resource_query_rule) = &module_rule.resource_query && let Some(resource_query) = &resource_data.resource_query && resource_query_rule.is_match(resource_query) {
          return Some(Ok(module_rule));
        }


        None
      })
      .collect::<Result<Vec<_>>>()?
      .into_iter()
      .flat_map(|module_rule| {
        if module_rule.module_type.is_some() {
          resolved_module_type = module_rule.module_type;
        };

        module_rule.uses.iter().map(Box::as_ref).rev()
      })
      .collect::<Vec<_>>();

    Ok((
      LoaderRunner::new(resource_data.clone(), vec![])
        .run(
          &loaders,
          &LoaderRunnerAdditionalContext {
            compiler: &self.compiler_context,
            compilation: &(),
          },
        )
        .await?,
      resolved_module_type,
    ))
  }
}
