use std::sync::LazyLock;

use rustc_hash::FxHashSet;
use swc_experimental_ecma_ast::{GetSpan, Ident, ObjectPatProp, Pat, VarDeclarator};

use super::JavascriptParserPlugin;
use crate::visitors::{
  JavascriptParser, VariableDeclaration, VariableDeclarationKind, create_traceable_error,
};

static STRICT_MODE_RESERVED_WORDS: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
  [
    "implements",
    "interface",
    "let",
    "package",
    "private",
    "protected",
    "public",
    "static",
    "yield",
    "await",
  ]
  .iter()
  .copied()
  .collect()
});

fn is_reserved_word_in_strict(word: &str) -> bool {
  STRICT_MODE_RESERVED_WORDS.contains(word)
}

pub struct CheckVarDeclaratorIdent;

impl CheckVarDeclaratorIdent {
  fn check_ident(&self, parser: &mut JavascriptParser, ident: Ident) {
    if is_reserved_word_in_strict(parser.ast.get_utf8(ident.sym(&parser.ast))) {
      if parser.is_strict() {
        parser.add_error(
          create_traceable_error(
            "JavaScript parse error".into(),
            format!(
              "The keyword '{}' is reserved in strict mode",
              parser.ast.get_utf8(ident.sym(&parser.ast))
            ),
            parser.source.to_string(),
            ident.span(&parser.ast).into(),
          )
          .into(),
        );
      } else {
        parser.add_error(
          create_traceable_error(
            "JavaScript parse error".into(),
            format!(
              "{} is disallowed as a lexically bound name",
              parser.ast.get_utf8(ident.sym(&parser.ast))
            ),
            parser.source.to_string(),
            ident.span(&parser.ast).into(),
          )
          .into(),
        );
      }
    }
  }

  fn check_var_decl_pat(&self, parser: &mut JavascriptParser, pat: Pat) {
    match pat {
      Pat::Ident(ident) => {
        self.check_ident(parser, ident.id(&parser.ast));
      }
      Pat::Array(bindings) => {
        for binding in bindings.elems(&parser.ast).iter() {
          let binding = parser.ast.get_node_in_sub_range(binding);
          if let Some(binding) = binding {
            self.check_var_decl_pat(parser, binding);
          }
        }
      }
      Pat::Object(obj) => {
        for prop in obj.props(&parser.ast).iter() {
          let prop = parser.ast.get_node_in_sub_range(prop);
          match prop {
            ObjectPatProp::KeyValue(pair) => {
              self.check_var_decl_pat(parser, pair.value(&parser.ast));
            }
            ObjectPatProp::Assign(assign) => {
              self.check_ident(parser, assign.key(&parser.ast).id(&parser.ast));
            }
            ObjectPatProp::Rest(rest) => {
              self.check_var_decl_pat(parser, rest.arg(&parser.ast));
            }
          }
        }
      }
      Pat::Assign(assign) => {
        self.check_var_decl_pat(parser, assign.left(&parser.ast));
      }
      Pat::Rest(rest) => {
        self.check_var_decl_pat(parser, rest.arg(&parser.ast));
      }
      _ => unreachable!(),
    }
  }
}

impl JavascriptParserPlugin for CheckVarDeclaratorIdent {
  fn declarator(
    &self,
    parser: &mut JavascriptParser,
    _expr: VarDeclarator,
    stmt: VariableDeclaration,
  ) -> Option<bool> {
    let should_check = match stmt.kind(&parser.ast) {
      VariableDeclarationKind::Var => parser.is_strict(),
      _ => true,
    };
    if should_check {
      for ele in stmt.declarators(&parser.ast).iter() {
        let ele = parser.ast.get_node_in_sub_range(ele);
        self.check_var_decl_pat(parser, ele.name(&parser.ast));
      }
    }
    None
  }
}
