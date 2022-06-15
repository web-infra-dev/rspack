use std::{collections::HashMap, sync::Arc};

use nodejs_resolver::Resolver;
use tracing::instrument;

use crate::{
  Asset, BoxModule, CompilerOptions, JobContext, LoadArgs, ParseModuleArgs, Plugin, PluginContext,
  PluginResolveHookOutput, RenderManifestArgs, ResolveArgs, SourceType,
};

#[derive(Debug)]
pub struct PluginDriver {
  pub(crate) options: Arc<CompilerOptions>,
  pub plugins: Vec<Box<dyn Plugin>>,
  pub resolver: Arc<Resolver>,
  pub module_parser: HashMap<SourceType, usize>,
}

impl PluginDriver {
  pub fn new(
    options: Arc<CompilerOptions>,
    plugins: Vec<Box<dyn Plugin>>,
    resolver: Arc<Resolver>,
  ) -> Self {
    let module_parser: HashMap<SourceType, usize> = plugins
      .iter()
      .enumerate()
      .filter_map(|(index, plugin)| {
        let registered = plugin.register_parse_module(PluginContext::new())?;
        Some(
          registered
            .into_iter()
            .map(|source_type| (source_type, index))
            .collect::<Vec<_>>(),
        )
        // Some((plugin.register_parse_module(PluginContext::new())?, index))
      })
      .flatten()
      .collect();
    Self {
      options,
      plugins,
      resolver,
      module_parser,
    }
  }

  pub async fn resolve(
    &self,
    args: ResolveArgs<'_>,
    job_ctx: &mut JobContext,
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
    job_ctx: &mut JobContext,
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

  #[instrument(skip_all)]
  pub fn parse_module(
    &self,
    args: ParseModuleArgs,
    job_ctx: &mut JobContext,
  ) -> anyhow::Result<BoxModule> {
    let parser_index = self
      .module_parser
      .get(
        job_ctx
          .source_type
          .as_ref()
          .ok_or_else(|| anyhow::format_err!("source type is empty for {:?}", args.uri))?,
      )
      .unwrap_or_else(|| panic!("No parser for source type {:?}", &job_ctx.source_type));
    let module =
      self.plugins[*parser_index].parse_module(PluginContext::with_context(job_ctx), args)?;
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
