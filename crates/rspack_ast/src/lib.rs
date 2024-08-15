mod ast;

pub use crate::ast::javascript;
use crate::ast::javascript::Ast as JsAst;

/**
 *  AST used in first class Module
 */
#[derive(Debug, Clone, Hash)]
pub enum RspackAst {
  JavaScript(JsAst),
}

impl RspackAst {
  pub fn as_javascript(&self) -> Option<&JsAst> {
    match self {
      RspackAst::JavaScript(program) => Some(program),
    }
  }
}
