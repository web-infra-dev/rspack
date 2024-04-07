use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;

use rspack_error::Diagnostic;
use rspack_hash::RspackHash;
use rspack_loader_runner::ResourceData;
use rspack_sources::BoxSource;
use rustc_hash::FxHashSet as HashSet;

use crate::{
  BoxModule, Chunk, ChunkInitFragments, ChunkUkey, Compilation, Context, ContextModuleFactory,
  DependencyCategory, DependencyType, ErrorSpan, FactoryMeta, ModuleIdentifier,
  NormalModuleFactory, Resolve, RuntimeGlobals,
};

#[derive(Debug)]
pub struct ProcessAssetsArgs<'me> {
  pub compilation: &'me mut Compilation,
}

#[derive(Debug)]
pub struct AssetEmittedInfo {
  pub source: BoxSource,
  pub output_path: PathBuf,
  pub target_path: PathBuf,
}

#[derive(Debug)]
pub struct ContentHashArgs<'c> {
  pub chunk_ukey: ChunkUkey,
  pub compilation: &'c Compilation,
}

impl<'me> ContentHashArgs<'me> {
  pub fn chunk(&self) -> &Chunk {
    self.compilation.chunk_by_ukey.expect_get(&self.chunk_ukey)
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
    self.compilation.chunk_by_ukey.expect_get(&self.chunk_ukey)
  }
}

#[derive(Debug, Clone)]
pub struct RenderManifestArgs<'me> {
  pub chunk_ukey: ChunkUkey,
  pub compilation: &'me Compilation,
}

impl<'me> RenderManifestArgs<'me> {
  pub fn chunk(&self) -> &Chunk {
    self.compilation.chunk_by_ukey.expect_get(&self.chunk_ukey)
  }
}

#[derive(Debug)]
pub struct NormalModuleCreateData {
  pub raw_request: String,
  pub request: String,
  pub user_request: String,
  pub resource_resolve_data: ResourceData,
  pub match_resource: Option<String>,
}

#[derive(Debug)]
pub struct AfterResolveArgs<'a> {
  pub request: &'a str,
  pub context: &'a str,
  pub file_dependencies: &'a HashSet<PathBuf>,
  pub context_dependencies: &'a HashSet<PathBuf>,
  pub missing_dependencies: &'a HashSet<PathBuf>,
  pub factory_meta: &'a FactoryMeta,
  pub diagnostics: &'a mut Vec<Diagnostic>,
  pub create_data: Option<NormalModuleCreateData>,
}

#[derive(Debug)]
pub struct ResolveArgs<'a> {
  pub importer: Option<&'a ModuleIdentifier>,
  pub issuer: Option<&'a str>,
  pub context: Context,
  pub specifier: &'a str,
  pub dependency_type: &'a DependencyType,
  pub dependency_category: &'a DependencyCategory,
  pub span: Option<ErrorSpan>,
  pub resolve_options: Option<Box<Resolve>>,
  pub resolve_to_context: bool,
  pub optional: bool,
  pub file_dependencies: &'a mut HashSet<PathBuf>,
  pub missing_dependencies: &'a mut HashSet<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct LoadArgs<'a> {
  pub uri: &'a str,
}

#[derive(Debug)]
pub struct OptimizeChunksArgs<'me> {
  pub compilation: &'me mut Compilation,
}

#[derive(Debug)]
pub struct AdditionalChunkRuntimeRequirementsArgs<'a> {
  pub compilation: &'a mut Compilation,
  pub chunk: &'a ChunkUkey,
  pub runtime_requirements: &'a mut RuntimeGlobals,
}

#[derive(Debug)]
pub struct RuntimeRequirementsInTreeArgs<'a> {
  pub compilation: &'a mut Compilation,
  pub chunk: &'a ChunkUkey,
  pub runtime_requirements: &'a RuntimeGlobals,
  pub runtime_requirements_mut: &'a mut RuntimeGlobals,
}

#[derive(Debug)]
pub struct AdditionalModuleRequirementsArgs<'a> {
  pub compilation: &'a mut Compilation,
  pub module_identifier: &'a ModuleIdentifier,
  pub runtime_requirements: &'a RuntimeGlobals,
  pub runtime_requirements_mut: &'a mut RuntimeGlobals,
}

impl<'me> AdditionalChunkRuntimeRequirementsArgs<'me> {
  pub fn chunk(&self) -> &Chunk {
    self.compilation.chunk_by_ukey.expect_get(self.chunk)
  }
}

#[derive(Debug)]
pub struct RenderChunkArgs<'a> {
  pub compilation: &'a Compilation,
  pub chunk_ukey: &'a ChunkUkey,
  pub module_source: BoxSource,
}

impl<'me> RenderChunkArgs<'me> {
  pub fn chunk(&self) -> &Chunk {
    self.compilation.chunk_by_ukey.expect_get(self.chunk_ukey)
  }
}

#[derive(Debug)]
pub struct RenderModuleContentArgs<'a> {
  pub module_source: BoxSource,
  pub chunk_init_fragments: ChunkInitFragments,
  pub compilation: &'a Compilation,
  pub module: &'a BoxModule,
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
    self.compilation.chunk_by_ukey.expect_get(self.chunk)
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
    self.compilation.chunk_by_ukey.expect_get(self.chunk)
  }
}

pub struct JsChunkHashArgs<'a> {
  pub chunk_ukey: &'a ChunkUkey,
  pub compilation: &'a Compilation,
  pub hasher: &'a mut RspackHash,
}

impl<'me> JsChunkHashArgs<'me> {
  pub fn chunk(&self) -> &Chunk {
    self.compilation.chunk_by_ukey.expect_get(self.chunk_ukey)
  }
}

#[derive(Debug)]
pub struct CompilationParams {
  pub normal_module_factory: Arc<NormalModuleFactory>,
  pub context_module_factory: Arc<ContextModuleFactory>,
}
