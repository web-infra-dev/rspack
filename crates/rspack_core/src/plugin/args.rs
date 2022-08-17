use crate::{
  ChunkRid, Compilation, CompilerOptions, Dependency, ErrorSpan, PluginDriver, ResolveKind,
  RuntimeSourceNode,
};
use rspack_loader_runner::Content;
use std::{fmt::Debug, sync::Arc};
use swc_css::ast::Stylesheet;
use swc_ecma_ast as ast;

#[derive(Debug)]
pub struct ParseModuleArgs<'a> {
  pub uri: &'a str,
  pub options: Arc<CompilerOptions>,
  pub source: Content,
  pub meta: Option<String>, // pub ast: Option<ModuleAst>,
}

#[derive(Debug)]
pub struct ProcessAssetsArgs<'me> {
  pub compilation: &'me mut Compilation,
}

#[derive(Debug, Clone)]
pub struct RenderManifestArgs<'me> {
  pub chunk_rid: ChunkRid,
  pub compilation: &'me Compilation,
}

#[derive(Debug, Clone)]
pub struct RenderRuntimeArgs<'me> {
  pub sources: &'me Vec<RuntimeSourceNode>,
  pub compilation: &'me Compilation,
}

#[derive(Debug, Clone)]
pub struct FactorizeAndBuildArgs<'me> {
  pub dependency: &'me Dependency,
  pub plugin_driver: &'me Arc<PluginDriver>,
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
  JavaScript(ast::Program),
  Css(Stylesheet),
}

/**
 *  AST used in first class Module
 */
#[derive(Debug, Clone)]
pub enum ModuleAst {
  JavaScript(ast::Program),
  Css(Stylesheet),
}

impl From<TransformAst> for ModuleAst {
  fn from(ast: TransformAst) -> ModuleAst {
    match ast {
      TransformAst::Css(_ast) => ModuleAst::Css(_ast),
      TransformAst::JavaScript(_ast) => ModuleAst::JavaScript(_ast),
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
