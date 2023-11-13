mod ast;

use rspack_error::{internal_error, Result};

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
  pub fn try_into_javascript(self) -> Result<JsAst> {
    match self {
      RspackAst::JavaScript(program) => Ok(program),
      RspackAst::Css(_) => Err(internal_error!("Failed to cast `CSS` AST to `JavaScript`")),
    }
  }

  pub fn try_into_css(self) -> Result<CssAst> {
    match self {
      RspackAst::Css(stylesheet) => Ok(stylesheet),
      RspackAst::JavaScript(_) => Err(internal_error!("Failed to cast `JavaScript` AST to `CSS`")),
    }
  }

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
