use std::{
  collections::HashMap,
  path::Path,
  sync::{Arc, Mutex},
};

use rayon::prelude::*;
use rspack_error::{Diagnostic, Result};
use rspack_loader_runner::ResourceData;
use tracing::instrument;

use crate::{
  AdditionalChunkRuntimeRequirementsArgs, ApplyContext, BoxLoader, BoxedParserAndGeneratorBuilder,
  Chunk, ChunkAssetArgs, ChunkHashArgs, Compilation, CompilationArgs, CompilerOptions, Content,
  ContentHashArgs, DoneArgs, FactorizeArgs, JsChunkHashArgs, Module, ModuleArgs, ModuleType,
  NormalModule, NormalModuleBeforeResolveArgs, NormalModuleFactoryContext, OptimizeChunksArgs,
  Plugin, PluginAdditionalChunkRuntimeRequirementsOutput, PluginBuildEndHookOutput,
  PluginChunkHashHookOutput, PluginCompilationHookOutput, PluginContext, PluginFactorizeHookOutput,
  PluginJsChunkHashHookOutput, PluginMakeHookOutput, PluginModuleHookOutput,
  PluginNormalModuleFactoryBeforeResolveOutput, PluginProcessAssetsOutput,
  PluginRenderChunkHookOutput, PluginRenderHookOutput, PluginRenderManifestHookOutput,
  PluginRenderModuleContentOutput, PluginRenderStartupHookOutput, PluginThisCompilationHookOutput,
  ProcessAssetsArgs, RenderArgs, RenderChunkArgs, RenderManifestArgs, RenderModuleContentArgs,
  RenderStartupArgs, Resolver, ResolverFactory, SourceType, Stats, ThisCompilationArgs,
};

pub struct PluginDriver {
  pub(crate) options: Arc<CompilerOptions>,
  pub plugins: Vec<Box<dyn Plugin>>,
  pub resolver_factory: Arc<ResolverFactory>,
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
    resolver_factory: Arc<ResolverFactory>,
  ) -> Self {
    let registered_parser_and_generator_builder = plugins
      .par_iter_mut()
      .map(|plugin| {
        let mut apply_context = ApplyContext::default();
        plugin
          .apply(PluginContext::with_context(&mut apply_context))
          .expect("TODO:");
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
      resolver_factory,
      // registered_parser,
      registered_parser_and_generator_builder,
      diagnostics: Arc::new(Mutex::new(vec![])),
    }
  }

  pub fn take_diagnostic(&self) -> Vec<Diagnostic> {
    let mut diagnostic = self.diagnostics.lock().expect("TODO:");
    std::mem::take(&mut diagnostic)
  }

  /// Read resource with the given `resource_data`
  ///
  /// Warning:
  /// Webpack does not expose this as the documented API, even though you can reach this with `NormalModule.getCompilationHooks(compilation)`.
  /// For the most of time, you would not need this.
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
  //     let mut diagnostic = self.diagnostics.lock().expect("TODO:");
  //     diagnostic.append(&mut module.diagnostic);
  //   }
  //   Ok(module.take_inner())
  // }

  /// Runs a plugin after a compilation has been created.
  ///
  /// See: https://webpack.js.org/api/compiler-hooks/#compilation
  #[instrument(name = "plugin:compilation", skip_all)]
  pub async fn compilation(
    &mut self,
    compilation: &mut Compilation,
  ) -> PluginCompilationHookOutput {
    for plugin in &mut self.plugins {
      plugin.compilation(CompilationArgs { compilation }).await?;
    }

    Ok(())
  }

  #[instrument(name = "plugin:chunk_asset", skip_all)]
  pub async fn chunk_asset(
    &mut self,
    chunk: &Chunk,
    filename: String,
  ) -> PluginCompilationHookOutput {
    for plugin in &mut self.plugins {
      plugin
        .chunk_asset(&ChunkAssetArgs {
          chunk,
          filename: &filename,
        })
        .await?;
    }

    Ok(())
  }

  pub async fn before_compile(
    &mut self,
    // compilationParams: &mut CompilationParams<'_>,
  ) -> PluginCompilationHookOutput {
    for plugin in &mut self.plugins {
      plugin.before_compile().await?;
    }

    Ok(())
  }
  /// Executed while initializing the compilation, right before emitting the compilation event. This hook is not copied to child compilers.
  ///
  /// See: https://webpack.js.org/api/compiler-hooks/#thiscompilation
  pub async fn this_compilation(
    &mut self,
    compilation: &mut Compilation,
  ) -> PluginThisCompilationHookOutput {
    for plugin in &mut self.plugins {
      plugin
        .this_compilation(ThisCompilationArgs {
          this_compilation: compilation,
        })
        .await?;
    }

    Ok(())
  }

  pub async fn content_hash(
    &self,
    args: &ContentHashArgs<'_>,
  ) -> Result<Vec<Option<(SourceType, String)>>> {
    let mut result = vec![];
    for plugin in &self.plugins {
      let hash = plugin.content_hash(PluginContext::new(), args).await?;
      result.push(hash);
    }
    Ok(result)
  }

  pub async fn chunk_hash(&self, args: &ChunkHashArgs<'_>) -> PluginChunkHashHookOutput {
    for plugin in &self.plugins {
      if let Some(hash) = plugin.chunk_hash(PluginContext::new(), args).await? {
        return Ok(Some(hash));
      }
    }
    Ok(None)
  }

  pub async fn render_manifest(
    &self,
    args: RenderManifestArgs<'_>,
  ) -> PluginRenderManifestHookOutput {
    let mut assets = vec![];
    for plugin in &self.plugins {
      let res = plugin
        .render_manifest(PluginContext::new(), args.clone())
        .await?;
      tracing::trace!(
        "For Chunk({:?}), Plugin({}) generate files {:?}",
        args.chunk().id,
        plugin.name(),
        res
          .iter()
          .map(|manifest| manifest.filename())
          .collect::<Vec<_>>()
      );
      assets.extend(res);
    }
    Ok(assets)
  }

  pub async fn render_chunk(&self, args: RenderChunkArgs<'_>) -> PluginRenderChunkHookOutput {
    for plugin in &self.plugins {
      if let Some(source) = plugin.render_chunk(PluginContext::new(), &args).await? {
        return Ok(Some(source));
      }
    }
    Ok(None)
  }

  pub fn render(&self, args: RenderArgs) -> PluginRenderHookOutput {
    for plugin in &self.plugins {
      if let Some(source) = plugin.render(PluginContext::new(), &args)? {
        return Ok(Some(source));
      }
    }
    Ok(None)
  }

  pub fn render_startup(&self, args: RenderStartupArgs) -> PluginRenderStartupHookOutput {
    let mut source = args.source;
    for plugin in &self.plugins {
      if let Some(s) = plugin.render_startup(
        PluginContext::new(),
        &RenderStartupArgs {
          source: source.clone(),
          ..args
        },
      )? {
        source = s;
      }
    }
    Ok(Some(source))
  }

  pub fn js_chunk_hash(&self, mut args: JsChunkHashArgs) -> PluginJsChunkHashHookOutput {
    for plugin in &self.plugins {
      plugin.js_chunk_hash(PluginContext::new(), &mut args)?
    }
    Ok(())
  }

  pub fn render_module_content(
    &self,
    args: RenderModuleContentArgs,
  ) -> PluginRenderModuleContentOutput {
    for plugin in &self.plugins {
      if let Some(source) = plugin.render_module_content(PluginContext::new(), &args)? {
        return Ok(Some(source));
      }
    }
    Ok(None)
  }

  pub async fn factorize(
    &self,
    args: FactorizeArgs<'_>,
    job_ctx: &mut NormalModuleFactoryContext,
  ) -> PluginFactorizeHookOutput {
    for plugin in &self.plugins {
      if let Some(module) = plugin
        .factorize(PluginContext::new(), args.clone(), job_ctx)
        .await?
      {
        return Ok(Some(module));
      }
    }
    Ok(None)
  }

  pub async fn module(&self, args: ModuleArgs) -> PluginModuleHookOutput {
    for plugin in &self.plugins {
      tracing::trace!("running render runtime:{}", plugin.name());
      if let Some(module) = plugin.module(PluginContext::new(), &args).await? {
        return Ok(Some(module));
      }
    }
    Ok(None)
  }

  pub async fn before_resolve(
    &self,
    args: NormalModuleBeforeResolveArgs<'_>,
  ) -> PluginNormalModuleFactoryBeforeResolveOutput {
    for plugin in &self.plugins {
      tracing::trace!("running resolve for scheme:{}", plugin.name());
      if let Some(data) = plugin.before_resolve(PluginContext::new(), &args).await? {
        return Ok(Some(data));
      }
    }
    Ok(None)
  }

  pub async fn context_module_before_resolve(
    &self,
    args: NormalModuleBeforeResolveArgs<'_>,
  ) -> PluginNormalModuleFactoryBeforeResolveOutput {
    for plugin in &self.plugins {
      tracing::trace!("running resolve for scheme:{}", plugin.name());
      if let Some(data) = plugin
        .context_module_before_resolve(PluginContext::new(), &args)
        .await?
      {
        return Ok(Some(data));
      }
    }
    Ok(None)
  }

  pub async fn normal_module_factory_resolve_for_scheme(
    &self,
    args: ResourceData,
  ) -> Result<ResourceData> {
    let mut args = args;
    for plugin in &self.plugins {
      tracing::trace!("running resolve for scheme:{}", plugin.name());
      let (ret, stop) = plugin
        .normal_module_factory_resolve_for_scheme(PluginContext::new(), args)
        .await?;
      if stop {
        return Ok(ret);
      } else {
        args = ret;
      }
    }
    Ok(args)
  }

  #[instrument(name = "plugin:additional_chunk_runtime_requirements", skip_all)]
  pub fn additional_chunk_runtime_requirements(
    &self,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    for plugin in &self.plugins {
      plugin.additional_chunk_runtime_requirements(PluginContext::new(), args)?;
    }
    Ok(())
  }

  #[instrument(name = "plugin:additional_tree_runtime_requirements", skip_all)]
  pub fn additional_tree_runtime_requirements(
    &self,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    for plugin in &self.plugins {
      plugin.additional_tree_runtime_requirements(PluginContext::new(), args)?;
    }
    Ok(())
  }

  #[instrument(name = "plugin:runtime_requirements_in_tree", skip_all)]
  pub fn runtime_requirements_in_tree(
    &self,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    for plugin in &self.plugins {
      plugin.runtime_requirements_in_tree(PluginContext::new(), args)?;
    }
    Ok(())
  }

  #[instrument(name = "plugin:process_assets", skip_all)]
  pub async fn process_assets(&mut self, args: ProcessAssetsArgs<'_>) -> PluginProcessAssetsOutput {
    macro_rules! run_stage {
      ($stage: ident) => {
        for plugin in &mut self.plugins {
          plugin
            .$stage(
              PluginContext::new(),
              ProcessAssetsArgs {
                compilation: args.compilation,
              },
            )
            .await?;
        }
      };
    }
    run_stage!(process_assets_stage_additional);
    run_stage!(process_assets_stage_pre_process);
    run_stage!(process_assets_stage_additions);
    run_stage!(process_assets_stage_none);
    run_stage!(process_assets_stage_optimize_size);
    run_stage!(process_assets_stage_dev_tooling);
    run_stage!(process_assets_stage_optimize_inline);
    run_stage!(process_assets_stage_summarize);
    run_stage!(process_assets_stage_optimize_hash);
    run_stage!(process_assets_stage_report);
    Ok(())
  }

  #[instrument(name = "plugin:make", skip_all)]
  pub async fn make(&self, compilation: &Compilation) -> PluginMakeHookOutput {
    for plugin in &self.plugins {
      plugin.make(PluginContext::new(), compilation).await?;
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
  #[instrument(name = "plugin:optimize_chunks", skip_all)]
  pub async fn optimize_chunks(&mut self, compilation: &mut Compilation) -> Result<()> {
    for plugin in &mut self.plugins {
      plugin
        .optimize_chunks(PluginContext::new(), OptimizeChunksArgs { compilation })
        .await?;
    }
    Ok(())
  }

  #[instrument(name = "plugin:optimize_modules", skip_all)]
  pub async fn optimize_modules(&mut self, compilation: &mut Compilation) -> Result<()> {
    for plugin in &mut self.plugins {
      plugin.optimize_modules(compilation).await?;
    }
    Ok(())
  }

  #[instrument(name = "plugin:optimize_chunk_modules", skip_all)]
  pub async fn optimize_chunk_modules(&mut self, compilation: &mut Compilation) -> Result<()> {
    for plugin in &mut self.plugins {
      plugin
        .optimize_chunk_modules(OptimizeChunksArgs { compilation })
        .await?;
    }
    Ok(())
  }

  #[instrument(name = "plugin:finish_modules", skip_all)]
  pub async fn finish_modules(&mut self, modules: &mut Compilation) -> Result<()> {
    for plugin in &mut self.plugins {
      plugin.finish_modules(modules).await?;
    }
    Ok(())
  }

  pub async fn resolve_loader(
    &self,
    compiler_options: &CompilerOptions,
    context: &Path,
    resolver: &Resolver,
    loader_request: &str,
  ) -> Result<Option<BoxLoader>> {
    for plugin in &self.plugins {
      if let Some(loader) = plugin
        .resolve_loader(compiler_options, context, resolver, loader_request)
        .await?
      {
        return Ok(Some(loader));
      };
    }

    Ok(None)
  }

  pub async fn before_loaders(&self, module: &mut NormalModule) -> Result<()> {
    for plugin in &self.plugins {
      plugin.before_loaders(module).await?;
    }
    Ok(())
  }

  pub async fn build_module(&self, module: &mut dyn Module) -> Result<()> {
    for plugin in &self.plugins {
      plugin.build_module(module).await?;
    }
    Ok(())
  }

  pub async fn succeed_module(&self, module: &dyn Module) -> Result<()> {
    for plugin in &self.plugins {
      plugin.succeed_module(module).await?;
    }
    Ok(())
  }

  #[instrument(name = "plugin:module_ids", skip_all)]
  pub fn module_ids(&mut self, compilation: &mut Compilation) -> Result<()> {
    for plugin in &mut self.plugins {
      plugin.module_ids(compilation)?;
    }
    Ok(())
  }

  #[instrument(name = "plugin:chunk_ids", skip_all)]
  pub fn chunk_ids(&mut self, compilation: &mut Compilation) -> Result<()> {
    for plugin in &mut self.plugins {
      plugin.chunk_ids(compilation)?;
    }
    Ok(())
  }

  #[instrument(name = "plugin:emit", skip_all)]
  pub async fn emit(&mut self, compilation: &mut Compilation) -> Result<()> {
    for plugin in &mut self.plugins {
      plugin.emit(compilation).await?;
    }
    Ok(())
  }

  #[instrument(name = "plugin:after_emit", skip_all)]
  pub async fn after_emit(&mut self, compilation: &mut Compilation) -> Result<()> {
    for plugin in &mut self.plugins {
      plugin.after_emit(compilation).await?;
    }
    Ok(())
  }
}
