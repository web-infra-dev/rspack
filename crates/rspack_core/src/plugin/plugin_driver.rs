use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

use rayon::prelude::*;
use rspack_loader_runner::ResourceData;
use tracing::instrument;

use crate::{
  ApplyContext, BoxedParserAndGeneratorBuilder, Compilation, CompilerOptions, Content, DoneArgs,
  FactorizeAndBuildArgs, ModuleType, NormalModule, NormalModuleFactoryContext, OptimizeChunksArgs,
  Plugin, PluginBuildEndHookOutput, PluginBuildStartHookOutput, PluginContext,
  PluginFactorizeAndBuildHookOutput, PluginProcessAssetsOutput, PluginRenderManifestHookOutput,
  PluginRenderRuntimeHookOutput, ProcessAssetsArgs, RenderManifestArgs, RenderRuntimeArgs,
  Resolver, Stats,
};
use rspack_error::{Diagnostic, Result};

pub struct PluginDriver {
  pub(crate) options: Arc<CompilerOptions>,
  pub plugins: Vec<Box<dyn Plugin>>,
  pub resolver: Arc<Resolver>,
  // pub registered_parser: HashMap<ModuleType, BoxedParser>,
  pub registered_parser_and_generator_builder: HashMap<ModuleType, BoxedParserAndGeneratorBuilder>,
  /// Collecting error generated by plugin phase, e.g., `Syntax Error`
  pub diagnostics: Arc<Mutex<Vec<Diagnostic>>>,
}

impl std::fmt::Debug for PluginDriver {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("PluginDriver")
      .field("options", &self.options)
      .field("plugins", &self.plugins)
      .field("resolver", &self.resolver)
      // field("registered_parser", &self.registered_parser)
      .field("registered_parser_and_generator_builder", &"{..}")
      .field("diagnostics", &self.diagnostics)
      .finish()
  }
}

impl PluginDriver {
  pub fn new(
    options: Arc<CompilerOptions>,
    mut plugins: Vec<Box<dyn Plugin>>,
    resolver: Arc<Resolver>,
  ) -> Self {
    //   let registered_parser = plugins
    //     .par_iter_mut()
    //     .map(|plugin| {
    //       let mut apply_context = ApplyContext::default();
    //       plugin
    //         .apply(PluginContext::with_context(&mut apply_context))
    //         .unwrap();
    //       apply_context
    //     })
    //     .flat_map(|apply_context| {
    //       apply_context
    //         .registered_parser
    //         .into_iter()
    //         .collect::<Vec<_>>()
    //     })
    //     .collect::<HashMap<ModuleType, BoxedParser>>();

    let registered_parser_and_generator_builder = plugins
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
          .registered_parser_and_generator_builder
          .into_iter()
          .collect::<Vec<_>>()
      })
      .collect::<HashMap<ModuleType, BoxedParserAndGeneratorBuilder>>();

    Self {
      options,
      plugins,
      resolver,
      // registered_parser,
      registered_parser_and_generator_builder,
      diagnostics: Arc::new(Mutex::new(vec![])),
    }
  }

  pub fn take_diagnostic(&self) -> Vec<Diagnostic> {
    let mut diagnostic = self.diagnostics.lock().unwrap();
    std::mem::take(&mut diagnostic)
  }

  /// Read resource with the given `resource_data`
  ///
  /// Warning:
  /// Webpack does not expose this as the documented API, even though you can reach this with `NormalModule.getCompilationHooks(compilation)`.
  /// For the most of time, you would not need this.
  #[instrument(name = "plugin:read_resource")]
  pub async fn read_resource(&self, resource_data: &ResourceData) -> Result<Option<Content>> {
    for plugin in &self.plugins {
      let result = plugin.read_resource(resource_data).await?;
      if result.is_some() {
        return Ok(result);
      }
    }

    Ok(None)
  }

  // Disable this clippy rule because lock error is un recoverable, we don't need to
  // bubble it.
  // #[allow(clippy::unwrap_in_result)]
  // #[instrument(skip_all)]
  // pub fn parse(
  //   &self,
  //   args: ParseModuleArgs,
  //   job_ctx: &mut NormalModuleFactoryContext,
  // ) -> Result<BoxModule> {
  //   let module_type = job_ctx.module_type.ok_or_else(|| {
  //     Error::InternalError(format!(
  //       "Failed to parse {} as module_type is not set",
  //       args.uri
  //     ))
  //   })?;

  //   let parser = self.registered_parser.get(&module_type).ok_or_else(|| {
  //     Error::InternalError(format!(
  //       "parser for module type {:?} is not registered",
  //       module_type
  //     ))
  //   })?;

  //   let mut module = parser.parse(module_type, args)?;
  //   // Collecting coverable parse error
  //   if !module.diagnostic.is_empty() {
  //     let mut diagnostic = self.diagnostics.lock().unwrap();
  //     diagnostic.append(&mut module.diagnostic);
  //   }
  //   Ok(module.take_inner())
  // }

  #[instrument(name = "plugin:render_manifest", skip_all)]
  pub fn render_manifest(&self, args: RenderManifestArgs) -> PluginRenderManifestHookOutput {
    let mut assets = vec![];
    self.plugins.iter().try_for_each(|plugin| -> Result<()> {
      let res = plugin.render_manifest(PluginContext::new(), args.clone())?;
      assets.extend(res);
      Ok(())
    })?;
    Ok(assets)
  }

  pub async fn factorize_and_build(
    &self,
    args: FactorizeAndBuildArgs<'_>,
    job_ctx: &mut NormalModuleFactoryContext,
  ) -> PluginFactorizeAndBuildHookOutput {
    for plugin in &self.plugins {
      tracing::trace!("running render runtime:{}", plugin.name());
      if let Some(module) = plugin
        .factorize_and_build(PluginContext::new(), args.clone(), job_ctx)
        .await?
      {
        return Ok(Some(module));
      }
    }
    Ok(None)
  }
  #[instrument(name = "plugin:render_runtime")]
  pub fn render_runtime(&self, args: RenderRuntimeArgs) -> PluginRenderRuntimeHookOutput {
    let mut sources = vec![];
    for plugin in &self.plugins {
      tracing::trace!("running render runtime:{}", plugin.name());
      let args = RenderRuntimeArgs {
        compilation: args.compilation,
        sources,
      };
      let res = plugin.render_runtime(PluginContext::new(), args)?;
      sources = res;
    }
    Ok(sources)
  }
  #[instrument(name = "plugin:process_assets", skip_all)]
  pub async fn process_assets(&mut self, args: ProcessAssetsArgs<'_>) -> PluginProcessAssetsOutput {
    for plugin in &mut self.plugins {
      plugin
        .process_assets(
          PluginContext::new(),
          ProcessAssetsArgs {
            compilation: args.compilation,
          },
        )
        .await?;
    }
    Ok(())
  }

  #[instrument(name = "plugin:make", skip_all)]
  pub fn make(&self, compilation: &Compilation) -> PluginBuildStartHookOutput {
    for plugin in &self.plugins {
      plugin.make(PluginContext::new(), compilation)?;
    }
    Ok(())
  }

  #[instrument(name = "plugin:done", skip_all)]
  pub async fn done<'s, 'c>(&mut self, stats: &'s mut Stats<'c>) -> PluginBuildEndHookOutput {
    for plugin in &mut self.plugins {
      plugin
        .done(PluginContext::new(), DoneArgs { stats })
        .await?;
    }
    Ok(())
  }
  #[instrument(name = "plugin:optimize_chunks")]
  pub fn optimize_chunks(&mut self, compilation: &mut Compilation) -> Result<()> {
    for plugin in &mut self.plugins {
      plugin.optimize_chunks(PluginContext::new(), OptimizeChunksArgs { compilation })?;
    }
    Ok(())
  }

  #[instrument(name = "plugin:build_module")]
  pub async fn build_module(&self, module: &mut NormalModule) -> Result<()> {
    for plugin in &self.plugins {
      plugin.build_module(module).await?;
    }
    Ok(())
  }

  #[instrument(name = "plugin:succeed_module")]
  pub async fn succeed_module(&self, module: &NormalModule) -> Result<()> {
    for plugin in &self.plugins {
      plugin.succeed_module(module).await?;
    }
    Ok(())
  }
}
