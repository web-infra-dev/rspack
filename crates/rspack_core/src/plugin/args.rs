use crate::ast::css::Ast as CssAst;
use crate::ast::javascript::Ast as JsAst;
use crate::{
  Chunk, ChunkUkey, Compilation, Dependency, ErrorSpan, ModuleIdentifier, Resolve, ResolveKind,
  SharedPluginDriver, Stats,
};
use hashbrown::HashSet;
use rspack_error::{internal_error, Error, Result};
use std::fmt::Debug;

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

#[derive(Debug)]
pub struct ContentHashArgs<'me> {
  pub chunk_ukey: ChunkUkey,
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

#[derive(Debug, Clone)]
pub struct FactorizeArgs<'me> {
  pub dependency: &'me Dependency,
  pub plugin_driver: &'me SharedPluginDriver,
}

#[derive(Debug, Clone)]
pub struct ModuleArgs {
  pub indentfiler: ModuleIdentifier,
  pub kind: ResolveKind,
  // lazy compilation visit module
  pub lazy_visit_modules: std::collections::HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct ResolveArgs<'a> {
  pub importer: Option<&'a str>,
  pub specifier: &'a str,
  pub kind: ResolveKind,
  pub span: Option<ErrorSpan>,
  pub resolve_options: Option<Resolve>,
}

#[derive(Debug, Clone)]
pub struct LoadArgs<'a> {
  pub uri: &'a str,
}

/**
 *  AST used in first class Module
 */
#[derive(Debug, Clone, Hash)]
pub enum ModuleAst {
  JavaScript(JsAst),
  Css(CssAst),
}

impl ModuleAst {
  pub fn try_into_javascript(self) -> Result<JsAst> {
    match self {
      ModuleAst::JavaScript(program) => Ok(program),
      ModuleAst::Css(_) => Err(Error::InternalError(internal_error!("Failed".to_owned()))),
    }
  }

  pub fn try_into_css(self) -> Result<CssAst> {
    match self {
      ModuleAst::Css(stylesheet) => Ok(stylesheet),
      ModuleAst::JavaScript(_) => Err(Error::InternalError(internal_error!("Failed".to_owned()))),
    }
  }

  pub fn as_javascript(&self) -> Option<&JsAst> {
    match self {
      ModuleAst::JavaScript(program) => Some(program),
      ModuleAst::Css(_) => None,
    }
  }

  pub fn as_css(&self) -> Option<&CssAst> {
    match self {
      ModuleAst::Css(stylesheet) => Some(stylesheet),
      ModuleAst::JavaScript(_) => None,
    }
  }
}

#[derive(Debug)]
pub struct OptimizeChunksArgs<'me> {
  pub compilation: &'me mut Compilation,
}

#[derive(Debug)]
pub struct DoneArgs<'s, 'c: 's> {
  pub stats: &'s mut Stats<'c>,
}

#[derive(Debug)]
pub struct CompilationArgs<'c> {
  pub compilation: &'c mut Compilation,
}

#[derive(Debug)]
pub struct ThisCompilationArgs<'c> {
  pub this_compilation: &'c mut Compilation,
}

#[derive(Debug)]
pub struct AdditionalChunkRuntimeRequirementsArgs<'a> {
  pub compilation: &'a mut Compilation,
  pub chunk: &'a ChunkUkey,
  pub runtime_requirements: &'a mut HashSet<String>,
  // TODO context
}

#[derive(Debug)]
pub struct RenderChunkArgs<'a> {
  pub compilation: &'a Compilation,
  pub chunk_ukey: &'a ChunkUkey,
}

impl<'me> RenderChunkArgs<'me> {
  pub fn chunk(&self) -> &Chunk {
    self
      .compilation
      .chunk_by_ukey
      .get(self.chunk_ukey)
      .expect("chunk should exsit in chunk_by_ukey")
  }
}
