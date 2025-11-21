use derive_more::Debug;
use futures::future::BoxFuture;
use rspack_core::{
  ModuleFactoryCreateData, NormalModuleCreateData, NormalModuleFactoryAfterResolve,
  NormalModuleFactoryBeforeResolve, Plugin,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::Utf8PathBuf;
use rspack_regex::RspackRegex;

pub struct NormalModuleReplacementPluginOptions {
  pub resource_reg_exp: RspackRegex,
  pub new_resource: NormalModuleReplacer,
}

pub enum NormalModuleReplacer {
  String(String),
  Fn(NormalModuleReplacerFn),
}

pub type NormalModuleReplacerFn = Box<
  dyn for<'a> Fn(
      &'a mut ModuleFactoryCreateData,
      Option<&'a mut NormalModuleCreateData>,
    ) -> BoxFuture<'a, Result<()>>
    + Sync
    + Send,
>;

#[plugin]
#[derive(Debug)]
pub struct NormalModuleReplacementPlugin {
  resource_reg_exp: RspackRegex,
  #[debug(skip)]
  new_resource: NormalModuleReplacer,
}

impl NormalModuleReplacementPlugin {
  pub fn new(options: NormalModuleReplacementPluginOptions) -> Self {
    Self::new_inner(options.resource_reg_exp, options.new_resource)
  }
}

#[plugin_hook(NormalModuleFactoryBeforeResolve for NormalModuleReplacementPlugin)]
async fn nmf_before_resolve(&self, data: &mut ModuleFactoryCreateData) -> Result<Option<bool>> {
  if self.resource_reg_exp.test(&data.request) {
    match &self.new_resource {
      NormalModuleReplacer::String(s) => data.request = s.clone(),
      NormalModuleReplacer::Fn(f) => f(data, None).await?,
    }
  }
  Ok(None)
}

#[plugin_hook(NormalModuleFactoryAfterResolve for NormalModuleReplacementPlugin)]
async fn nmf_after_resolve(
  &self,
  data: &mut ModuleFactoryCreateData,
  create_data: &mut NormalModuleCreateData,
) -> Result<Option<bool>> {
  if self
    .resource_reg_exp
    .test(create_data.resource_resolve_data.resource())
  {
    match &self.new_resource {
      NormalModuleReplacer::String(new_resource) => {
        // Same as [JsCreateData::update_nmf_data] in crates/rspack_binding_api/src/normal_module_factory.rs
        if Utf8PathBuf::from(new_resource).is_absolute() {
          create_data
            .resource_resolve_data
            .update_resource_data(new_resource.clone());
        } else if let Some(dir) =
          Utf8PathBuf::from(create_data.resource_resolve_data.resource()).parent()
        {
          create_data
            .resource_resolve_data
            .update_resource_data(dir.join(new_resource.clone()).to_string());
        }
      }
      NormalModuleReplacer::Fn(f) => f(data, Some(create_data)).await?,
    }
  }
  Ok(None)
}

impl Plugin for NormalModuleReplacementPlugin {
  fn name(&self) -> &'static str {
    "rspack.NormalModuleReplacementPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .normal_module_factory_hooks
      .before_resolve
      .tap(nmf_before_resolve::new(self));
    ctx
      .normal_module_factory_hooks
      .after_resolve
      .tap(nmf_after_resolve::new(self));
    Ok(())
  }
}
