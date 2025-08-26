use derive_more::Debug;
use futures::future::BoxFuture;
use rspack_core::{
  BeforeResolveResult, ContextModuleFactoryBeforeResolve, ModuleFactoryCreateData,
  NormalModuleFactoryBeforeResolve, Plugin,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_regex::RspackRegex;

pub type CheckResourceFn =
  Box<dyn for<'a> Fn(&'a str, &'a str) -> BoxFuture<'a, Result<bool>> + Sync + Send>;

pub enum CheckResourceContent {
  Fn(CheckResourceFn),
}

#[derive(Debug)]
pub struct IgnorePluginOptions {
  pub resource_reg_exp: Option<RspackRegex>,
  pub context_reg_exp: Option<RspackRegex>,
  #[debug(skip)]
  pub check_resource: Option<CheckResourceContent>,
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

  async fn check_ignore(&self, request: &str, context: &str) -> Result<Option<bool>> {
    if let Some(check_resource) = &self.options.check_resource {
      match check_resource {
        CheckResourceContent::Fn(check) => match check(request, context).await {
          Ok(true) => return Ok(Some(false)),
          Err(err) => return Err(err.wrap_err("IgnorePlugin: failed to call `checkResource`")),
          _ => {}
        },
      }
    }

    if let Some(resource_reg_exp) = &self.options.resource_reg_exp
      && resource_reg_exp.test(request)
    {
      if let Some(context_reg_exp) = &self.options.context_reg_exp {
        if context_reg_exp.test(context) {
          return Ok(Some(false));
        }
      } else {
        return Ok(Some(false));
      }
    }

    Ok(None)
  }
}

#[plugin_hook(NormalModuleFactoryBeforeResolve for IgnorePlugin)]
async fn nmf_before_resolve(&self, data: &mut ModuleFactoryCreateData) -> Result<Option<bool>> {
  self.check_ignore(&data.request, &data.context).await
}

#[plugin_hook(ContextModuleFactoryBeforeResolve for IgnorePlugin)]
async fn cmf_before_resolve(&self, data: BeforeResolveResult) -> Result<BeforeResolveResult> {
  match data {
    BeforeResolveResult::Ignored => Ok(BeforeResolveResult::Ignored),
    BeforeResolveResult::Data(d) => {
      if let Some(false) = self.check_ignore(&d.request, &d.context).await? {
        Ok(BeforeResolveResult::Ignored)
      } else {
        Ok(BeforeResolveResult::Data(d))
      }
    }
  }
}

impl Plugin for IgnorePlugin {
  fn name(&self) -> &'static str {
    "IgnorePlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .normal_module_factory_hooks
      .before_resolve
      .tap(nmf_before_resolve::new(self));

    ctx
      .context_module_factory_hooks
      .before_resolve
      .tap(cmf_before_resolve::new(self));

    Ok(())
  }
}
