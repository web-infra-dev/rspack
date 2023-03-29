use rspack_core::Provide;
use swc_core::common::Span;
use swc_core::common::{Mark, DUMMY_SP};
use swc_core::ecma::ast::{
  CallExpr, Callee, ComputedPropName, Expr, ExprOrSpread, Ident, Lit, MemberExpr, MemberProp, Str,
};
use swc_core::ecma::visit::{as_folder, Fold, VisitMut, VisitMutWith};

pub fn provide_builtin<'a>(opts: &'a Provide, unresolved_mark: Mark) -> impl Fold + 'a {
  as_folder(ProvideBuiltin::new(opts, unresolved_mark))
}

pub struct ProvideBuiltin<'a> {
  opts: &'a Provide,
  unresolved_mark: Mark,
}

impl<'a> ProvideBuiltin<'a> {
  pub fn new(opts: &'a Provide, unresolved_mark: Mark) -> Self {
    ProvideBuiltin {
      opts,
      unresolved_mark,
    }
  }

  fn handle_ident(&self, ident: &mut Ident) -> Expr {
    if let Some(module_path) = self.opts.get(&ident.sym.to_string()) {
      // println!("Before Ident transformation Span: {:?}", ident.span);

      let new_ident = self.create_obj_expr(ident.span, module_path);
      // println!("After Ident transformation Span: {:?}", new_ident);
      new_ident
    } else {
      Expr::Ident(ident.clone())
    }
  }

  fn handle_member_expr(&self, member_expr: &mut MemberExpr) -> Expr {
    let identifier_name = self.get_nested_identifier_name(member_expr);
    if let Some(module_path) = self.opts.get(&identifier_name) {
      // println!("Before Member transformation Span: {:?}", member_expr.span);

      let unresolved_span = DUMMY_SP.apply_mark(self.unresolved_mark);

      let new_expr = self.create_obj_expr(unresolved_span, module_path);
      // println!("After Member transformation Span: {:?}", new_expr);
      new_expr
    } else {
      Expr::Member(member_expr.clone())
    }
  }

  fn create_obj_expr(&self, span: Span, module_path: &[String]) -> Expr {
    let call_expr = self.create_call_expr(span, &module_path[0]);
    println!("call_expr: {:?}", call_expr);
    let mut obj_expr = Expr::Call(call_expr);

    // println!("obj_expr_sym: {:?}", obj_expr.as_call().unwrap().callee);

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
      // println!("obj_expr_sym: {:?}", obj_expr.as_member().unwrap().obj);
    }

    // println!("obj_expr_sym: {:?}", obj_expr.as_call().unwrap().callee);
    obj_expr
  }

  fn create_call_expr(&self, span: Span, module_path: &str) -> CallExpr {
    // println!("module_path: {}", module_path);
    // println!("span: {:?}", span);

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
            identifier_name.push_str(".");
          }
          identifier_name.push_str(&ident.sym.to_string());
        }
        Expr::This(_) => {
          if !identifier_name.is_empty() {
            identifier_name.push_str(".");
          }
          identifier_name.push_str("this");
        }
        _ => {}
      }

      if let Some(ident_prop) = member_expr.prop.as_ident() {
        identifier_name.push_str(".");
        identifier_name.push_str(&ident_prop.sym.to_string());
      }
    }

    build_identifier_name(member_expr, &mut identifier_name);
    identifier_name
  }
}

impl<'a> VisitMut for ProvideBuiltin<'a> {
  fn visit_mut_expr(&mut self, expr: &mut Expr) {
    *expr = match expr {
      Expr::Ident(ident) => self.handle_ident(ident),
      Expr::Member(member_expr) => self.handle_member_expr(member_expr),
      _ => expr.clone(),
    };

    expr.visit_mut_children_with(self);
  }
}
