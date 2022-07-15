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
  pub fn try_to_string(&self) -> Result<String> {
    match self {
      Content::String(s) => Ok(s.to_owned()),
      Content::Buffer(b) => String::from_utf8(b.to_owned()).map_err(anyhow::Error::from),
    }
  }

  /// # Safety
  ///
  /// This function is unsafe because it does not check that the bytes passed
  /// to it are valid UTF-8. If this constraint is violated, it may cause
  /// memory unsafety issues with future users of the `String`, as the rest of
  /// the standard library assumes that `String`s are valid UTF-8.
  pub unsafe fn to_string_unchecked(&self) -> String {
    match self {
      Content::String(s) => s.to_owned(),
      Content::Buffer(b) => String::from_utf8_unchecked(b.to_owned()),
    }
  }

  pub fn try_into_string(self) -> Result<String> {
    match self {
      Content::String(s) => Ok(s),
      Content::Buffer(b) => String::from_utf8(b).map_err(anyhow::Error::from),
    }
  }

  /// # Safety
  ///
  /// This function is unsafe because it does not check that the bytes passed
  /// to it are valid UTF-8. If this constraint is violated, it may cause
  /// memory unsafety issues with future users of the `String`, as the rest of
  /// the standard library assumes that `String`s are valid UTF-8.
  pub unsafe fn into_string_unchecked(self) -> String {
    match self {
      Content::String(s) => s,
      Content::Buffer(b) => String::from_utf8_unchecked(b),
    }
  }

  pub fn as_bytes(&self) -> &[u8] {
    match self {
      Content::String(s) => s.as_bytes(),
      Content::Buffer(b) => b,
    }
  }

  /// Converts the content to a mutable byte slice
  ///
  /// # Safety
  ///
  /// The caller must ensure that the content of the slice is valid UTF-8
  /// before the borrow ends and the underlying `str` is used.
  ///
  /// Use of a `str` whose contents are not valid UTF-8 is undefined behavior.
  pub unsafe fn as_bytes_mut(&mut self) -> &mut [u8] {
    match self {
      Content::String(s) => s.as_bytes_mut(),
      Content::Buffer(b) => b,
    }
  }

  pub fn into_bytes(self) -> Vec<u8> {
    match self {
      Content::String(s) => s.into_bytes(),
      Content::Buffer(b) => b,
    }
  }
}

impl TryFrom<Content> for String {
  type Error = anyhow::Error;

  fn try_from(content: Content) -> Result<Self, Self::Error> {
    content.try_into_string()
  }
}

impl TryFrom<&Content> for String {
  type Error = anyhow::Error;

  fn try_from(content: &Content) -> Result<Self, Self::Error> {
    content.try_to_string()
  }
}

impl From<Content> for Vec<u8> {
  fn from(content: Content) -> Self {
    content.into_bytes()
  }
}

impl From<&Content> for Vec<u8> {
  fn from(content: &Content) -> Self {
    content.as_bytes().to_vec()
  }
}

impl From<&str> for Content {
  fn from(s: &str) -> Self {
    Self::String(s.to_owned())
  }
}

impl From<String> for Content {
  fn from(s: String) -> Self {
    Self::String(s)
  }
}

impl From<&[u8]> for Content {
  fn from(buf: &[u8]) -> Self {
    Self::Buffer(buf.to_vec())
  }
}

impl From<Vec<u8>> for Content {
  fn from(buf: Vec<u8>) -> Self {
    Self::Buffer(buf)
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
