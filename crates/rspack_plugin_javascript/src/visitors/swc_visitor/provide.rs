use std::sync::Arc;

use rspack_core::Provide;
use swc_core::common::Span;
use swc_core::common::{errors::Handler, SourceMap, DUMMY_SP};
use swc_core::ecma::ast::{
  CallExpr, Callee, ComputedPropName, Expr, ExprOrSpread, Ident, ImportDecl, Lit, MemberExpr,
  MemberProp, ModuleDecl, ModuleItem, Str,
};
use swc_core::ecma::visit::{as_folder, Fold, VisitMut, VisitMutWith};

pub fn provide_builtin<'a>(
  opts: &'a Provide,
  handler: &'a Handler,
  cm: &'a Arc<SourceMap>,
) -> impl Fold + 'a {
  as_folder(ProvideBuiltin::new(opts, handler, cm))
}

pub struct ProvideBuiltin<'a> {
  opts: &'a Provide,
  handler: &'a Handler,
  cm: &'a Arc<SourceMap>,
}

impl<'a> ProvideBuiltin<'a> {
  pub fn new(opts: &'a Provide, handler: &'a Handler, cm: &'a Arc<SourceMap>) -> Self {
    ProvideBuiltin { opts, handler, cm }
  }

  fn handle_ident(&self, ident: &mut Ident) -> Expr {
    if let Some(module_path) = self.opts.get(&ident.sym.to_string()) {
      self.create_obj_expr(ident.span, module_path)
    } else {
      Expr::Ident(ident.clone())
    }
  }

  fn handle_member_expr(&self, member_expr: &mut MemberExpr) -> Expr {
    let identifier_name = self.get_nested_identifier_name(member_expr);
    println!("identifier_name: {}", identifier_name);
    if let Some(module_path) = self.opts.get(&identifier_name) {
      self.create_obj_expr_with_prop(member_expr.span, module_path)
    } else {
      Expr::Member(member_expr.clone())
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

  fn create_obj_expr_with_prop(&self, span: Span, module_path: &[String]) -> Expr {
    let call_expr = self.create_call_expr(span, &module_path[0]);

    if module_path.len() > 1 {
      let prop_sym = module_path[1..].join(".").to_string();

      let member_expr = MemberExpr {
        span,
        obj: Box::new(Expr::Call(call_expr)),
        prop: MemberProp::Ident(Ident::new(prop_sym.into(), span)),
      };

      Expr::Member(member_expr)
    } else {
      Expr::Call(call_expr)
    }
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

      if let Some(iden_prop) = member_expr.prop.as_ident() {
        identifier_name.push_str(".");
        identifier_name.push_str(&iden_prop.sym.to_string());
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

  fn visit_mut_module_items(&mut self, node: &mut Vec<ModuleItem>) {
    for (_key, module_paths) in self.opts {
      // println!("key: {:?}", key);
      // println!("module_paths: {:?}", module_paths);

      let import_decl = ImportDecl {
        span: DUMMY_SP,
        specifiers: vec![],
        src: Box::new(Str {
          span: DUMMY_SP,
          value: module_paths[0].to_string().into(),
          raw: Option::None,
        }),
        type_only: false,
        asserts: None,
      };
      node.push(ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)));
    }

    node.visit_mut_children_with(self);
  }
}
