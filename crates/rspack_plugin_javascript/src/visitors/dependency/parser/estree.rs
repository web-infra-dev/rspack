//! The compat estree helpers for swc ecma ast

use swc_core::{
  atoms::Atom,
  common::{Span, Spanned},
  ecma::ast::{
    BlockStmt, BreakStmt, Class, ClassDecl, ClassExpr, ContinueStmt, DebuggerStmt, Decl,
    DoWhileStmt, EmptyStmt, ExportAll, ExportDecl, ExportDefaultDecl, ExportDefaultExpr, Expr,
    ExprStmt, FnDecl, FnExpr, ForInStmt, ForOfStmt, ForStmt, Function, Ident, IfStmt, LabeledStmt,
    NamedExport, ReturnStmt, Stmt, SwitchStmt, ThrowStmt, TryStmt, UsingDecl, VarDecl, WhileStmt,
    WithStmt,
  },
};

#[derive(Debug, Clone, Copy)]
pub enum ClassDeclOrExpr<'ast> {
  Decl(MaybeNamedClassDecl<'ast>),
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

impl ClassDeclOrExpr<'_> {
  pub fn ident(&self) -> Option<&Ident> {
    match self {
      ClassDeclOrExpr::Decl(decl) => decl.ident,
      ClassDeclOrExpr::Expr(expr) => expr.ident.as_ref(),
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

#[derive(Debug, Clone, Copy)]
pub struct MaybeNamedFunctionDecl<'ast> {
  pub span: Span,
  pub ident: Option<&'ast Ident>,
  pub function: &'ast Function,
}

impl Spanned for MaybeNamedFunctionDecl<'_> {
  fn span(&self) -> Span {
    self.span
  }
}

impl<'ast> From<&'ast FnDecl> for MaybeNamedFunctionDecl<'ast> {
  fn from(value: &'ast FnDecl) -> Self {
    Self {
      span: value.span(),
      ident: Some(&value.ident),
      function: &value.function,
    }
  }
}

impl<'ast> From<&'ast FnExpr> for MaybeNamedFunctionDecl<'ast> {
  fn from(f: &'ast FnExpr) -> Self {
    Self {
      span: f.span(),
      ident: f.ident.as_ref(),
      function: &f.function,
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct MaybeNamedClassDecl<'ast> {
  pub span: Span,
  pub ident: Option<&'ast Ident>,
  pub class: &'ast Class,
}

impl Spanned for MaybeNamedClassDecl<'_> {
  fn span(&self) -> Span {
    self.span
  }
}

impl<'ast> From<&'ast ClassDecl> for MaybeNamedClassDecl<'ast> {
  fn from(value: &'ast ClassDecl) -> Self {
    Self {
      span: value.span(),
      ident: Some(&value.ident),
      class: &value.class,
    }
  }
}

impl<'ast> From<&'ast ClassExpr> for MaybeNamedClassDecl<'ast> {
  fn from(value: &'ast ClassExpr) -> Self {
    Self {
      span: value.span(),
      ident: value.ident.as_ref(),
      class: &value.class,
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum Statement<'ast> {
  Block(&'ast BlockStmt),
  Empty(&'ast EmptyStmt),
  Debugger(&'ast DebuggerStmt),
  With(&'ast WithStmt),
  Return(&'ast ReturnStmt),
  Labeled(&'ast LabeledStmt),
  Break(&'ast BreakStmt),
  Continue(&'ast ContinueStmt),
  If(&'ast IfStmt),
  Switch(&'ast SwitchStmt),
  Throw(&'ast ThrowStmt),
  Try(&'ast TryStmt),
  While(&'ast WhileStmt),
  DoWhile(&'ast DoWhileStmt),
  For(&'ast ForStmt),
  ForIn(&'ast ForInStmt),
  ForOf(&'ast ForOfStmt),
  Expr(&'ast ExprStmt),
  // ClassDecl, don't put ClassExpr into it, unless it's DefaultDecl::ClassExpr
  // which is represented by ClassExpr but it actually is a ClassDecl without ident
  Class(MaybeNamedClassDecl<'ast>),
  // FnDecl, don't put FnExpr into it, unless it's DefaultDecl::FnExpr
  // which is represented by FnExpr but it actually is a FnExpr without ident
  Fn(MaybeNamedFunctionDecl<'ast>),
  Var(&'ast VarDecl),
  Using(&'ast UsingDecl),
}

impl Spanned for Statement<'_> {
  fn span(&self) -> Span {
    use Statement::*;
    match self {
      Block(d) => d.span(),
      Empty(d) => d.span(),
      Debugger(d) => d.span(),
      With(d) => d.span(),
      Return(d) => d.span(),
      Labeled(d) => d.span(),
      Break(d) => d.span(),
      Continue(d) => d.span(),
      If(d) => d.span(),
      Switch(d) => d.span(),
      Throw(d) => d.span(),
      Try(d) => d.span(),
      While(d) => d.span(),
      DoWhile(d) => d.span(),
      For(d) => d.span(),
      ForIn(d) => d.span(),
      ForOf(d) => d.span(),
      Expr(d) => d.span(),
      Class(d) => d.span(),
      Fn(d) => d.span(),
      Var(d) => d.span(),
      Using(d) => d.span(),
    }
  }
}

impl<'ast> From<&'ast Stmt> for Statement<'ast> {
  fn from(value: &'ast Stmt) -> Self {
    use Statement::*;
    match value {
      Stmt::Block(d) => Block(d),
      Stmt::Empty(d) => Empty(d),
      Stmt::Debugger(d) => Debugger(d),
      Stmt::With(d) => With(d),
      Stmt::Return(d) => Return(d),
      Stmt::Labeled(d) => Labeled(d),
      Stmt::Break(d) => Break(d),
      Stmt::Continue(d) => Continue(d),
      Stmt::If(d) => If(d),
      Stmt::Switch(d) => Switch(d),
      Stmt::Throw(d) => Throw(d),
      Stmt::Try(d) => Try(d),
      Stmt::While(d) => While(d),
      Stmt::DoWhile(d) => DoWhile(d),
      Stmt::For(d) => For(d),
      Stmt::ForIn(d) => ForIn(d),
      Stmt::ForOf(d) => ForOf(d),
      Stmt::Expr(d) => Expr(d),
      Stmt::Decl(d) => d.into(),
    }
  }
}

impl<'ast> From<&'ast Decl> for Statement<'ast> {
  fn from(value: &'ast Decl) -> Self {
    use Statement::*;
    match value {
      Decl::Class(d) => Class(d.into()),
      Decl::Fn(d) => Fn(d.into()),
      Decl::Var(d) => Var(d),
      Decl::Using(d) => Using(d),
      Decl::TsInterface(_) | Decl::TsTypeAlias(_) | Decl::TsEnum(_) | Decl::TsModule(_) => {
        unreachable!()
      }
    }
  }
}

impl<'ast> Statement<'ast> {
  pub fn as_function_decl(&self) -> Option<MaybeNamedFunctionDecl<'ast>> {
    match self {
      Statement::Fn(f) => Some(*f),
      _ => None,
    }
  }

  pub fn as_class_decl(&self) -> Option<MaybeNamedClassDecl<'ast>> {
    match self {
      Statement::Class(c) => Some(*c),
      _ => None,
    }
  }
}
