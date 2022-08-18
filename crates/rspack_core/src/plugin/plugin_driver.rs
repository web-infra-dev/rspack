use std::{collections::HashMap, sync::Arc};

use rayon::prelude::*;
use tracing::instrument;

use crate::{
  ApplyContext, BoxModule, BoxedParser, CompilerOptions, ModuleType, NormalModuleFactoryContext,
  ParseModuleArgs, Plugin, PluginContext, PluginProcessAssetsOutput,
  PluginRenderManifestHookOutput, PluginRenderRuntimeHookOutput, ProcessAssetsArgs,
  RenderManifestArgs, RenderRuntimeArgs, Resolver,
};

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

  #[instrument(skip_all)]
  pub fn parse(
    &self,
    args: ParseModuleArgs,
    job_ctx: &mut NormalModuleFactoryContext,
  ) -> anyhow::Result<BoxModule> {
    let module_type = job_ctx
      .module_type
      .ok_or_else(|| anyhow::format_err!("module_type is not set"))?;

    let parser = self.registered_parser.get(&module_type).ok_or_else(|| {
      anyhow::format_err!("parser for module type {:?} is not registered", module_type)
    })?;

    let module = parser.parse(module_type, args)?;
    Ok(module)
  }

  #[instrument(skip_all)]
  pub fn render_manifest(&self, args: RenderManifestArgs) -> PluginRenderManifestHookOutput {
    let mut assets = vec![];
    self
      .plugins
      .iter()
      .try_for_each(|plugin| -> anyhow::Result<()> {
        let res = plugin.render_manifest(PluginContext::new(), args.clone())?;
        assets.extend(res);
        Ok(())
      })?;
    Ok(assets)
  }

  pub fn render_runtime(&self, args: RenderRuntimeArgs) -> PluginRenderRuntimeHookOutput {
    let mut sources = vec![];
    for plugin in &self.plugins {
      tracing::debug!("running render runtime:{}", plugin.name());
      let x = sources;
      let args = RenderRuntimeArgs {
        compilation: args.compilation,
        sources: &x,
      };
      let res = plugin.render_runtime(PluginContext::new(), args)?;
      sources = res;
    }
    Ok(sources)
  }
  #[instrument(skip_all)]
  pub fn process_assets(&self, args: ProcessAssetsArgs) -> PluginProcessAssetsOutput {
    self
      .plugins
      .iter()
      .try_for_each(|plugin| -> anyhow::Result<()> {
        plugin.process_assets(
          PluginContext::new(),
          ProcessAssetsArgs {
            compilation: args.compilation,
          },
        )?;
        Ok(())
      })?;
    Ok(())
  }
}
