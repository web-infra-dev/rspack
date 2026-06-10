//! The compat estree helpers for swc ecma ast

use swc_atoms::Atom;
use swc_experimental_ecma_ast::{
  BlockStmt, BreakStmt, Class, ClassDecl, ClassExpr, ContinueStmt, DebuggerStmt, Decl, DoWhileStmt,
  EmptyStmt, ExportAll, ExportDecl, ExportDefaultDecl, ExportDefaultExpr, ExportSpecifier, Expr,
  ExprStmt, FnDecl, FnExpr, ForInStmt, ForOfStmt, ForStmt, Function, GetSpan, Ident, IfStmt,
  LabeledStmt, ModuleExportName, NamedExport, ObjectLit, ReturnStmt, Span, Stmt, SwitchStmt,
  ThrowStmt, TryStmt, UsingDecl, VarDecl, VarDeclKind, VarDeclarator, WhileStmt, WithStmt,
};

use crate::JS_DEFAULT_KEYWORD;

fn wtf8_atom_to_atom(value: swc_experimental_allocator::atom::Wtf8Atom<'_>) -> Atom {
  Atom::from(value.as_wtf8().to_string_lossy().as_ref())
}

#[derive(Debug, Clone, Copy)]
pub enum ClassDeclOrExpr<'ast> {
  Decl(MaybeNamedClassDecl<'ast>),
  Expr(&'ast ClassExpr<'ast>),
}

impl ClassDeclOrExpr<'_> {
  pub fn span(&self) -> Span {
    match self {
      ClassDeclOrExpr::Decl(decl) => decl.span(),
      ClassDeclOrExpr::Expr(expr) => expr.span(),
    }
  }
}

impl ClassDeclOrExpr<'_> {
  pub fn ident(&self) -> Option<&Ident<'_>> {
    match self {
      ClassDeclOrExpr::Decl(decl) => decl.ident,
      ClassDeclOrExpr::Expr(expr) => expr.ident.as_deref(),
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportAllDeclaration<'ast> {
  /// `export * from 'm'`
  All(&'ast ExportAll<'ast>),
  /// `export * as x from 'm'`
  NamedAll(&'ast NamedExport<'ast>),
}

impl ExportAllDeclaration<'_> {
  pub fn span(&self) -> Span {
    match self {
      ExportAllDeclaration::All(all) => all.span(),
      ExportAllDeclaration::NamedAll(all) => all.span(),
    }
  }
}

impl ExportAllDeclaration<'_> {
  pub fn source(&self) -> Atom {
    match self {
      ExportAllDeclaration::All(e) => wtf8_atom_to_atom(e.src.value),
      ExportAllDeclaration::NamedAll(e) => wtf8_atom_to_atom(
        e.src
          .as_ref()
          .expect("ExportAllDeclaration::NamedAll (export * as x from 'm') must have src")
          .value,
      ),
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

  pub fn exported_name_span(&self) -> Option<Span> {
    match self {
      ExportAllDeclaration::All(_) => None,
      ExportAllDeclaration::NamedAll(e) => Some(
        e.specifiers
          .first()
          .and_then(|e| e.as_namespace())
          .map(|e| e.name.span())
          .expect("ExportAllDeclaration::NamedAll (export * as x from 'm') must one specifier"),
      ),
    }
  }

  pub fn exported_name(&self) -> Option<Atom> {
    match self {
      ExportAllDeclaration::All(_) => None,
      ExportAllDeclaration::NamedAll(e) => Some(
        e.specifiers
          .first()
          .and_then(|e| e.as_namespace())
          .map(|e| module_export_name_to_atom(&e.name))
          .expect("ExportAllDeclaration::NamedAll (export * as x from 'm') must one specifier"),
      ),
    }
  }

  pub fn get_with_obj(&self) -> Option<&ObjectLit<'_>> {
    match self {
      ExportAllDeclaration::All(e) => e.with.as_deref(),
      ExportAllDeclaration::NamedAll(e) => e.with.as_deref(),
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportNamedDeclaration<'ast> {
  /// `export var x = 1`
  /// `export class X {}`
  Decl(&'ast ExportDecl<'ast>),
  /// `export { x } from 'm'`
  /// `export { x }`
  Specifiers(&'ast NamedExport<'ast>),
}

impl ExportNamedDeclaration<'_> {
  pub fn span(&self) -> Span {
    match self {
      ExportNamedDeclaration::Decl(decl) => decl.span(),
      ExportNamedDeclaration::Specifiers(export) => export.span(),
    }
  }
}

impl ExportNamedDeclaration<'_> {
  pub fn source(&self) -> Option<Atom> {
    match self {
      Self::Decl(_) => None,
      Self::Specifiers(e) => e.src.as_ref().map(|s| wtf8_atom_to_atom(s.value)),
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

  pub fn get_with_obj(&self) -> Option<&ObjectLit<'_>> {
    match self {
      ExportNamedDeclaration::Decl(_) => None,
      ExportNamedDeclaration::Specifiers(e) => e.with.as_deref(),
    }
  }

  pub fn named_export_specifiers<'a>(
    named: &'a NamedExport<'a>,
  ) -> impl Iterator<Item = (Atom, Atom, Span)> + use<'a> {
    named.specifiers.iter().map(|spec| {
      match spec {
        ExportSpecifier::Namespace(_) => unreachable!("should handle ExportSpecifier::Namespace by ExportAllOrNamedAll::NamedAll in block_pre_walk_export_all_declaration"),
        ExportSpecifier::Default(s) => {
          (
            JS_DEFAULT_KEYWORD.clone(),
            Atom::from(s.exported.sym.as_str()),
            s.exported.span(),
          )
        },
        ExportSpecifier::Named(n) => {
          let exported_name = n.exported.as_ref().unwrap_or(&n.orig);
          (
            module_export_name_to_atom(&n.orig),
            module_export_name_to_atom(exported_name),
            exported_name.span(),
          )
        },
      }
    })
  }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportDefaultDeclaration<'ast> {
  /// `export default class X {}`
  /// `export default class {}`
  /// `export default function x() {}`
  /// `export default function () {}`
  Decl(&'ast ExportDefaultDecl<'ast>),
  /// `export default (class X {})`
  /// `export default 'x'`
  Expr(&'ast ExportDefaultExpr<'ast>),
}

impl ExportDefaultDeclaration<'_> {
  pub fn span(&self) -> Span {
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
  FnDecl(&'ast FnExpr<'ast>),
  /// `export default class {}`
  ClassDecl(&'ast ClassExpr<'ast>),
  /// `export default (class {})`
  /// `export default 'x'`
  Expr(&'ast Expr<'ast>),
}

impl ExportDefaultExpression<'_> {
  pub fn span(&self) -> Span {
    match self {
      ExportDefaultExpression::FnDecl(f) => f.span(),
      ExportDefaultExpression::ClassDecl(c) => c.span(),
      ExportDefaultExpression::Expr(e) => e.span(),
    }
  }
}

impl ExportDefaultExpression<'_> {
  pub fn ident(&self) -> Option<Atom> {
    match self {
      ExportDefaultExpression::FnDecl(f) => {
        f.ident.as_ref().map(|ident| Atom::from(ident.sym.as_str()))
      }
      ExportDefaultExpression::ClassDecl(c) => {
        c.ident.as_ref().map(|ident| Atom::from(ident.sym.as_str()))
      }
      ExportDefaultExpression::Expr(_) => None,
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportImport<'ast> {
  All(ExportAllDeclaration<'ast>),
  Named(ExportNamedDeclaration<'ast>),
}

impl ExportImport<'_> {
  pub fn span(&self) -> Span {
    match self {
      ExportImport::All(all) => all.span(),
      ExportImport::Named(named) => named.span(),
    }
  }
}

impl ExportImport<'_> {
  pub fn source(&self) -> Atom {
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

  pub fn get_with_obj(&self) -> Option<&ObjectLit<'_>> {
    match self {
      ExportImport::All(e) => e.get_with_obj(),
      ExportImport::Named(e) => e.get_with_obj(),
    }
  }

  pub fn is_star_export(&self) -> bool {
    matches!(self, ExportImport::All(ExportAllDeclaration::All(_)))
  }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportLocal<'ast> {
  Named(ExportNamedDeclaration<'ast>),
  Default(ExportDefaultDeclaration<'ast>),
}

impl ExportLocal<'_> {
  pub fn span(&self) -> Span {
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
  span: Span,
  ident: Option<&'ast Ident<'ast>>,
  function: &'ast Function<'ast>,
}

impl MaybeNamedFunctionDecl<'_> {
  pub fn span(&self) -> Span {
    self.span
  }
}

impl<'ast> From<&'ast FnDecl<'ast>> for MaybeNamedFunctionDecl<'ast> {
  fn from(value: &'ast FnDecl<'ast>) -> Self {
    Self {
      span: value.span(),
      ident: Some(&value.ident),
      function: &value.function,
    }
  }
}

impl<'ast> From<&'ast FnExpr<'ast>> for MaybeNamedFunctionDecl<'ast> {
  fn from(f: &'ast FnExpr<'ast>) -> Self {
    Self {
      span: f.span(),
      ident: f.ident.as_deref(),
      function: &f.function,
    }
  }
}

impl<'ast> MaybeNamedFunctionDecl<'ast> {
  pub fn ident(&self) -> Option<&'ast Ident<'ast>> {
    self.ident
  }

  pub fn function(&self) -> &'ast Function<'ast> {
    self.function
  }
}

#[derive(Debug, Clone, Copy)]
pub struct MaybeNamedClassDecl<'ast> {
  span: Span,
  ident: Option<&'ast Ident<'ast>>,
  class: &'ast Class<'ast>,
}

impl MaybeNamedClassDecl<'_> {
  pub fn span(&self) -> Span {
    self.span
  }
}

impl<'ast> From<&'ast ClassDecl<'ast>> for MaybeNamedClassDecl<'ast> {
  fn from(value: &'ast ClassDecl<'ast>) -> Self {
    Self {
      span: value.span(),
      ident: Some(&value.ident),
      class: &value.class,
    }
  }
}

impl<'ast> From<&'ast ClassExpr<'ast>> for MaybeNamedClassDecl<'ast> {
  fn from(value: &'ast ClassExpr<'ast>) -> Self {
    Self {
      span: value.span(),
      ident: value.ident.as_deref(),
      class: &value.class,
    }
  }
}

impl MaybeNamedClassDecl<'_> {
  pub fn ident(&self) -> Option<&Ident<'_>> {
    self.ident
  }

  pub fn class(&self) -> &Class<'_> {
    self.class
  }
}

#[derive(Debug, Clone, Copy)]
pub enum Statement<'ast> {
  Block(&'ast BlockStmt<'ast>),
  Empty(&'ast EmptyStmt),
  Debugger(&'ast DebuggerStmt),
  With(&'ast WithStmt<'ast>),
  Return(&'ast ReturnStmt<'ast>),
  Labeled(&'ast LabeledStmt<'ast>),
  Break(&'ast BreakStmt<'ast>),
  Continue(&'ast ContinueStmt<'ast>),
  If(&'ast IfStmt<'ast>),
  Switch(&'ast SwitchStmt<'ast>),
  Throw(&'ast ThrowStmt<'ast>),
  Try(&'ast TryStmt<'ast>),
  While(&'ast WhileStmt<'ast>),
  DoWhile(&'ast DoWhileStmt<'ast>),
  For(&'ast ForStmt<'ast>),
  ForIn(&'ast ForInStmt<'ast>),
  ForOf(&'ast ForOfStmt<'ast>),
  Expr(&'ast ExprStmt<'ast>),
  // ClassDecl, don't put ClassExpr into it, unless it's DefaultDecl::ClassExpr
  // which is represented by ClassExpr but it actually is a ClassDecl without ident
  Class(MaybeNamedClassDecl<'ast>),
  // FnDecl, don't put FnExpr into it, unless it's DefaultDecl::FnExpr
  // which is represented by FnExpr but it actually is a FnDecl without ident
  Fn(MaybeNamedFunctionDecl<'ast>),
  Var(VariableDeclaration<'ast>),
}

impl Statement<'_> {
  pub fn span(&self) -> Span {
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
    }
  }
}

impl<'ast> From<&'ast Stmt<'ast>> for Statement<'ast> {
  fn from(value: &'ast Stmt<'ast>) -> Self {
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
      Stmt::Decl(d) => (&**d).into(),
    }
  }
}

impl<'ast> From<&'ast Decl<'ast>> for Statement<'ast> {
  fn from(value: &'ast Decl<'ast>) -> Self {
    use Statement::*;
    match value {
      Decl::Class(d) => Class((&**d).into()),
      Decl::Fn(d) => Fn((&**d).into()),
      Decl::Var(d) => Var(VariableDeclaration::VarDecl(d)),
      Decl::Using(d) => Var(VariableDeclaration::UsingDecl(d)),
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

#[derive(Debug, Clone, Copy)]
pub enum VariableDeclaration<'a> {
  VarDecl(&'a VarDecl<'a>),
  UsingDecl(&'a UsingDecl<'a>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VariableDeclarationKind {
  Var,
  Let,
  Const,
  Using,
  AwaitUsing,
}

impl VariableDeclaration<'_> {
  pub fn span(&self) -> Span {
    match self {
      VariableDeclaration::VarDecl(var_decl) => var_decl.span(),
      VariableDeclaration::UsingDecl(using_decl) => using_decl.span(),
    }
  }
}

impl<'a> VariableDeclaration<'a> {
  pub fn kind(&self) -> VariableDeclarationKind {
    match self {
      VariableDeclaration::VarDecl(v) => match v.kind {
        VarDeclKind::Var => VariableDeclarationKind::Var,
        VarDeclKind::Let => VariableDeclarationKind::Let,
        VarDeclKind::Const => VariableDeclarationKind::Const,
      },
      VariableDeclaration::UsingDecl(u) => {
        if u.is_await {
          VariableDeclarationKind::AwaitUsing
        } else {
          VariableDeclarationKind::Using
        }
      }
    }
  }

  pub fn declarators(&self) -> &'a [VarDeclarator<'a>] {
    match self {
      VariableDeclaration::VarDecl(v) => &v.decls,
      VariableDeclaration::UsingDecl(u) => &u.decls,
    }
  }
}

fn module_export_name_to_atom(name: &ModuleExportName<'_>) -> Atom {
  match name {
    ModuleExportName::Ident(ident) => Atom::from(ident.sym.as_str()),
    ModuleExportName::Str(s) => wtf8_atom_to_atom(s.value),
  }
}
