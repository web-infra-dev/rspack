use swc_core::common::Spanned;
use swc_core::ecma::ast::{Ident, ObjectPatProp, Pat, VarDeclKind};

use super::{JavaScriptParserPluginDrive, JavascriptParserPlugin};
use crate::visitors::{create_traceable_error, is_reserved_word_in_strict, JavascriptParser};

pub struct CheckVarDeclaratorIdent;

impl CheckVarDeclaratorIdent {
  fn check_ident(&self, parser: &mut JavascriptParser, ident: &Ident) {
    if is_reserved_word_in_strict(ident.sym.as_str()) {
      if parser.is_strict() {
        parser.errors.push(Box::new(create_traceable_error(
          "JavaScript parsing error".into(),
          format!("The keyword '{}' is reserved in strict mode", ident.sym),
          &parser.source_file,
          ident.span().into(),
        )));
      } else {
        parser.errors.push(Box::new(create_traceable_error(
          "JavaScript parsing error".into(),
          format!("{} is disallowed as a lexically bound name", ident.sym),
          &parser.source_file,
          ident.span().into(),
        )));
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

impl<'ast, 'parser> JavascriptParserPlugin<'ast, 'parser> for CheckVarDeclaratorIdent {
  fn declarator(
    &self,
    parser: &mut JavascriptParser,
    _expr: &swc_core::ecma::ast::VarDeclarator,
    stmt: &swc_core::ecma::ast::VarDecl,
    _plugin_drive: &JavaScriptParserPluginDrive,
  ) -> Option<bool> {
    let should_check = match stmt.kind {
      VarDeclKind::Var => parser.is_strict(),
      _ => true,
    };
    if should_check {
      for ele in &stmt.decls {
        self.check_var_decl_pat(parser, &ele.name);
      }
    }
    None
  }
}
