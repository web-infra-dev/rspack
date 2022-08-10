use std::{collections::HashMap, sync::Arc};

use tracing::instrument;

use crate::{
  ApplyContext, BoxModule, BoxedParser, CompilerOptions, LoadArgs, ModuleType,
  NormalModuleFactoryContext, ParseModuleArgs, Plugin, PluginContext, PluginLoadHookOutput,
  PluginProcessAssetsOutput, PluginRenderManifestHookOutput, PluginRenderRuntimeHookOutput,
  PluginResolveHookOutput, PluginTransformOutput, ProcessAssetsArgs, RenderManifestArgs,
  RenderRuntimeArgs, ResolveArgs, Resolver, TransformArgs, TransformResult,
};
use anyhow::Context;
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
  ) -> PluginLoadHookOutput {
    for plugin in &self.plugins {
      let content = plugin
        .load(PluginContext::with_context(job_ctx), args.clone())
        .await?;
      if content.is_some() {
        return Ok(content);
      }
    }
    Ok(None)
  }
  #[instrument(skip_all)]
  pub fn transform(
    &self,
    args: TransformArgs,
    job_ctx: &mut NormalModuleFactoryContext,
  ) -> PluginTransformOutput {
    let mut transformed_result = TransformResult {
      content: args.content,
      ast: args.ast,
    };
    for plugin in &self.plugins {
      if plugin.transform_include(args.uri) {
        tracing::debug!("running transform:{}", plugin.name());
        let x = transformed_result;
        let mut content = x.content;
        let mut ast = x.ast;
        // ast take precedence over code
        // if prev loader set ast and current loader can't reuse_ast then we have to codegen code for current loader
        if !plugin.reuse_ast() && ast.is_some() {
          content = Some(plugin.generate(&ast)?);
        }
        // if previous not set ast and current loader want to use ast, so we must parse it for loader
        if ast.is_none() && plugin.reuse_ast() {
          let y = plugin.parse(
            args.uri,
            content
              .as_ref()
              .with_context(|| format!("ast and code is both none for {}", &args.uri))?,
          )?;
          ast = Some(y)
        }
        let args = TransformArgs {
          uri: args.uri,
          ast,
          content,
        };
        let res = plugin.transform(PluginContext::with_context(job_ctx), args)?;
        transformed_result = res;
      }
    }
    Ok(transformed_result)
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
