use std::collections::HashSet;

use rspack_core::Provide;
use swc_core::common::Span;
use swc_core::common::{Mark, DUMMY_SP};
use swc_core::ecma::ast::{
  BindingIdent, CallExpr, Callee, ComputedPropName, Expr, ExprOrSpread, ExprStmt, Ident, Lit,
  MemberExpr, MemberProp, Module, ModuleItem, Stmt, Str, VarDecl, VarDeclarator,
};
use swc_core::ecma::atoms::JsWord;
use swc_core::ecma::visit::{as_folder, Fold, VisitMut, VisitMutWith};

pub fn provide_builtin(opts: &Provide, unresolved_mark: Mark) -> impl Fold + '_ {
  as_folder(ProvideBuiltin::new(opts, unresolved_mark))
}

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

  fn handle_ident(&mut self, ident: &Ident) {
    if let Some(module_path) = self.opts.get(&ident.sym.to_string()) {
      // dbg!(&ident);
      // self.create_obj_expr(ident.span, module_path)
      self.current_import_provide.insert(ident.sym.to_string());
    }
  }

  fn handle_member_expr(&self, member_expr: &MemberExpr) {
    let identifier_name = self.get_nested_identifier_name(member_expr);
    dbg!(&identifier_name);
    if let Some(module_path) = self.opts.get(&identifier_name) {
      dbg!("true");
      // self.current_import_provide.insert(ident.to_string());
      // let unresolved_span = DUMMY_SP.apply_mark(self.unresolved_mark);
      // self.create_obj_expr(unresolved_span, module_path)
    }
  }

  fn create_obj_expr(&self, span: Span, module_path: &[String]) -> Expr {
    let call_expr = self.create_call_expr(span, &module_path[0]);
    let mut obj_expr = Expr::Call(call_expr);

    for module_name in module_path.iter().skip(1) {
      let member_expr = MemberExpr {
        span,
        obj: Box::new(obj_expr),
        prop: MemberProp::Computed(ComputedPropName {
          span,
          expr: Box::new(Expr::Lit(Lit::Str(Str {
            span,
            value: module_name.to_string().into(),
            raw: None,
          }))),
        }),
      };

      obj_expr = Expr::Member(member_expr);
    }

    obj_expr
  }

  fn create_call_expr(&self, span: Span, module_path: &str) -> CallExpr {
    CallExpr {
      span,
      callee: Callee::Expr(Box::new(Expr::Ident(Ident::new("require".into(), span)))),
      args: vec![ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Lit(Lit::Str(Str {
          span,
          value: module_path.to_string().into(),
          raw: None,
        }))),
      }],
      type_args: Default::default(),
    }
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

  fn create_provide_require(&self) -> Vec<ModuleItem> {
    let mut module_item_vec = Vec::new();
    self.current_import_provide.iter().for_each(|item| {
      dbg!(item);
      if let Some(module_path) = self.opts.get(item) {
        let call = CallExpr {
          span: DUMMY_SP,
          callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(
            "require".into(),
            Span::apply_mark(DUMMY_SP, self.unresolved_mark),
          )))),
          args: vec![ExprOrSpread {
            spread: None,
            expr: Box::new(Expr::Lit(Lit::Str(Str {
              span: DUMMY_SP,
              value: module_path[0].clone().into(),
              raw: None,
            }))),
          }],
          type_args: Default::default(),
        };
        let mut obj_expr = Expr::Call(call);

        for module_name in module_path.iter().skip(1) {
          let member_expr = MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(obj_expr),
            prop: MemberProp::Computed(ComputedPropName {
              span: DUMMY_SP,
              expr: Box::new(Expr::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: module_name.to_string().into(),
                raw: None,
              }))),
            }),
          };

          obj_expr = Expr::Member(member_expr);
        }
        let module_item = ModuleItem::Stmt(Stmt::Decl(swc_core::ecma::ast::Decl::Var(Box::new(
          VarDecl {
            span: DUMMY_SP,
            declare: false,
            kind: swc_core::ecma::ast::VarDeclKind::Var,
            decls: vec![VarDeclarator {
              span: DUMMY_SP,
              definite: false,
              init: Some(Box::new(obj_expr)),
              name: swc_core::ecma::ast::Pat::Ident(BindingIdent {
                id: Ident::new(item.clone().into(), DUMMY_SP),
                type_ann: None,
              }),
            }],
          },
        ))));

        module_item_vec.push(module_item);
      }
    });
    dbg!(&module_item_vec);
    module_item_vec
  }
}

impl VisitMut for ProvideBuiltin<'_> {
  fn visit_mut_expr(&mut self, expr: &mut Expr) {
    match expr {
      Expr::Ident(ident) => self.handle_ident(ident),
      // Expr::Member(member_expr) => self.handle_member_expr(member_expr),
      _ => {}
    };

    expr.visit_mut_children_with(self);
  }

  fn visit_mut_module(&mut self, n: &mut swc_core::ecma::ast::Module) {
    n.visit_mut_children_with(self);
    dbg!(&self.current_import_provide);
    let module_item_vec = self.create_provide_require();
    // dbg!(&module_item_vec);
    module_item_vec.into_iter().for_each(|module_item| {
      dbg!(&module_item);
      n.body.insert(0, module_item);
    });
  }
}
