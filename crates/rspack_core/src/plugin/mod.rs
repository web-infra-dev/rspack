use anyhow::{Context, Result};
use std::fmt::Debug;

use crate::{
  parse_file, Chunk, Loader, NormalizedBundleOptions, OutputChunk, OutputChunkSourceMap,
};
use async_trait::async_trait;
use rspack_swc::swc_ecma_ast as ast;

mod context;
pub use context::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum ImportKind {
  Require,
  Import,
  DynamicImport,
}

#[derive(Debug, Clone)]
pub struct ResolveArgs {
  pub id: String,
  pub importer: Option<String>,
  pub kind: ImportKind,
}
#[derive(Debug, Clone)]
pub enum RspackAst {
  JavaScript(ast::Module),
  Css(ast::Module), // I'm not sure what the final ast is, so just take placehold
}
#[derive(Debug)]
pub struct TransformArgs<'a> {
  pub uri: String,
  pub ast: Option<RspackAst>,
  pub code: String,
  pub loader: &'a mut Option<Loader>,
}
#[derive(Clone, Default, Debug)]
pub struct TransformResult {
  pub code: String,
  pub ast: Option<RspackAst>,
}

impl<'a> From<TransformArgs<'a>> for TransformResult {
  fn from(args: TransformArgs) -> TransformResult {
    TransformResult {
      code: args.code,
      ast: args.ast,
    }
  }
}
impl From<String> for TransformResult {
  fn from(code: String) -> TransformResult {
    TransformResult { code, ast: None }
  }
}
impl From<ast::Module> for TransformResult {
  fn from(ast: ast::Module) -> TransformResult {
    TransformResult {
      code: "".to_string(), // no need code, since we have ast
      ast: Some(RspackAst::JavaScript(ast)),
    }
  }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct OnResolveResult {
  pub uri: String,
  pub external: bool,
}

#[derive(Debug, Clone)]
pub struct LoadArgs {
  pub id: String,
  pub kind: ImportKind,
}

pub struct RenderChunkArgs<'a> {
  pub chunk: &'a Chunk,
  pub file_name: String,
  pub entry: String,
  pub code: String,
  pub map: OutputChunkSourceMap,
}

pub type PluginBuildStartHookOutput = Result<()>;
pub type PluginBuildEndHookOutput = Result<()>;
pub type PluginLoadHookOutput = Result<Option<LoadedSource>>;
pub type PluginResolveHookOutput = Result<Option<OnResolveResult>>;
pub type PluginTransformAstHookOutput = Result<ast::Module>;
pub type PluginParseOutput = Result<RspackAst>;
pub type PluginGenerateOutput = Result<String>;
pub type PluginTransformHookOutput = Result<TransformResult>;
pub type PluginTapGeneratedChunkHookOutput = Result<()>;
pub type PluginRenderChunkHookOutput = Result<OutputChunk>;

#[async_trait]
pub trait Plugin: Sync + Send + Debug {
  fn name(&self) -> &'static str;
  #[inline]
  fn transform_include(&self, _: &str, _loader: &Option<Loader>) -> bool {
    false
  }
  #[inline]
  fn need_build_start(&self) -> bool {
    true
  }

  #[inline]
  fn need_build_end(&self) -> bool {
    true
  }

  #[inline]
  fn need_resolve(&self) -> bool {
    true
  }

  #[inline]
  fn need_load(&self) -> bool {
    true
  }
  #[inline]
  fn reuse_ast(&self) -> bool {
    true
  }

  #[inline]
  fn need_tap_generated_chunk(&self) -> bool {
    true
  }

  #[inline]
  fn need_render_chunk(&self) -> bool {
    true
  }

  fn generate(&self, ast: &Option<RspackAst>) -> PluginGenerateOutput {
    let ast = ast.as_ref().context("call generate when ast is empty")?;
    match ast {
      RspackAst::JavaScript(_ast) => Err(anyhow::anyhow!("js ast codegen not supported yet")),
      RspackAst::Css(_ast) => Err(anyhow::anyhow!("css ast codegen not supported yet ")),
    }
  }
  fn parse(&self, uri: &str, code: &str, loader: &Option<Loader>) -> PluginParseOutput {
    match loader {
      Some(l) if l.is_js_family() => {
        let module = parse_file(code, uri, l).expect_module();
        Ok(RspackAst::JavaScript(module))
      }
      _ => Err(anyhow::anyhow!("parse for {:?} not supported yet", uri)),
    }
  }
  #[inline]
  async fn build_start(&self, _ctx: &PluginContext) -> PluginBuildStartHookOutput {
    Ok(())
  }
  #[inline]
  async fn build_end(&self, _ctx: &PluginContext) -> PluginBuildEndHookOutput {
    Ok(())
  }

  #[inline]
  async fn resolve(&self, _ctx: &PluginContext, _args: &ResolveArgs) -> PluginResolveHookOutput {
    Ok(None)
  }

  #[inline]
  async fn load(&self, _ctx: &PluginContext, _args: &LoadArgs) -> PluginLoadHookOutput {
    Ok(None)
  }

  #[inline]
  fn transform(&self, _ctx: &PluginContext, args: TransformArgs) -> PluginTransformHookOutput {
    Ok(TransformResult {
      code: args.code,
      ast: args.ast,
    })
  }

  #[inline]
  fn optimize_ast(
    &self,
    _ctx: &PluginContext,
    _uri: &str,
    ast: ast::Module,
  ) -> PluginTransformAstHookOutput {
    Ok(ast)
  }

  fn render_chunk<'a>(
    &self,
    _ctx: &PluginContext,
    args: RenderChunkArgs<'a>,
  ) -> PluginRenderChunkHookOutput {
    Ok(OutputChunk {
      code: args.code,
      map: args.map,
      file_name: args.file_name,
      entry: args.entry,
    })
  }

  #[inline]
  fn tap_generated_chunk(
    &self,
    _ctx: &PluginContext,
    _chunk: &Chunk,
    _bundle_options: &NormalizedBundleOptions,
  ) -> PluginTapGeneratedChunkHookOutput {
    Ok(())
  }
}

#[derive(Debug, Default, Clone)]
pub struct LoadedSource {
  pub content: Option<String>,
  pub loader: Option<Loader>,
}

impl LoadedSource {
  pub fn new(content: String) -> Self {
    Self {
      content: Some(content),
      ..Default::default()
    }
  }
  pub fn with_loader(content: String, loader: Loader) -> Self {
    Self {
      content: Some(content),
      loader: Some(loader),
    }
  }
}
