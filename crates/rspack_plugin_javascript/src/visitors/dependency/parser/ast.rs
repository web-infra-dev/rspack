use swc_next_ecma_ast::{
  Ast, AwaitExpression, CallExpression, ChainExpression, Expr, ExprData, GetSpan,
  IdentifierReference, JsxMemberExpression, MemberExpression, MetaProperty, Span, ThisExpression,
};

/// The hook-facing source location for an identifier-like reference.
///
/// JSX identifiers and ECMAScript identifier references use different SWC
/// Next node kinds, while parser plugins only need the source span; keeping
/// that semantic shape avoids manufacturing AST nodes.
#[derive(Debug, Clone, Copy)]
pub struct Identifier {
  pub span: Span,
}

impl Identifier {
  pub fn span(self) -> Span {
    self.span
  }
}

/// The hook-facing representation of an ECMAScript or JSX member expression.
///
/// SWC Next keeps those node kinds distinct. Parser plugins mostly need the
/// complete source span, while the few ECMAScript-only paths can explicitly
/// request the underlying node instead of manufacturing an incompatible AST
/// handle for JSX.
#[derive(Debug, Clone, Copy)]
pub enum HookMemberExpression {
  Ecma(MemberExpression),
  Jsx(JsxMemberExpression),
}

impl HookMemberExpression {
  pub fn span(self, ast: &Ast<'_>) -> Span {
    match self {
      Self::Ecma(expression) => expression.span(ast),
      Self::Jsx(expression) => expression.span(ast),
    }
  }

  pub fn ecma(self) -> Option<MemberExpression> {
    match self {
      Self::Ecma(expression) => Some(expression),
      Self::Jsx(_) => None,
    }
  }
}

impl From<MemberExpression> for HookMemberExpression {
  fn from(expression: MemberExpression) -> Self {
    Self::Ecma(expression)
  }
}

impl From<JsxMemberExpression> for HookMemberExpression {
  fn from(expression: JsxMemberExpression) -> Self {
    Self::Jsx(expression)
  }
}

/// A small discriminant helper for member-chain analysis. Payloads remain SWC
/// Next typed handles and must always be read through the parser's `Ast`.
#[derive(Debug, Clone, Copy)]
pub enum ExprRef {
  Await(AwaitExpression),
  Call(CallExpression),
  Ident(IdentifierReference),
  Member(MemberExpression),
  MetaProp(MetaProperty),
  OptChain(ChainExpression),
  This(ThisExpression),
  Other(Expr),
}

impl ExprRef {
  #[inline]
  pub fn from_expr(ast: &Ast<'_>, expr: Expr) -> Self {
    match ast.expr_data(expr) {
      ExprData::AwaitExpression(node) => Self::Await(node),
      ExprData::CallExpression(node) => Self::Call(node),
      ExprData::IdentifierReference(node) => Self::Ident(node),
      ExprData::MemberExpression(node) => Self::Member(node),
      ExprData::MetaProperty(node) => Self::MetaProp(node),
      ExprData::ChainExpression(node) => Self::OptChain(node),
      ExprData::ThisExpression(node) => Self::This(node),
      _ => Self::Other(expr),
    }
  }

  #[inline]
  pub fn expression(self) -> Expr {
    match self {
      Self::Await(node) => Expr::AwaitExpression(node),
      Self::Call(node) => Expr::CallExpression(node),
      Self::Ident(node) => Expr::IdentifierReference(node),
      Self::Member(node) => Expr::MemberExpression(node),
      Self::MetaProp(node) => Expr::MetaProperty(node),
      Self::OptChain(node) => Expr::ChainExpression(node),
      Self::This(node) => Expr::ThisExpression(node),
      Self::Other(expr) => expr,
    }
  }
}
