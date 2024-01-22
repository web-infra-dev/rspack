mod ast;

pub use crate::ast::css;
use crate::ast::css::Ast as CssAst;
pub use crate::ast::javascript;
use crate::ast::javascript::Ast as JsAst;

/**
 *  AST used in first class Module
 */
#[derive(Debug, Clone, Hash)]
pub enum RspackAst {
  JavaScript(JsAst),
  Css(CssAst),
}

impl RspackAst {
  pub fn as_javascript(&self) -> Option<&JsAst> {
    match self {
      RspackAst::JavaScript(program) => Some(program),
      RspackAst::Css(_) => None,
    }
  }

  pub fn as_css(&self) -> Option<&CssAst> {
    match self {
      RspackAst::Css(stylesheet) => Some(stylesheet),
      RspackAst::JavaScript(_) => None,
    }
  }
}
