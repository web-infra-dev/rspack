//! The compat estree helpers for swc ecma ast

use swc_core::{
  atoms::Atom,
  common::{Span, Spanned},
  ecma::ast::{
    ClassDecl, ClassExpr, ExportAll, ExportDecl, ExportDefaultDecl, ExportDefaultExpr, Expr,
    FnExpr, NamedExport,
  },
};

#[derive(Debug, Clone, Copy)]
pub enum ClassDeclOrExpr<'ast> {
  Decl(&'ast ClassDecl),
  Expr(&'ast ClassExpr),
}

impl Spanned for ClassDeclOrExpr<'_> {
  fn span(&self) -> Span {
    match self {
      ClassDeclOrExpr::Decl(decl) => decl.span(),
      ClassDeclOrExpr::Expr(expr) => expr.span(),
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportAllDeclaration<'ast> {
  /// `export * from 'm'`
  All(&'ast ExportAll),
  /// `export * as x from 'm'`
  NamedAll(&'ast NamedExport),
}

impl Spanned for ExportAllDeclaration<'_> {
  fn span(&self) -> Span {
    match self {
      ExportAllDeclaration::All(all) => all.span(),
      ExportAllDeclaration::NamedAll(all) => all.span(),
    }
  }
}

impl ExportAllDeclaration<'_> {
  pub fn source(&self) -> &Atom {
    match self {
      ExportAllDeclaration::All(e) => &e.src.value,
      ExportAllDeclaration::NamedAll(e) => {
        &e.src
          .as_ref()
          .expect("ExportAllDeclaration::NamedAll (export * as x from 'm') must have src")
          .value
      }
    }
  }

  pub fn source_span(&self) -> Span {
    match self {
      ExportAllDeclaration::All(all) => all.src.span(),
      ExportAllDeclaration::NamedAll(all) => all
        .src
        .as_ref()
        .expect("ExportAllDeclaration::NamedAll (export * as x from 'm') must have src")
        .span(),
    }
  }

  pub fn exported_name(&self) -> Option<&Atom> {
    match self {
      ExportAllDeclaration::All(_) => None,
      ExportAllDeclaration::NamedAll(e) => Some(
        e.specifiers
          .first()
          .and_then(|e| e.as_namespace())
          .map(|e| e.name.atom())
          .expect("ExportAllDeclaration::NamedAll (export * as x from 'm') must one specifier"),
      ),
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportNamedDeclaration<'ast> {
  /// `export var x = 1`
  /// `export class X {}`
  Decl(&'ast ExportDecl),
  /// `export { x } from 'm'`
  /// `export { x }`
  Specifiers(&'ast NamedExport),
}

impl Spanned for ExportNamedDeclaration<'_> {
  fn span(&self) -> Span {
    match self {
      ExportNamedDeclaration::Decl(decl) => decl.span(),
      ExportNamedDeclaration::Specifiers(export) => export.span(),
    }
  }
}

impl ExportNamedDeclaration<'_> {
  pub fn source(&self) -> Option<&Atom> {
    match self {
      Self::Decl(_) => None,
      Self::Specifiers(e) => e.src.as_ref().map(|s| &s.value),
    }
  }

  pub fn source_span(&self) -> Option<Span> {
    match self {
      ExportNamedDeclaration::Decl(_) => None,
      ExportNamedDeclaration::Specifiers(e) => e.src.as_ref().map(|s| s.span()),
    }
  }

  pub fn declaration_span(&self) -> Option<Span> {
    match self {
      ExportNamedDeclaration::Decl(decl) => Some(decl.decl.span()),
      ExportNamedDeclaration::Specifiers(_) => None,
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportDefaultDeclaration<'ast> {
  /// `export default class X {}`
  /// `export default class {}`
  /// `export default function x() {}`
  /// `export default function () {}`
  Decl(&'ast ExportDefaultDecl),
  /// `export default (class X {})`
  /// `export default 'x'`
  Expr(&'ast ExportDefaultExpr),
}

impl Spanned for ExportDefaultDeclaration<'_> {
  fn span(&self) -> Span {
    match self {
      ExportDefaultDeclaration::Decl(decl) => decl.span(),
      ExportDefaultDeclaration::Expr(expr) => expr.span(),
    }
  }
}

impl ExportDefaultDeclaration<'_> {
  fn declaration_span(&self) -> Span {
    match self {
      ExportDefaultDeclaration::Decl(decl) => decl.decl.span(),
      ExportDefaultDeclaration::Expr(expr) => expr.expr.span(),
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportDefaultExpression<'ast> {
  /// `export default function () {}`
  FnDecl(&'ast FnExpr),
  /// `export default class {}`
  ClassDecl(&'ast ClassExpr),
  /// `export default (class {})`
  /// `export default 'x'`
  Expr(&'ast Expr),
}

impl Spanned for ExportDefaultExpression<'_> {
  fn span(&self) -> Span {
    match self {
      ExportDefaultExpression::FnDecl(f) => f.span(),
      ExportDefaultExpression::ClassDecl(c) => c.span(),
      ExportDefaultExpression::Expr(e) => e.span(),
    }
  }
}

impl ExportDefaultExpression<'_> {
  pub fn ident(&self) -> Option<&Atom> {
    match self {
      ExportDefaultExpression::FnDecl(f) => f.ident.as_ref().map(|ident| &ident.sym),
      ExportDefaultExpression::ClassDecl(c) => c.ident.as_ref().map(|ident| &ident.sym),
      ExportDefaultExpression::Expr(_) => None,
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportImport<'ast> {
  All(ExportAllDeclaration<'ast>),
  Named(ExportNamedDeclaration<'ast>),
}

impl Spanned for ExportImport<'_> {
  fn span(&self) -> Span {
    match self {
      ExportImport::All(all) => all.span(),
      ExportImport::Named(named) => named.span(),
    }
  }
}

impl ExportImport<'_> {
  pub fn source(&self) -> &Atom {
    match self {
      ExportImport::All(e) => e.source(),
      ExportImport::Named(e) => e
        .source()
        .expect("ExportImport::Named (export { x } from 'm') should have src"),
    }
  }

  pub fn source_span(&self) -> Span {
    match self {
      ExportImport::All(all) => all.source_span(),
      ExportImport::Named(named) => named
        .source_span()
        .expect("ExportImport::Named (export { x } from 'm') should have src"),
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportLocal<'ast> {
  Named(ExportNamedDeclaration<'ast>),
  Default(ExportDefaultDeclaration<'ast>),
}

impl Spanned for ExportLocal<'_> {
  fn span(&self) -> Span {
    match self {
      ExportLocal::Named(decl) => decl.span(),
      ExportLocal::Default(decl) => decl.span(),
    }
  }
}

impl ExportLocal<'_> {
  pub fn declaration_span(&self) -> Option<Span> {
    match self {
      ExportLocal::Named(named) => named.declaration_span(),
      ExportLocal::Default(default) => Some(default.declaration_span()),
    }
  }
}
