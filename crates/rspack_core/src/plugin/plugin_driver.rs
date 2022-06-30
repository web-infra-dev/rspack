use std::{collections::HashMap, sync::Arc};

use nodejs_resolver::Resolver;
use tracing::instrument;

use crate::{
  ApplyContext, Asset, BoxModule, BoxedParser, CompilerOptions, LoadArgs, ModuleType,
  NormalModuleFactoryContext, ParseModuleArgs, Plugin, PluginContext, PluginResolveHookOutput,
  RenderManifestArgs, ResolveArgs,
};

use rayon::prelude::*;

#[derive(Debug)]
pub struct PluginDriver {
  pub(crate) options: Arc<CompilerOptions>,
  pub plugins: Vec<Box<dyn Plugin>>,
  pub resolver: Arc<Resolver>,
  pub registered_parser: HashMap<ModuleType, BoxedParser>,
}

impl PluginDriver {
  pub fn new(
    options: Arc<CompilerOptions>,
    mut plugins: Vec<Box<dyn Plugin>>,
    resolver: Arc<Resolver>,
  ) -> Self {
    let registered_parser = plugins
      .par_iter_mut()
      .map(|plugin| {
        let mut apply_context = ApplyContext::default();
        plugin
          .apply(PluginContext::with_context(&mut apply_context))
          .unwrap();
        apply_context
      })
      .flat_map(|apply_context| {
        apply_context
          .registered_parser
          .into_iter()
          .collect::<Vec<_>>()
      })
      .collect::<HashMap<ModuleType, BoxedParser>>();

    Self {
      options,
      plugins,
      resolver,
      registered_parser,
    }
  }

  pub async fn resolve(
    &self,
    args: ResolveArgs<'_>,
    job_ctx: &mut NormalModuleFactoryContext,
  ) -> PluginResolveHookOutput {
    for plugin in &self.plugins {
      let output = plugin
        .resolve(PluginContext::with_context(job_ctx), args.clone())
        .await?;
      if output.is_some() {
        return Ok(output);
      }
    }
    Ok(None)
  }

  pub async fn load(
    &self,
    args: LoadArgs<'_>,
    job_ctx: &mut NormalModuleFactoryContext,
  ) -> PluginResolveHookOutput {
    for plugin in &self.plugins {
      let output = plugin
        .load(PluginContext::with_context(job_ctx), args.clone())
        .await?;
      if output.is_some() {
        return Ok(output);
      }
    }
    Ok(None)
  }

  // #[instrument(skip_all)]
  pub fn parse(
    &self,
    args: ParseModuleArgs,
    job_ctx: &mut NormalModuleFactoryContext,
  ) -> anyhow::Result<BoxModule> {
    let module_type = job_ctx
      .module_type
      .ok_or_else(|| anyhow::format_err!("module_type is not set"))?;

    let parser = self
      .registered_parser
      .get(&module_type)
      .ok_or_else(|| anyhow::format_err!("parser is not registered"))?;

    let module = parser.parse(module_type, args)?;
    Ok(module)
  }

  #[instrument(skip_all)]
  pub fn render_manifest(&self, args: RenderManifestArgs) -> Vec<Asset> {
    self
      .plugins
      .iter()
      .flat_map(|plugin| {
        plugin
          .render_manifest(PluginContext::new(), args.clone())
          .unwrap()
      })
      .collect()
  }
}
