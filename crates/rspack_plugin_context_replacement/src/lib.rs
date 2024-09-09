use derivative::Derivative;
use futures::future::BoxFuture;
use rspack_core::{
  AfterResolveResult, ApplyContext, BeforeResolveResult, CompilerOptions,
  ContextModuleFactoryAfterResolve, ContextModuleFactoryBeforeResolve, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_regex::RspackRegex;

enum ResolveResult {
  Before(BeforeResolveResult),
  After(AfterResolveResult),
}

pub type ContentCallback =
  Box<dyn Fn(&mut ResolveResult) -> BoxFuture<'static, Result<()>> + Sync + Send>;

pub struct ContextReplacementPluginOptions {
  resource_reg_exp: RspackRegex,
  new_content_resource: Option<String>,
  new_content_recursive: Option<bool>,
  new_content_reg_exp: Option<RspackRegex>,
  new_content_callback: Option<ContentCallback>,
}

#[plugin]
#[derive(Derivative)]
#[derivative(Debug)]
pub struct ContextReplacementPlugin {
  resource_reg_exp: RspackRegex,
  new_content_resource: Option<String>,
  new_content_recursive: Option<bool>,
  new_content_reg_exp: Option<RspackRegex>,
  #[derivative(Debug = "ignore")]
  new_content_callback: Option<ContentCallback>,
}

impl ContextReplacementPlugin {
  pub fn new(options: ContextReplacementPluginOptions) -> Self {
    Self::new_inner(
      options.resource_reg_exp,
      options.new_content_resource,
      options.new_content_recursive,
      options.new_content_reg_exp,
      options.new_content_callback,
    )
  }
}

#[plugin_hook(ContextModuleFactoryBeforeResolve for ContextReplacementPlugin)]
async fn cmf_before_resolve(&self, mut result: BeforeResolveResult) -> Result<BeforeResolveResult> {
  if let BeforeResolveResult::Data(data) = &mut result {
    if let Some(request) = &data.request {
      if self.resource_reg_exp.test(request) {
        data.request = self.new_content_resource.clone();
      }
      if let Some(new_content_recursive) = self.new_content_recursive {
        data.recursive = new_content_recursive;
      }
      if let Some(new_content_reg_exp) = &self.new_content_reg_exp {
        data.reg_exp = Some(new_content_reg_exp.clone());
      }
      if let Some(new_content_callback) = &self.new_content_callback {
        // new_content_callback(&mut result).await?;
      } else {
        // for (const d of result.dependencies) {
        //   if (d.critical) d.critical = false;
        // }
      }
    }
  }

  Ok(result)
}

#[plugin_hook(ContextModuleFactoryAfterResolve for ContextReplacementPlugin)]
async fn cmf_after_resolve(&self, mut result: AfterResolveResult) -> Result<AfterResolveResult> {
  if let AfterResolveResult::Data(data) = &mut result {
    if self.resource_reg_exp.test(data.resource.as_str()) {
      if let Some(new_content_resource) = &self.new_content_resource {
        if new_content_resource.starts_with('/') || new_content_resource.chars().nth(1) == Some(':')
        {
          data.resource = new_content_resource.clone().into();
        } else {
          // result.resource = join(
          //   /** @type {InputFileSystem} */ (compiler.inputFileSystem),
          //   result.resource,
          //   newContentResource
          // );
        }
      }
      if let Some(new_content_recursive) = self.new_content_recursive {
        data.recursive = new_content_recursive;
      }
      if let Some(new_content_reg_exp) = &self.new_content_reg_exp {
        data.reg_exp = Some(new_content_reg_exp.clone());
      }
      if let Some(new_content_callback) = &self.new_content_callback {
        // new_content_callback(&mut result).await?;
      } else {
        // for (const d of result.dependencies) {
        //   if (d.critical) d.critical = false;
        // }
      }
    }
  }
  Ok(result)
}

impl Plugin for ContextReplacementPlugin {
  fn name(&self) -> &'static str {
    "rspack.ContextReplacementPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .context_module_factory_hooks
      .before_resolve
      .tap(cmf_before_resolve::new(self));
    ctx
      .context
      .context_module_factory_hooks
      .after_resolve
      .tap(cmf_after_resolve::new(self));
    Ok(())
  }
}
