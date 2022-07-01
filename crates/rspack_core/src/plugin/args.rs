use crate::{Compilation, ResolveKind};
use swc_css::ast::Stylesheet;
use swc_ecma_ast as ast;

#[derive(Debug)]
pub struct ParseModuleArgs<'a> {
  pub uri: &'a str,
  pub source: Option<String>,
  pub ast: Option<RspackAst>,
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
#[derive(Debug, Clone)]
pub enum RspackAst {
  JavaScript(ast::Program),
  Css(Stylesheet), // I'm not sure what the final ast is, so just take placehold
}

#[derive(Clone, Default, Debug)]
pub struct TransformArgs<'a> {
  pub uri: &'a str,
  pub code: Option<String>,
  pub ast: Option<RspackAst>,
}

#[derive(Clone, Default, Debug)]
pub struct TransformResult {
  pub code: Option<String>,
  pub ast: Option<RspackAst>,
}
