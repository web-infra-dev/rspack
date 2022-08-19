use std::sync::Arc;

use rspack_error::Result;

pub use rspack_loader_runner::*;

use crate::{CompilerOptions, ModuleRule};

pub struct LoaderRunnerRunner {
  pub options: Arc<CompilerOptions>,
}

impl LoaderRunnerRunner {
  pub fn new(options: Arc<CompilerOptions>) -> Self {
    Self { options }
  }

  pub async fn run(&self, resource_data: ResourceData) -> Result<LoaderResult> {
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
      .flat_map(|module_rule| module_rule.uses.iter().map(Box::as_ref).rev())
      .collect::<Vec<_>>();

    LoaderRunner::new(resource_data.clone()).run(&loaders).await
  }
}
