use std::borrow::Cow;
use std::collections::VecDeque;

use swc_core::common::{Span, SyntaxContext};
use swc_core::ecma::ast::*;
use swc_core::ecma::atoms::Atom;

pub enum MaybeExpr<'e> {
  Expr(Cow<'e, Expr>),
  MemberExpr(Cow<'e, MemberExpr>),
  OptChainExpr(Cow<'e, OptChainExpr>),
}

impl<'e> From<&'e Expr> for MaybeExpr<'e> {
  fn from(v: &'e Expr) -> Self {
    MaybeExpr::Expr(Cow::Borrowed(v))
  }
}

impl<'e> From<&'e MemberExpr> for MaybeExpr<'e> {
  fn from(v: &'e MemberExpr) -> Self {
    MaybeExpr::MemberExpr(Cow::Borrowed(v))
  }
}

impl<'e> From<&'e OptChainExpr> for MaybeExpr<'e> {
  fn from(v: &'e OptChainExpr) -> Self {
    MaybeExpr::OptChainExpr(Cow::Borrowed(v))
  }
}

impl From<Expr> for MaybeExpr<'_> {
  fn from(v: Expr) -> Self {
    MaybeExpr::Expr(Cow::Owned(v))
  }
}

impl From<MemberExpr> for MaybeExpr<'_> {
  fn from(v: MemberExpr) -> Self {
    MaybeExpr::MemberExpr(Cow::Owned(v))
  }
}

impl From<OptChainExpr> for MaybeExpr<'_> {
  fn from(v: OptChainExpr) -> Self {
    MaybeExpr::OptChainExpr(Cow::Owned(v))
  }
}

#[derive(Debug)]
pub struct ExpressionInfoCallExpression {
  root_members: VecDeque<(Atom, SyntaxContext)>,
  args: Vec<ExprOrSpread>,
}

impl ExpressionInfoCallExpression {
  pub fn root_members(&self) -> &VecDeque<(Atom, SyntaxContext)> {
    &self.root_members
  }
  pub fn args(&self) -> &Vec<ExprOrSpread> {
    &self.args
  }
}

#[derive(Debug)]
pub struct ExpressionInfoMemberExpression {
  object: MemberExpr,
}

impl ExpressionInfoMemberExpression {
  pub fn obj(&self) -> &MemberExpr {
    &self.object
  }
}

#[derive(Debug)]
pub enum ExpressionInfoKind {
  CallExpression(Box<ExpressionInfoCallExpression>),
  MemberExpression(Box<ExpressionInfoMemberExpression>),
  Expression,
}

#[derive(Debug)]
pub struct ExpressionInfo {
  kind: ExpressionInfoKind,
  members: VecDeque<(Atom, SyntaxContext)>,
  members_optionals: VecDeque<bool>,
  members_spans: VecDeque<Span>,
}

impl ExpressionInfo {
  pub fn kind(&self) -> &ExpressionInfoKind {
    &self.kind
  }

  pub fn members(&self) -> VecDeque<Cow<(Atom, SyntaxContext)>> {
    self.members.iter().map(Cow::Borrowed).collect()
  }

  pub fn members_optionals(&self) -> &VecDeque<bool> {
    &self.members_optionals
  }

  pub fn members_spans(&self) -> &VecDeque<Span> {
    &self.members_spans
  }

  pub fn non_optional_part(&self) -> VecDeque<Cow<(Atom, SyntaxContext)>> {
    let index = self
      .members_optionals
      .iter()
      .enumerate()
      .find(|(_, o)| **o)
      .map(|(i, _)| i)
      .unwrap_or(self.members.len() - 1);
    self
      .members
      .iter()
      .take(index + 1)
      .map(Cow::Borrowed)
      .collect()
  }
}

pub fn extract_member_expression_chain<'e, T: Into<MaybeExpr<'e>>>(
  maybe_member_expr: T,
) -> ExpressionInfo {
  fn walk_opt_chain(
    expr: &OptChainExpr,
    members: &mut VecDeque<(Atom, SyntaxContext)>,
    members_optionals: &mut VecDeque<bool>,
    members_spans: &mut VecDeque<Span>,
    kind: &mut ExpressionInfoKind,
  ) {
    match expr.base {
      box OptChainBase::Member(ref member_expr) => {
        members_optionals.push_front(expr.optional);
        walk_member_expr(
          member_expr,
          true,
          members,
          members_optionals,
          members_spans,
          kind,
        )
      }
      box OptChainBase::Call(ref expr) => {
        let mut members = VecDeque::new();
        walk_expr(
          &expr.callee,
          &mut members,
          &mut Default::default(),
          &mut Default::default(),
          &mut ExpressionInfoKind::Expression,
        );
        *kind = ExpressionInfoKind::CallExpression(Box::new(ExpressionInfoCallExpression {
          root_members: members,
          args: expr.args.to_owned(),
        }));
      }
    }
  }

  fn walk_callee(callee: &Callee, args: &Vec<ExprOrSpread>, kind: &mut ExpressionInfoKind) {
    let mut members = VecDeque::new();
    let mut members_optionals = VecDeque::new();
    let mut members_spans = VecDeque::new();
    match callee {
      Callee::Expr(box ref expr) => walk_expr(
        expr,
        &mut members,
        &mut members_optionals,
        &mut members_spans,
        &mut ExpressionInfoKind::Expression,
      ),
      Callee::Super(ref s) => {
        members.push_front((Atom::from("super"), s.span.ctxt));
        members_spans.push_front(s.span);
      }
      Callee::Import(ref i) => {
        members.push_front((Atom::from("import"), i.span.ctxt));
        members_spans.push_front(i.span);
      }
    }
    *kind = ExpressionInfoKind::CallExpression(Box::new(ExpressionInfoCallExpression {
      root_members: members,
      args: args.to_owned(),
    }))
  }

  fn walk_call_expr(
    call_expr: &CallExpr,
    _members: &mut VecDeque<(Atom, SyntaxContext)>,
    _members_optionals: &mut VecDeque<bool>,
    _members_spans: &mut VecDeque<Span>,
    kind: &mut ExpressionInfoKind,
  ) {
    walk_callee(&call_expr.callee, &call_expr.args, kind)
  }

  fn walk_member_expr(
    expr: &MemberExpr,
    in_opt: bool,
    members: &mut VecDeque<(Atom, SyntaxContext)>,
    members_optionals: &mut VecDeque<bool>,
    members_spans: &mut VecDeque<Span>,
    kind: &mut ExpressionInfoKind,
  ) {
    if let MemberProp::Computed(ComputedPropName {
      expr: box Expr::Lit(Lit::Str(ref val)),
      ..
    }) = expr.prop
    {
      members.push_front((val.value.clone(), val.span.ctxt));
    } else if let MemberProp::Computed(ComputedPropName {
      expr: box Expr::Lit(Lit::Num(ref val)),
      ..
    }) = expr.prop
    {
      members.push_front((val.value.to_string().into(), val.span.ctxt));
    } else if let MemberProp::Ident(ref ident) = expr.prop {
      members.push_front((ident.sym.clone(), ident.span.ctxt));
      members_spans.push_front(ident.span);
    } else {
      *kind = ExpressionInfoKind::MemberExpression(Box::new(ExpressionInfoMemberExpression {
        object: expr.to_owned(),
      }));
      return;
    }

    if !in_opt {
      members_optionals.push_front(false);
    }

    walk_expr(&expr.obj, members, members_optionals, members_spans, kind);
  }

  fn walk_expr(
    expr: &Expr,
    members: &mut VecDeque<(Atom, SyntaxContext)>,
    members_optionals: &mut VecDeque<bool>,
    members_spans: &mut VecDeque<Span>,
    kind: &mut ExpressionInfoKind,
  ) {
    match expr {
      Expr::OptChain(ref opt_chain) => {
        walk_opt_chain(opt_chain, members, members_optionals, members_spans, kind)
      }
      Expr::Call(ref call_expr) => {
        walk_call_expr(call_expr, members, members_optionals, members_spans, kind)
      }
      Expr::Member(ref expr) => {
        walk_member_expr(expr, false, members, members_optionals, members_spans, kind)
      }
      Expr::This(ref this_expr) => {
        members.push_front((Atom::from("this"), this_expr.span.ctxt));
        members_spans.push_front(this_expr.span);
      }
      Expr::Ident(ref ident) => {
        members.push_front((ident.sym.clone(), ident.span.ctxt));
        members_spans.push_front(ident.span);
      }
      _ => (),
    }
  }

  let mut kind: ExpressionInfoKind = ExpressionInfoKind::Expression;
  let mut members: VecDeque<(Atom, SyntaxContext)> = VecDeque::new();
  let mut members_optionals: VecDeque<bool> = VecDeque::new();
  let mut members_spans: VecDeque<Span> = VecDeque::new();

  match maybe_member_expr.into() {
    MaybeExpr::Expr(e) => walk_expr(
      &e,
      &mut members,
      &mut members_optionals,
      &mut members_spans,
      &mut kind,
    ),
    MaybeExpr::MemberExpr(e) => walk_member_expr(
      &e,
      false,
      &mut members,
      &mut members_optionals,
      &mut members_spans,
      &mut kind,
    ),
    MaybeExpr::OptChainExpr(e) => walk_opt_chain(
      &e,
      &mut members,
      &mut members_optionals,
      &mut members_spans,
      &mut kind,
    ),
  };

  // dbg!(&members_optionals);
  // dbg!(&members);

  ExpressionInfo {
    kind,
    members,
    members_optionals,
    members_spans,
  }
}

#[test]
fn optional() {
  macro_rules! test {
    ($tt:tt,$expr:expr) => {{
      use super::extract_member_expression_chain;
      let info = extract_member_expression_chain(swc_core::quote!($tt as Expr));
      assert_eq!(info.members_optionals(), &VecDeque::from_iter($expr))
    }};
  }

  test!("a().b?.c", [false, true]);
  test!("a?.().b.c", [false, false]);
  test!("a.b?.c?.().e", [false]);
  test!("b?.a?.a", [true, true]);
  test!("bb.call?.().c", [false]);
  test!("bb.call?.().b?.a", [false, true]);
  test!("bb.a?.call()", []);
  test!("bb.a?.c?.b", [false, true, true]);
}

#[test]
fn call_expr() {
  macro_rules! test {
    ($tt:tt,$expr:expr) => {{
      use super::{extract_member_expression_chain, ExpressionInfoKind};
      let info = extract_member_expression_chain(swc_core::quote!($tt as Expr));
      assert!(matches!(info.kind(), ExpressionInfoKind::CallExpression(_)));
      if let ExpressionInfoKind::CallExpression(info) = info.kind() {
        assert_eq!(
          info
            .root_members()
            .iter()
            .map(|(v, _)| v.as_ref())
            .collect::<VecDeque<_>>(),
          VecDeque::from_iter($expr)
        )
      }
    }};
  }

  test!("a().b?.c", ["a"]);
  test!("a?.().b.c", ["a"]);
  test!("a.b?.c?.().e", ["a", "b", "c"]);
  test!("bb.call?.().c", ["bb", "call"]);
  test!("bb.call?.().b?.a", ["bb", "call"]);
  test!("bb.a?.call()", ["bb", "a", "call"]);
}
