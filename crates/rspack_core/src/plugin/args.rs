use crate::{Compilation, ResolveKind};
use anyhow::Result;
use std::fmt::Debug;
use swc_css::ast::Stylesheet;
use swc_ecma_ast as ast;

#[derive(Debug)]
pub struct ParseModuleArgs<'a> {
  pub uri: &'a str,
  pub source: Option<Content>,
  pub ast: Option<ModuleAst>,
}

#[derive(Debug, Clone)]
pub struct RenderManifestArgs<'me> {
  pub chunk_id: &'me str,
  pub compilation: &'me Compilation,
}

#[derive(Debug, Clone)]
pub struct ResolveArgs<'a> {
  pub importer: Option<&'a str>,
  pub specifier: &'a str,
  pub kind: ResolveKind,
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
#[derive(Clone)]
pub enum Content {
  String(String),
  Buffer(Vec<u8>),
}

impl Content {
  pub fn as_string(&self) -> Result<String> {
    match self {
      Content::String(s) => Ok(s.to_owned()),
      Content::Buffer(b) => String::from_utf8(b.clone()).map_err(anyhow::Error::from),
    }
  }

  pub fn as_string_unchecked(&self) -> String {
    self.as_string().unwrap()
  }

  pub fn as_bytes(&self) -> Result<Vec<u8>> {
    match self {
      Content::String(s) => Ok(s.as_bytes().to_vec()),
      Content::Buffer(b) => Ok(b.clone()),
    }
  }

  pub fn as_bytes_unchecked(&self) -> Vec<u8> {
    self.as_bytes().unwrap()
  }
}

impl Debug for Content {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut content = f.debug_struct("Content");

    match self {
      Self::String(s) => content
        .field("String", &s[0..usize::min(s.len(), 20)].to_owned())
        .finish(),
      Self::Buffer(_) => content.field("Buffer", &{ .. }).finish(),
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
