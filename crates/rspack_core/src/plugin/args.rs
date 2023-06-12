use std::fmt::Debug;
use std::path::{Path, PathBuf};

use rspack_error::{internal_error, Result};
use rspack_hash::RspackHash;
use rspack_sources::BoxSource;
use rustc_hash::FxHashSet as HashSet;

use crate::ast::css::Ast as CssAst;
use crate::ast::javascript::Ast as JsAst;
use crate::{
  Chunk, ChunkUkey, Compilation, Context, DependencyCategory, DependencyType, ErrorSpan,
  FactoryMeta, ModuleDependency, ModuleIdentifier, Resolve, RuntimeGlobals, SharedPluginDriver,
  Stats,
};
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
pub struct AssetEmittedArgs<'me> {
  pub filename: &'me str,
  pub source: BoxSource,
  pub output_path: &'me Path,
  pub compilation: &'me Compilation,
  pub target_path: &'me Path,
}

#[derive(Debug)]
pub struct ContentHashArgs<'c> {
  pub chunk_ukey: ChunkUkey,
  pub compilation: &'c Compilation,
}

impl<'me> ContentHashArgs<'me> {
  pub fn chunk(&self) -> &Chunk {
    self
      .compilation
      .chunk_by_ukey
      .get(&self.chunk_ukey)
      .expect("chunk should exist in chunk_by_ukey")
  }
}

#[derive(Debug)]
pub struct ChunkHashArgs<'c> {
  pub chunk_ukey: ChunkUkey,
  pub compilation: &'c Compilation,
  pub hasher: &'c mut RspackHash,
}

impl<'me> ChunkHashArgs<'me> {
  pub fn chunk(&self) -> &Chunk {
    self
      .compilation
      .chunk_by_ukey
      .get(&self.chunk_ukey)
      .expect("chunk should exist in chunk_by_ukey")
  }
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
      .expect("chunk should exist in chunk_by_ukey")
  }
}

#[derive(Debug, Clone)]
pub struct FactorizeArgs<'me> {
  pub context: &'me Context,
  pub dependency: &'me dyn ModuleDependency,
  pub plugin_driver: &'me SharedPluginDriver,
}

#[derive(Debug, Clone)]
pub struct ModuleArgs {
  pub indentfiler: ModuleIdentifier,
  pub dependency_type: DependencyType,
  // lazy compilation visit module
  pub lazy_visit_modules: std::collections::HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct NormalModuleBeforeResolveArgs {
  pub request: String,
  pub context: String,
}
#[derive(Debug, Clone)]
pub struct NormalModuleAfterResolveArgs<'a> {
  pub request: &'a str,
  pub context: &'a str,
  pub file_dependencies: &'a HashSet<PathBuf>,
  pub context_dependencies: &'a HashSet<PathBuf>,
  pub missing_dependencies: &'a HashSet<PathBuf>,
  pub factory_meta: &'a FactoryMeta,
}

#[derive(Debug)]
pub struct ResolveArgs<'a> {
  pub importer: Option<&'a ModuleIdentifier>,
  pub context: Context,
  pub specifier: &'a str,
  pub dependency_type: &'a DependencyType,
  pub dependency_category: &'a DependencyCategory,
  pub span: Option<ErrorSpan>,
  pub resolve_options: Option<Resolve>,
  pub resolve_to_context: bool,
  pub optional: bool,
  pub file_dependencies: &'a mut HashSet<PathBuf>,
  pub missing_dependencies: &'a mut HashSet<PathBuf>,
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
      ModuleAst::Css(_) => Err(internal_error!("Failed")),
    }
  }

  pub fn try_into_css(self) -> Result<CssAst> {
    match self {
      ModuleAst::Css(stylesheet) => Ok(stylesheet),
      ModuleAst::JavaScript(_) => Err(internal_error!("Failed")),
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
  pub runtime_requirements: &'a mut RuntimeGlobals,
  // TODO context
}

#[derive(Debug)]
pub struct RenderChunkArgs<'a> {
  pub compilation: &'a Compilation,
  pub chunk_ukey: &'a ChunkUkey,
  pub module_source: BoxSource,
}

#[derive(Debug)]
pub struct ChunkAssetArgs<'a> {
  pub chunk: &'a Chunk,
  pub filename: &'a str,
}

impl<'me> RenderChunkArgs<'me> {
  pub fn chunk(&self) -> &Chunk {
    self
      .compilation
      .chunk_by_ukey
      .get(self.chunk_ukey)
      .expect("chunk should exist in chunk_by_ukey")
  }
}

#[derive(Debug)]
pub struct RenderModuleContentArgs<'a> {
  pub module_source: &'a BoxSource,
  pub compilation: &'a Compilation,
}

#[derive(Debug)]
pub struct RenderStartupArgs<'a> {
  // pub module_source: &'a BoxSource,
  pub compilation: &'a Compilation,
  pub chunk: &'a ChunkUkey,
  pub module: ModuleIdentifier,
  pub source: BoxSource,
}

impl<'me> RenderStartupArgs<'me> {
  pub fn chunk(&self) -> &Chunk {
    self
      .compilation
      .chunk_by_ukey
      .get(self.chunk)
      .expect("chunk should exist in chunk_by_ukey")
  }
}

#[derive(Debug)]
pub struct RenderArgs<'a> {
  pub source: &'a BoxSource,
  pub chunk: &'a ChunkUkey,
  pub compilation: &'a Compilation,
}

impl<'me> RenderArgs<'me> {
  pub fn chunk(&self) -> &Chunk {
    self
      .compilation
      .chunk_by_ukey
      .get(self.chunk)
      .expect("chunk should exist in chunk_by_ukey")
  }
}

pub struct JsChunkHashArgs<'a> {
  pub chunk_ukey: &'a ChunkUkey,
  pub compilation: &'a Compilation,
  pub hasher: &'a mut RspackHash,
}

impl<'me> JsChunkHashArgs<'me> {
  pub fn chunk(&self) -> &Chunk {
    self
      .compilation
      .chunk_by_ukey
      .get(self.chunk_ukey)
      .expect("chunk should exist in chunk_by_ukey")
  }
}
