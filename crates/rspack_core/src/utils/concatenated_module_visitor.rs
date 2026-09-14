use rspack_intern::Atom;
use swc_core::common::SyntaxContext;
use swc_next_ecma_ast::Span;

/// Owned occurrence shared by concatenation and rendering caches after the AST
/// is dropped. Cloning retains the interned name and copies its range and flags.
#[derive(Clone, Debug)]
pub struct ConcatenatedModuleIdent {
  pub name: Atom,
  pub span: Span,
  pub scope: SyntaxContext,
  pub shorthand: bool,
  pub is_class_expr_with_ident: bool,
}
