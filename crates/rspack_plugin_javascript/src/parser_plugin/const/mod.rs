mod if_stmt;
mod logic_expr;

use rspack_core::{CachedConstDependency, ConstDependency};
use rspack_util::SpanExt;
use swc_core::common::Spanned;

pub use self::logic_expr::is_logic_op;
use super::JavascriptParserPlugin;
use crate::{
  utils::eval::evaluate_to_string,
  visitors::{JavascriptParser, Statement},
};

pub struct ConstPlugin;

const RESOURCE_FRAGMENT: &str = "__resourceFragment";
const RESOURCE_QUERY: &str = "__resourceQuery";

impl JavascriptParserPlugin for ConstPlugin {
  fn expression_logical_operator(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::BinExpr,
  ) -> Option<bool> {
    self::logic_expr::expression_logic_operator(parser, expr)
  }

  fn expression_conditional_operation(
    &self,
    parser: &mut JavascriptParser,
    expression: &swc_core::ecma::ast::CondExpr,
  ) -> Option<bool> {
    let param = parser.evaluate_expression(&expression.test);
    if let Some(bool) = param.as_bool() {
      if !param.could_have_side_effects() {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          param.range().into(),
          format!(" {bool}").into(),
        )));
      } else {
        parser.walk_expression(&expression.test);
      }
      if bool {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          expression.alt.span().into(),
          "0".into(),
        )));
      } else {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          expression.cons.span().into(),
          "0".into(),
        )));
      }
      Some(bool)
    } else {
      None
    }
  }

  fn statement_if(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::IfStmt,
  ) -> Option<bool> {
    self::if_stmt::statement_if(parser, expr)
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &swc_core::ecma::ast::Ident,
    for_name: &str,
  ) -> Option<bool> {
    match for_name {
      RESOURCE_FRAGMENT => {
        let resource_fragment = parser.resource_data.fragment().unwrap_or("");
        parser.add_presentational_dependency(Box::new(CachedConstDependency::new(
          ident.span.into(),
          "__resourceFragment".into(),
          serde_json::to_string(resource_fragment)
            .expect("should render module id")
            .into(),
        )));
        Some(true)
      }
      RESOURCE_QUERY => {
        let resource_query = parser.resource_data.query().unwrap_or("");
        parser.add_presentational_dependency(Box::new(CachedConstDependency::new(
          ident.span.into(),
          "__resourceQuery".into(),
          serde_json::to_string(resource_query)
            .expect("should render module id")
            .into(),
        )));
        Some(true)
      }
      _ => None,
    }
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser,
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<crate::utils::eval::BasicEvaluatedExpression<'static>> {
    match for_name {
      RESOURCE_QUERY => Some(evaluate_to_string(
        parser
          .resource_data
          .query()
          .map(ToOwned::to_owned)
          .unwrap_or_default(),
        start,
        end,
      )),
      RESOURCE_FRAGMENT => Some(evaluate_to_string(
        parser
          .resource_data
          .fragment()
          .map(ToOwned::to_owned)
          .unwrap_or_default(),
        start,
        end,
      )),
      _ => None,
    }
  }

  fn unused_statement(&self, parser: &mut JavascriptParser, stmt: Statement) -> Option<bool> {
    // Skip top level scope to align with webpack's ConstPlugin behavior.
    if parser.is_top_level_scope() {
      return None;
    }

    use swc_core::ecma::ast::{
      BlockStmt, ClassDecl, Decl, DoWhileStmt, EmptyStmt, ExprStmt, ForInStmt, ForOfStmt, ForStmt,
      LabeledStmt, ReturnStmt, Stmt as SwcStmt, SwitchStmt, ThrowStmt, TryStmt, VarDecl, WhileStmt,
      WithStmt,
    };

    // Recreate a swc `Stmt` clone from our lightweight `Statement` wrapper so
    // that we can reuse `get_hoisted_declarations` to compute hoisted
    // declarations from the dead branch.
    let swc_stmt: SwcStmt = match stmt {
      Statement::Block(b) => SwcStmt::Block(BlockStmt {
        span: b.span,
        ctxt: b.ctxt,
        stmts: b.stmts.clone(),
      }),
      Statement::Empty(e) => SwcStmt::Empty(EmptyStmt { span: e.span }),
      Statement::Debugger(d) => SwcStmt::Debugger(*d),
      Statement::With(w) => SwcStmt::With(WithStmt {
        span: w.span,
        obj: w.obj.clone(),
        body: w.body.clone(),
      }),
      Statement::Return(r) => SwcStmt::Return(ReturnStmt {
        span: r.span,
        arg: r.arg.clone(),
      }),
      Statement::Labeled(l) => SwcStmt::Labeled(LabeledStmt {
        span: l.span,
        label: l.label.clone(),
        body: l.body.clone(),
      }),
      Statement::Break(b) => SwcStmt::Break(b.clone()),
      Statement::Continue(c) => SwcStmt::Continue(c.clone()),
      Statement::If(i) => SwcStmt::If(i.clone()),
      Statement::Switch(s) => SwcStmt::Switch(SwitchStmt {
        span: s.span,
        discriminant: s.discriminant.clone(),
        cases: s.cases.clone(),
      }),
      Statement::Throw(t) => SwcStmt::Throw(ThrowStmt {
        span: t.span,
        arg: t.arg.clone(),
      }),
      Statement::Try(t) => SwcStmt::Try(Box::new(TryStmt {
        span: t.span,
        block: t.block.clone(),
        handler: t.handler.clone(),
        finalizer: t.finalizer.clone(),
      })),
      Statement::While(w) => SwcStmt::While(WhileStmt {
        span: w.span,
        test: w.test.clone(),
        body: w.body.clone(),
      }),
      Statement::DoWhile(d) => SwcStmt::DoWhile(DoWhileStmt {
        span: d.span,
        body: d.body.clone(),
        test: d.test.clone(),
      }),
      Statement::For(f) => SwcStmt::For(ForStmt {
        span: f.span,
        init: f.init.clone(),
        test: f.test.clone(),
        update: f.update.clone(),
        body: f.body.clone(),
      }),
      Statement::ForIn(fi) => SwcStmt::ForIn(ForInStmt {
        span: fi.span,
        left: fi.left.clone(),
        right: fi.right.clone(),
        body: fi.body.clone(),
      }),
      Statement::ForOf(fo) => SwcStmt::ForOf(ForOfStmt {
        span: fo.span,
        is_await: fo.is_await,
        left: fo.left.clone(),
        right: fo.right.clone(),
        body: fo.body.clone(),
      }),
      Statement::Expr(e) => SwcStmt::Expr(ExprStmt {
        span: e.span,
        expr: e.expr.clone(),
      }),
      Statement::Class(class_decl) => {
        let class = ClassDecl {
          ident: class_decl
            .ident()
            .cloned()
            .unwrap_or_else(|| unreachable!("class decl should have ident")),
          class: Box::new(class_decl.class().clone()),
          declare: false,
        };
        SwcStmt::Decl(Decl::Class(class))
      }
      Statement::Fn(fn_decl) => {
        let ident = fn_decl
          .ident()
          .cloned()
          .unwrap_or_else(|| unreachable!("function decl should have ident"));
        let fn_decl = swc_core::ecma::ast::FnDecl {
          ident,
          declare: false,
          function: Box::new(fn_decl.function().clone()),
        };
        SwcStmt::Decl(Decl::Fn(fn_decl))
      }
      Statement::Var(var_decl) => {
        // Convert our wrapper back into a swc `VarDecl`.
        let swc_var: VarDecl = match var_decl {
          crate::visitors::VariableDeclaration::VarDecl(v) => (*v).clone(),
          crate::visitors::VariableDeclaration::UsingDecl(u) => VarDecl {
            span: u.span,
            ctxt: Default::default(),
            kind: swc_core::ecma::ast::VarDeclKind::Var,
            declare: false,
            decls: u.decls.clone(),
          },
        };
        SwcStmt::Decl(Decl::Var(Box::new(swc_var)))
      }
    };

    // Compute hoisted declarations from the dead statement.
    let include_function_declarations = !parser.is_strict();
    let declarations =
      self::if_stmt::get_hoisted_declarations(&swc_stmt, include_function_declarations);

    let replacement_body = if declarations.is_empty() {
      "{}".to_string()
    } else {
      let mut names: Vec<&str> = declarations.iter().map(|decl| decl.sym.as_str()).collect();
      names.sort_unstable();
      format!("{{ var {} }}", names.join(", "))
    };

    // Prepend the same comment as webpack for easier debugging.
    let mut replacement = String::from("// removed by dead control flow\n");
    replacement.push_str(&replacement_body);

    let span = stmt.span();
    parser.add_presentational_dependency(Box::new(ConstDependency::new(
      (span.real_lo(), span.real_hi()).into(),
      replacement.into_boxed_str(),
    )));

    Some(true)
  }
}
