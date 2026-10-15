use swc_next_ecma_ast::{
  Ast, BindingIdentifier, GetSpan, IdentifierReference, JsxMemberExpression, MemberExpression, Span,
};

/// An identifier visited by pattern hooks, either in a binding or an assignment.
#[derive(Debug, Clone, Copy)]
pub enum PatternIdentifier {
  Binding(BindingIdentifier),
  Reference(IdentifierReference),
}

impl PatternIdentifier {
  pub fn name<'a>(self, ast: &'a Ast<'_>) -> &'a str {
    ast.get_utf8(match self {
      Self::Binding(identifier) => identifier.name(ast),
      Self::Reference(identifier) => identifier.name(ast),
    })
  }

  pub fn span(self, ast: &Ast<'_>) -> Span {
    match self {
      Self::Binding(identifier) => identifier.span(ast),
      Self::Reference(identifier) => identifier.span(ast),
    }
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
