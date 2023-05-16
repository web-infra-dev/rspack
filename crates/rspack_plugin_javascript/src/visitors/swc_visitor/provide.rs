use std::collections::HashSet;

use rspack_core::Provide;
use swc_core::common::util::take::Take;
use swc_core::common::Span;
use swc_core::common::{Mark, DUMMY_SP};
use swc_core::ecma::ast::{
  BindingIdent, CallExpr, Callee, ComputedPropName, Expr, ExprOrSpread, Ident, Lit, MemberExpr,
  MemberProp, ModuleItem, Stmt, Str, VarDecl, VarDeclarator,
};
use swc_core::ecma::visit::{as_folder, Fold, VisitMut, VisitMutWith};

pub fn provide_builtin(opts: &Provide, unresolved_mark: Mark) -> impl Fold + '_ {
  as_folder(ProvideBuiltin::new(opts, unresolved_mark))
}
static SOURCE_DOT: &str = r#"."#;
static MODULE_DOT: &str = r#"_dot_"#;
pub struct ProvideBuiltin<'a> {
  opts: &'a Provide,
  unresolved_mark: Mark,
  current_import_provide: HashSet<String>,
}

impl<'a> ProvideBuiltin<'a> {
  pub fn new(opts: &'a Provide, unresolved_mark: Mark) -> Self {
    ProvideBuiltin {
      opts,
      unresolved_mark,
      current_import_provide: HashSet::new(),
    }
  }

  fn handle_ident(&mut self, ident: &mut Ident) {
    if self.opts.get(&ident.sym.to_string()).is_some() {
      self.current_import_provide.insert(ident.sym.to_string());
    }
  }

  fn handle_member_expr(&mut self, member_expr: &MemberExpr) -> Option<Ident> {
    let identifier_name = self.get_nested_identifier_name(member_expr);
    if self.opts.get(&identifier_name).is_some() {
      self.current_import_provide.insert(identifier_name.clone());
      let new_ident_sym = identifier_name.replace(SOURCE_DOT, MODULE_DOT);
      return Some(Ident::new(
        new_ident_sym.into(),
        member_expr.span.apply_mark(self.unresolved_mark),
      ));
    }
    None
  }

  fn get_nested_identifier_name(&self, member_expr: &MemberExpr) -> String {
    let mut identifier_name = String::new();

    fn build_identifier_name(member_expr: &MemberExpr, identifier_name: &mut String) {
      match &*member_expr.obj {
        Expr::Member(nested_member_expr) => {
          build_identifier_name(nested_member_expr, identifier_name);
        }
        Expr::Ident(ident) => {
          if !identifier_name.is_empty() {
            identifier_name.push('.');
          }
          identifier_name.push_str(&ident.sym);
        }
        Expr::This(_) => {
          if !identifier_name.is_empty() {
            identifier_name.push('.');
          }
          identifier_name.push_str("this");
        }
        _ => {}
      }

      if let Some(ident_prop) = member_expr.prop.as_ident() {
        identifier_name.push('.');
        identifier_name.push_str(&ident_prop.sym);
      }
    }

    build_identifier_name(member_expr, &mut identifier_name);
    identifier_name
  }

  fn create_provide_require(&self) -> Vec<Stmt> {
    let mut stmt_item_vec = Vec::new();
    self
      .current_import_provide
      .iter()
      .for_each(|provide_module_name| {
        if let Some(provide_module_path) = self.opts.get(provide_module_name) {
          // require({module_path})
          let call = CallExpr {
            span: DUMMY_SP.apply_mark(self.unresolved_mark),
            callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(
              "require".into(),
              Span::apply_mark(DUMMY_SP, self.unresolved_mark),
            )))),
            args: vec![ExprOrSpread {
              spread: None,
              expr: Box::new(Expr::Lit(Lit::Str(Str {
                span: DUMMY_SP.apply_mark(self.unresolved_mark),
                value: provide_module_path[0].clone().into(),
                raw: None,
              }))),
            }],
            type_args: Default::default(),
          };
          let mut obj_expr = Expr::Call(call);
          // [""]
          for provide_module_member in provide_module_path.iter().skip(1) {
            let member_expr = MemberExpr {
              span: DUMMY_SP.apply_mark(self.unresolved_mark),
              obj: Box::new(obj_expr),
              prop: MemberProp::Computed(ComputedPropName {
                span: DUMMY_SP.apply_mark(self.unresolved_mark),
                expr: Box::new(Expr::Lit(Lit::Str(Str {
                  span: DUMMY_SP.apply_mark(self.unresolved_mark),
                  value: provide_module_member.to_string().into(),
                  raw: None,
                }))),
              }),
            };

            obj_expr = Expr::Member(member_expr);
          }
          // var {provide_module_name} = require(provide_module_path)?[provide_args]
          let stmt_item = Stmt::Decl(swc_core::ecma::ast::Decl::Var(Box::new(VarDecl {
            span: DUMMY_SP.apply_mark(self.unresolved_mark),
            declare: false,
            kind: swc_core::ecma::ast::VarDeclKind::Var,
            decls: vec![VarDeclarator {
              span: DUMMY_SP.apply_mark(self.unresolved_mark),
              definite: false,
              init: Some(Box::new(obj_expr)),
              name: swc_core::ecma::ast::Pat::Ident(BindingIdent {
                id: Ident::new(
                  provide_module_name.replace(SOURCE_DOT, MODULE_DOT).into(),
                  DUMMY_SP.apply_mark(self.unresolved_mark),
                ),
                type_ann: None,
              }),
            }],
          })));

          stmt_item_vec.push(stmt_item);
        }
      });
    stmt_item_vec
  }
}

impl VisitMut for ProvideBuiltin<'_> {
  fn visit_mut_expr(&mut self, expr: &mut Expr) {
    match expr {
      Expr::Ident(ident) => self.handle_ident(ident),
      Expr::Member(member_expr) => {
        if let Some(ident) = self.handle_member_expr(member_expr) {
          *expr = Expr::Ident(ident);
        }
      }
      _ => {}
    };

    expr.visit_mut_children_with(self);
  }

  fn visit_mut_program(&mut self, program: &mut swc_core::ecma::ast::Program) {
    program.visit_mut_children_with(self);
    let mut stmt_vec = self.create_provide_require();

    match program {
      swc_core::ecma::ast::Program::Module(module) => {
        let new_body = stmt_vec
          .into_iter()
          .map(ModuleItem::Stmt)
          .chain(module.body.take())
          .collect();

        module.body = new_body;
      }
      swc_core::ecma::ast::Program::Script(script) => {
        stmt_vec.extend(script.body.take());
        script.body = stmt_vec;
      }
    }
  }

  fn visit_mut_var_decl(&mut self, n: &mut VarDecl) {
    n.visit_mut_children_with(self);
  }
}
