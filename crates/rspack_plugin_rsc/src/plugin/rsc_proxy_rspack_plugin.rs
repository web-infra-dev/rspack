use async_trait::async_trait;
use rspack_core::{
  ApplyContext, Compilation, CompilerMake, CompilerOptions, EntryDependency, EntryOptions, Plugin,
  PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

#[derive(Debug, Default, Clone)]
pub struct RSCProxyRspackPluginOptions {}

#[plugin]
#[derive(Debug, Default, Clone)]
pub struct RSCProxyRspackPlugin {
  pub options: RSCProxyRspackPluginOptions,
}

impl RSCProxyRspackPlugin {
  pub fn new(options: RSCProxyRspackPluginOptions) -> Self {
    Self::new_inner(options)
  }
  async fn add_entry(&self, compilation: &mut Compilation) -> Result<()> {
    // TODO: multiple server entry support
    let context = compilation.options.context.clone();
    let request = format!(
      "rsc-server-action-entry-loader.js?from={}&name={}",
      "server-entry", "server-entry"
    );
    let entry = Box::new(EntryDependency::new(request, context.clone(), None, false));
    compilation
      .add_include(
        entry,
        EntryOptions {
          name: Some(String::from("server-entry")),
          ..Default::default()
        },
      )
      .await?;
    Ok(())
  }
}

#[plugin_hook(CompilerMake for RSCProxyRspackPlugin)]
async fn make(&self, compilation: &mut Compilation) -> Result<()> {
  self.add_entry(compilation).await?;
  Ok(())
}

#[async_trait]
impl Plugin for RSCProxyRspackPlugin {
  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx.context.compiler_hooks.make.tap(make::new(self));
    Ok(())
  }
}
