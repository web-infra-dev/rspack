use crate::ast::javascript::Ast as JsAst;
use crate::{
  Chunk, ChunkUkey, Compilation, Dependency, ErrorSpan, ResolveKind, SharedPluginDriver, Stats,
};
use rspack_error::{Error, Result};
use rspack_loader_runner::Content;
use rspack_sources::RawSource;
use std::fmt::Debug;
use swc_css::ast::Stylesheet;
use swc_ecma_ast::Program as SwcProgram;

// #[derive(Debug)]
// pub struct ParseModuleArgs<'a> {
//   pub uri: &'a str,
//   pub options: Arc<CompilerOptions>,
//   pub source: BoxSource,
//   pub meta: Option<String>, // pub ast: Option<ModuleAst>,
// }

#[derive(Debug)]
pub struct ProcessAssetsArgs<'me> {
  pub compilation: &'me mut Compilation,
}

#[derive(Debug, Clone)]
pub struct RenderManifestArgs<'me> {
  pub chunk_ukey: ChunkUkey,
  pub compilation: &'me Compilation,
}

impl<'me> RenderManifestArgs<'me> {
  pub fn chunk(&self) -> &Chunk {
    self
      .compilation
      .chunk_by_ukey
      .get(&self.chunk_ukey)
      .expect("chunk should exsit in chunk_by_ukey")
  }
}

#[derive(Debug)]
pub struct RenderRuntimeArgs<'me> {
  pub sources: Vec<RawSource>,
  pub compilation: &'me Compilation,
}

#[derive(Debug, Clone)]
pub struct FactorizeAndBuildArgs<'me> {
  pub dependency: &'me Dependency,
  pub plugin_driver: &'me SharedPluginDriver,
}

#[derive(Debug, Clone)]
pub struct ResolveArgs<'a> {
  pub importer: Option<&'a str>,
  pub specifier: &'a str,
  pub kind: ResolveKind,
  pub span: Option<ErrorSpan>,
}

#[derive(Debug, Clone)]
pub struct LoadArgs<'a> {
  pub uri: &'a str,
}

/**
 * ast resused in transform hook
 */
#[derive(Debug, Clone)]
pub enum TransformAst {
  JavaScript(SwcProgram),
  Css(Stylesheet),
}

/**
 *  AST used in first class Module
 */
#[derive(Debug)]
pub enum ModuleAst {
  JavaScript(JsAst),
  Css(Stylesheet),
}

impl ModuleAst {
  pub fn try_into_javascript(self) -> Result<JsAst> {
    match self {
      ModuleAst::JavaScript(program) => Ok(program),
      ModuleAst::Css(_) => Err(Error::InternalError("Failed".to_owned())),
    }
  }

  pub fn try_into_css(self) -> Result<Stylesheet> {
    match self {
      ModuleAst::Css(stylesheet) => Ok(stylesheet),
      ModuleAst::JavaScript(_) => Err(Error::InternalError("Failed".to_owned())),
    }
  }

  pub fn as_javascript(&self) -> Option<&JsAst> {
    match self {
      ModuleAst::JavaScript(program) => Some(program),
      ModuleAst::Css(_) => None,
    }
  }

  pub fn as_css(&self) -> Option<&Stylesheet> {
    match self {
      ModuleAst::Css(stylesheet) => Some(stylesheet),
      ModuleAst::JavaScript(_) => None,
    }
  }
}

impl From<TransformAst> for ModuleAst {
  fn from(ast: TransformAst) -> ModuleAst {
    match ast {
      TransformAst::Css(_ast) => ModuleAst::Css(_ast),
      TransformAst::JavaScript(_ast) => ModuleAst::JavaScript(JsAst::new(_ast)),
    }
  }
}

#[derive(Clone, Default, Debug)]
pub struct TransformArgs<'a> {
  pub uri: &'a str,
  pub content: Option<Content>,
  pub ast: Option<TransformAst>,
}

#[derive(Clone, Default, Debug)]
pub struct TransformResult {
  pub content: Option<Content>,
  pub ast: Option<TransformAst>,
}

#[derive(Debug)]
pub struct OptimizeChunksArgs<'me> {
  pub compilation: &'me mut Compilation,
}

#[derive(Debug)]
pub struct DoneArgs<'s, 'c: 's> {
  pub stats: &'s mut Stats<'c>,
}
