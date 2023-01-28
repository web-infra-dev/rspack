pub mod css;
pub mod javascript;

use rspack_error::{internal_error, Error, Result};

#[derive(Debug, Clone, Hash)]
pub enum RspackAst {
  JavaScript(javascript::Ast),
  Css(css::Ast),
}

impl RspackAst {
  pub fn try_into_javascript(self) -> Result<javascript::Ast> {
    match self {
      Self::JavaScript(program) => Ok(program),
      Self::Css(_) => Err(Error::InternalError(internal_error!("Failed".to_owned()))),
    }
  }

  pub fn try_into_css(self) -> Result<css::Ast> {
    match self {
      Self::Css(stylesheet) => Ok(stylesheet),
      Self::JavaScript(_) => Err(Error::InternalError(internal_error!("Failed".to_owned()))),
    }
  }

  pub fn as_javascript(&self) -> Option<&javascript::Ast> {
    match self {
      Self::JavaScript(program) => Some(program),
      Self::Css(_) => None,
    }
  }

  pub fn as_css(&self) -> Option<&css::Ast> {
    match self {
      Self::Css(stylesheet) => Some(stylesheet),
      Self::JavaScript(_) => None,
    }
  }
}
