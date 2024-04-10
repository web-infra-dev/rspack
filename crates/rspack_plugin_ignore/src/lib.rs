use std::fmt::Debug;

use rspack_core::{ApplyContext, BeforeResolveArgs, CompilerOptions, Plugin, PluginContext};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook, AsyncSeriesBail};
use rspack_regex::RspackRegex;

#[derive(Debug)]
pub struct IgnorePluginOptions {
  pub resource_reg_exp: RspackRegex,
  pub context_reg_exp: Option<RspackRegex>,
}

#[plugin]
#[derive(Debug)]
pub struct IgnorePlugin {
  options: IgnorePluginOptions,
}

impl IgnorePlugin {
  pub fn new(options: IgnorePluginOptions) -> Self {
    Self::new_inner(options)
  }

  fn check_ignore(&self, resolve_data: &mut BeforeResolveArgs) -> Option<bool> {
    let resource_reg_exp: &RspackRegex = &self.options.resource_reg_exp;

    if resource_reg_exp.test(&resolve_data.request) {
      if let Some(context_reg_exp) = &self.options.context_reg_exp {
        if context_reg_exp.test(&resolve_data.context) {
          return Some(false);
        }
      } else {
        return Some(false);
      }
    }
    None
  }
}

#[plugin_hook(AsyncSeriesBail<BeforeResolveArgs, bool> for IgnorePlugin)]
async fn before_resolve(&self, resolve_data: &mut BeforeResolveArgs) -> Result<Option<bool>> {
  Ok(self.check_ignore(resolve_data))
}

impl Plugin for IgnorePlugin {
  fn name(&self) -> &'static str {
    "IgnorePlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .normal_module_factory_hooks
      .before_resolve
      .tap(before_resolve::new(self));

    ctx
      .context
      .context_module_factory_hooks
      .before_resolve
      .tap(before_resolve::new(self));

    Ok(())
  }
}
