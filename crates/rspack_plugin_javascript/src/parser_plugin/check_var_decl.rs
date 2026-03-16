use swc_core::{
  common::Spanned,
  ecma::ast::{Ident, ObjectPatProp, Pat},
};

use super::JavascriptParserPlugin;
use crate::visitors::{
  JavascriptParser, VariableDeclaration, VariableDeclarationKind, create_traceable_error,
};

fn is_reserved_word_in_strict(word: &str) -> bool {
  matches!(
    word,
    "implements"
      | "interface"
      | "let"
      | "package"
      | "private"
      | "protected"
      | "public"
      | "static"
      | "yield"
      | "await"
  )
}

pub struct CheckVarDeclaratorIdent;

impl CheckVarDeclaratorIdent {
  fn check_ident(&self, parser: &mut JavascriptParser, ident: &Ident) {
    if is_reserved_word_in_strict(ident.sym.as_str()) {
      if parser.is_strict() {
        parser.add_error(
          create_traceable_error(
            "JavaScript parse error".into(),
            format!("The keyword '{}' is reserved in strict mode", ident.sym),
            parser.source.to_string(),
            ident.span().into(),
          )
          .into(),
        );
      } else {
        parser.add_error(
          create_traceable_error(
            "JavaScript parse error".into(),
            format!("{} is disallowed as a lexically bound name", ident.sym),
            parser.source.to_string(),
            ident.span().into(),
          )
          .into(),
        );
      }
    }
  }

  fn check_var_decl_pat(&self, parser: &mut JavascriptParser, pat: &Pat) {
    match pat {
      Pat::Ident(ident) => {
        self.check_ident(parser, ident);
      }
      Pat::Array(bindings) => {
        for binding in bindings.elems.iter().flatten() {
          self.check_var_decl_pat(parser, binding);
        }
      }
      Pat::Object(obj) => {
        for prop in &obj.props {
          match prop {
            ObjectPatProp::KeyValue(pair) => {
              self.check_var_decl_pat(parser, &pair.value);
            }
            ObjectPatProp::Assign(assign) => {
              self.check_ident(parser, &assign.key);
            }
            ObjectPatProp::Rest(rest) => {
              self.check_var_decl_pat(parser, &rest.arg);
            }
          }
        }
      }
      Pat::Assign(assign) => {
        self.check_var_decl_pat(parser, &assign.left);
      }
      Pat::Rest(rest) => {
        self.check_var_decl_pat(parser, &rest.arg);
      }
      _ => unreachable!(),
    }
  }
}

#[rspack_macros::implemented_javascript_parser_hooks]
impl JavascriptParserPlugin for CheckVarDeclaratorIdent {
  fn declarator(
    &self,
    parser: &mut JavascriptParser,
    _expr: &swc_core::ecma::ast::VarDeclarator,
    stmt: VariableDeclaration<'_>,
  ) -> Option<bool> {
    let should_check = match stmt.kind() {
      VariableDeclarationKind::Var => parser.is_strict(),
      _ => true,
    };
    if should_check {
      for ele in stmt.declarators() {
        self.check_var_decl_pat(parser, &ele.name);
      }
    }
    None
  }
}
