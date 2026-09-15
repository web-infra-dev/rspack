use swc_experimental_ecma_ast::*;

/// A reference-based wrapper around SWC AST expression types.
///
/// This enum holds references to various expression types to avoid
/// unnecessary cloning during AST traversal and analysis. The lifetime
/// `'ast` represents the lifetime of the original AST node being referenced.
#[derive(Debug)]
pub enum ExprRef<'ast> {
  This(&'ast ThisExpr),
  Array(&'ast ArrayLit<'ast>),
  Object(&'ast ObjectLit<'ast>),
  Fn(&'ast FnExpr<'ast>),
  Unary(&'ast UnaryExpr<'ast>),
  Update(&'ast UpdateExpr<'ast>),
  Bin(&'ast BinExpr<'ast>),
  Assign(&'ast AssignExpr<'ast>),
  Member(&'ast MemberExpr<'ast>),
  SuperProp(&'ast SuperPropExpr<'ast>),
  Cond(&'ast CondExpr<'ast>),
  Call(&'ast CallExpr<'ast>),
  New(&'ast NewExpr<'ast>),
  Seq(&'ast SeqExpr<'ast>),
  Ident(&'ast Ident<'ast>),
  Lit(&'ast Lit<'ast>),
  Tpl(&'ast Tpl<'ast>),
  TaggedTpl(&'ast TaggedTpl<'ast>),
  Arrow(&'ast ArrowExpr<'ast>),
  Class(&'ast ClassExpr<'ast>),
  Yield(&'ast YieldExpr<'ast>),
  MetaProp(&'ast MetaPropExpr),
  Await(&'ast AwaitExpr<'ast>),
  Paren(&'ast ParenExpr<'ast>),
  JSXMember(&'ast JSXMemberExpr<'ast>),
  JSXNamespacedName(&'ast JSXNamespacedName<'ast>),
  JSXEmpty(&'ast JSXEmptyExpr),
  JSXElement(&'ast JSXElement<'ast>),
  JSXFragment(&'ast JSXFragment<'ast>),
  PrivateName(&'ast PrivateName<'ast>),
  OptChain(&'ast OptChainExpr<'ast>),
  Invalid(&'ast Invalid),
}

impl<'ast> From<&'ast Expr<'ast>> for ExprRef<'ast> {
  fn from(expr: &'ast Expr<'ast>) -> Self {
    match expr {
      Expr::This(this_expr) => ExprRef::This(this_expr),
      Expr::Array(array_lit) => ExprRef::Array(array_lit),
      Expr::Object(object_lit) => ExprRef::Object(object_lit),
      Expr::Fn(fn_expr) => ExprRef::Fn(fn_expr),
      Expr::Unary(unary_expr) => ExprRef::Unary(unary_expr),
      Expr::Update(update_expr) => ExprRef::Update(update_expr),
      Expr::Bin(bin_expr) => ExprRef::Bin(bin_expr),
      Expr::Assign(assign_expr) => ExprRef::Assign(assign_expr),
      Expr::Member(member_expr) => ExprRef::Member(member_expr),
      Expr::SuperProp(super_prop_expr) => ExprRef::SuperProp(super_prop_expr),
      Expr::Cond(cond_expr) => ExprRef::Cond(cond_expr),
      Expr::Call(call_expr) => ExprRef::Call(call_expr),
      Expr::New(new_expr) => ExprRef::New(new_expr),
      Expr::Seq(seq_expr) => ExprRef::Seq(seq_expr),
      Expr::Ident(ident) => ExprRef::Ident(ident),
      Expr::Lit(lit) => ExprRef::Lit(lit),
      Expr::Tpl(tpl) => ExprRef::Tpl(tpl),
      Expr::TaggedTpl(tagged_tpl) => ExprRef::TaggedTpl(tagged_tpl),
      Expr::Arrow(arrow_expr) => ExprRef::Arrow(arrow_expr),
      Expr::Class(class_expr) => ExprRef::Class(class_expr),
      Expr::Yield(yield_expr) => ExprRef::Yield(yield_expr),
      Expr::MetaProp(meta_prop_expr) => ExprRef::MetaProp(meta_prop_expr),
      Expr::Await(await_expr) => ExprRef::Await(await_expr),
      Expr::Paren(paren_expr) => ExprRef::Paren(paren_expr),
      Expr::JSXMember(jsxmember_expr) => ExprRef::JSXMember(jsxmember_expr),
      Expr::JSXNamespacedName(jsxnamespaced_name) => ExprRef::JSXNamespacedName(jsxnamespaced_name),
      Expr::JSXEmpty(jsxempty_expr) => ExprRef::JSXEmpty(jsxempty_expr),
      Expr::JSXElement(jsxelement) => ExprRef::JSXElement(jsxelement),
      Expr::JSXFragment(jsxfragment) => ExprRef::JSXFragment(jsxfragment),
      Expr::PrivateName(private_name) => ExprRef::PrivateName(private_name),
      Expr::OptChain(opt_chain_expr) => ExprRef::OptChain(opt_chain_expr),
      Expr::Invalid(invalid) => ExprRef::Invalid(invalid),
    }
  }
}
