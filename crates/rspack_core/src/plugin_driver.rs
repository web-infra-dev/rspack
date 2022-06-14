use std::{
  collections::HashSet,
  sync::{Arc, RwLock},
};

use anyhow::Result;
use futures::future::try_join_all;
use nodejs_resolver::Resolver;
use rspack_swc::swc_ecma_ast as ast;
use tracing::instrument;

use crate::{
  BundleContext, Chunk, LoadArgs, LoadedSource, Loader, NormalizedBundleOptions, OnResolveResult,
  OutputChunk, Plugin, PluginBuildEndHookOutput, PluginBuildStartHookOutput, PluginContext,
  PluginRenderChunkHookOutput, PluginTapGeneratedChunkHookOutput, PluginTransformAstHookOutput,
  PluginTransformHookOutput, RenderChunkArgs, ResolveArgs, TransformArgs, TransformResult,
};

#[derive(Debug)]
pub struct PluginDriver {
  pub plugins: Vec<Box<dyn Plugin>>,
  pub ctx: Arc<BundleContext>,
  pub resolver: Arc<Resolver>,
  build_start_hints: Vec<usize>,
  build_end_hints: Vec<usize>,
  resolve_id_hints: Vec<usize>,
  load_hints: Vec<usize>,
  tap_generated_chunk_hints: Vec<usize>,
  render_chunk_hints: Vec<usize>,
  pub resolved_entries: RwLock<Arc<HashSet<String>>>,
}

impl PluginDriver {
  /// TODO: Maybe we need to add some tracing here? It should not take too much time to init.
  pub fn new(
    plugins: Vec<Box<dyn Plugin>>,
    ctx: Arc<BundleContext>,
    resolver: Arc<Resolver>,
  ) -> Self {
    let mut build_start_hints: Vec<usize> = Vec::with_capacity(plugins.len());
    let mut build_end_hints = Vec::with_capacity(plugins.len());
    let mut resolve_id_hints = Vec::with_capacity(plugins.len());
    let mut load_hints = Vec::with_capacity(plugins.len());
    let mut tap_generated_chunk_hints = Vec::with_capacity(plugins.len());
    let mut render_chunk_hints = Vec::with_capacity(plugins.len());

    plugins.iter().enumerate().for_each(|(i, p)| {
      if p.need_build_start() {
        build_start_hints.push(i);
      }
      if p.need_build_end() {
        build_end_hints.push(i);
      }
      if p.need_load() {
        load_hints.push(i);
      }
      if p.need_resolve() {
        resolve_id_hints.push(i);
      }
      if p.need_tap_generated_chunk() {
        tap_generated_chunk_hints.push(i);
      }
      if p.need_render_chunk() {
        render_chunk_hints.push(i);
      }
    });
    Self {
      plugins,
      ctx,
      resolver,
      build_start_hints,
      build_end_hints,
      resolve_id_hints,
      load_hints,
      tap_generated_chunk_hints,
      render_chunk_hints,
      resolved_entries: Default::default(),
    }
  }

  #[instrument(skip_all)]
  pub async fn build_start(&self) -> PluginBuildStartHookOutput {
    let ctx = self.plugin_context();
    try_join_all(
      self
        .build_start_hints
        .iter()
        .map(|i| self.plugins[*i].build_start(&ctx)),
    )
    .await?;
    Ok(())
  }
  #[instrument(skip_all)]
  pub async fn build_end(&self) -> PluginBuildEndHookOutput {
    let ctx = self.plugin_context();
    let assets = &*ctx.assets().lock().unwrap().clone(); // can't solve the lifetime problem, so I have to clone
    try_join_all(
      self
        .build_end_hints
        .iter()
        .map(|i| self.plugins[*i].build_end(&ctx, assets)),
    )
    .await?;
    Ok(())
  }
  #[instrument(skip_all)]
  pub async fn resolve_id(&self, args: &ResolveArgs) -> Result<Option<OnResolveResult>> {
    for i in self.resolve_id_hints.iter() {
      let plugin = &self.plugins[*i];
      let res = plugin.resolve(&self.plugin_context(), args).await?;
      if res.is_some() {
        tracing::trace!("got load result of plugin {:?}", plugin.name());
        return Ok(res);
      }
    }
    Ok(None)
  }
  #[instrument(skip_all)]
  pub async fn load(&self, args: &LoadArgs) -> Result<Option<LoadedSource>> {
    for i in self.load_hints.iter() {
      let plugin = &self.plugins[*i];
      let res = plugin.load(&self.plugin_context(), args).await?;
      if res.is_some() {
        return Ok(res);
      }
    }
    Ok(None)
  }
  #[instrument(skip_all)]
  pub fn transform(
    &self,
    uri: &str,
    loader: &mut Option<Loader>,
    code: String,
  ) -> PluginTransformHookOutput {
    self.plugins.iter().fold(
      Ok(TransformResult { code, ast: None }),
      |transformed_result, plugin| {
        if plugin.transform_include(uri, loader) {
          let x = transformed_result?;
          let mut code = x.code;
          let mut ast = x.ast;
          // ast take precedence over code
          // if prev loader set ast and current loader can't reuse_ast then we have to codegen code for current loader
          if !plugin.reuse_ast() && ast.is_some() {
            code = plugin.generate(&ast)?;
          }
          // if previous not set ast and current loader want to use ast, so we must parse it for loader
          if ast.is_none() && plugin.reuse_ast() {
            let y = plugin.parse(uri, &code, loader)?;
            ast = Some(y)
          }
          let args = TransformArgs {
            uri: uri.to_string(),
            ast,
            code,
            loader,
          };
          let res = plugin.transform(&self.plugin_context(), args);
          res
        } else {
          transformed_result
        }
      },
    )
  }
  #[instrument(skip_all)]
  pub fn optimize_ast(
    &self,
    path: &str,
    ast: ast::Module,
    _loader: &Loader,
  ) -> PluginTransformAstHookOutput {
    self
      .plugins
      .iter()
      .fold(Ok(ast), |transformed_ast, plugin| {
        plugin.optimize_ast(&self.plugin_context(), path, transformed_ast?)
      })
  }

  #[instrument(skip_all)]
  pub fn render_chunk(
    &self,
    output_chunk: OutputChunk,
    chunk: &Chunk,
  ) -> PluginRenderChunkHookOutput {
    self
      .render_chunk_hints
      .iter()
      .fold(Ok(output_chunk), |rendered_chunk, i| {
        let rendered_chunk = rendered_chunk?;
        self.plugins[*i].render_chunk(
          &self.plugin_context(),
          RenderChunkArgs {
            code: rendered_chunk.code,
            map: rendered_chunk.map,
            file_name: rendered_chunk.file_name,
            entry: rendered_chunk.entry,
            chunk,
          },
        )
      })
  }

  #[instrument(skip_all)]
  pub fn tap_generated_chunk(
    &self,
    chunk: &Chunk,
    bundle_options: &NormalizedBundleOptions,
  ) -> PluginTapGeneratedChunkHookOutput {
    self
      .tap_generated_chunk_hints
      .iter()
      .try_for_each(|i| -> Result<()> {
        self.plugins[*i].tap_generated_chunk(&self.plugin_context(), chunk, bundle_options)
      })
  }
  pub fn module_parsed(&self, uri: &str) -> Result<()> {
    for ele in &self.plugins {
      ele.module_parsed(&self.plugin_context(), uri)?
    }
    Ok(())
  }
  fn plugin_context<'me>(&'me self) -> PluginContext<'me> {
    PluginContext::<'me>::new(
      &self.ctx.assets,
      self.ctx.compiler.clone(),
      self.ctx.options.clone(),
      self.resolved_entries.read().unwrap().clone(),
    )
  }
}
