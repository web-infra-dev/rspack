//! The compat estree helpers for swc ecma ast

use swc_core::{atoms::Atom, common::Span};
use swc_experimental_ecma_ast::*;

use crate::{JS_DEFAULT_KEYWORD, visitors::JavascriptParser};

#[derive(Debug, Clone, Copy)]
pub enum ClassDeclOrExpr {
  Decl(MaybeNamedClassDecl),
  Expr(ClassExpr),
}

impl Spanned for ClassDeclOrExpr {
  #[inline]
  fn span(&self, ast: &Ast) -> Span {
    match self {
      ClassDeclOrExpr::Decl(decl) => decl.span(ast),
      ClassDeclOrExpr::Expr(expr) => expr.span(ast),
    }
  }

  #[inline]
  fn set_span(&mut self, ast: &mut Ast, span: Span) {
    match self {
      ClassDeclOrExpr::Decl(decl) => decl.span = span,
      ClassDeclOrExpr::Expr(expr) => expr.set_span(ast, span),
    }
  }
}

impl ClassDeclOrExpr {
  pub fn ident(&self, ast: &Ast) -> Option<Ident> {
    match self {
      ClassDeclOrExpr::Decl(decl) => decl.ident(),
      ClassDeclOrExpr::Expr(expr) => expr.ident(ast),
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportAllDeclaration {
  /// `export * from 'm'`
  All(ExportAll),
  /// `export * as x from 'm'`
  NamedAll(NamedExport),
}

impl Spanned for ExportAllDeclaration {
  #[inline]
  fn span(&self, ast: &Ast) -> Span {
    match self {
      ExportAllDeclaration::All(all) => all.span(ast),
      ExportAllDeclaration::NamedAll(all) => all.span(ast),
    }
  }

  #[inline]
  fn set_span(&mut self, ast: &mut Ast, span: Span) {
    match self {
      ExportAllDeclaration::All(all) => all.set_span(ast, span),
      ExportAllDeclaration::NamedAll(all) => all.set_span(ast, span),
    }
  }
}

impl ExportAllDeclaration {
  pub fn source(&self, ast: &Ast) -> Atom {
    match self {
      ExportAllDeclaration::All(e) => ast
        .get_wtf8_atom(e.src(ast).value(ast))
        .to_atom_lossy()
        .into_owned(),
      ExportAllDeclaration::NamedAll(e) => ast
        .get_wtf8_atom(
          e.src(ast)
            .expect("ExportAllDeclaration::NamedAll (export * as x from 'm') must have src")
            .value(ast),
        )
        .to_atom_lossy()
        .into_owned(),
    }
  }

  pub fn source_span(&self, ast: &Ast) -> Span {
    match self {
      ExportAllDeclaration::All(all) => all.src(ast).span(ast),
      ExportAllDeclaration::NamedAll(all) => all
        .src(ast)
        .expect("ExportAllDeclaration::NamedAll (export * as x from 'm') must have src")
        .span(ast),
    }
  }

  pub fn exported_name_span(&self, ast: &Ast) -> Option<Span> {
    match self {
      ExportAllDeclaration::All(_) => None,
      ExportAllDeclaration::NamedAll(e) => Some(
        e.specifiers(ast)
          .first()
          .and_then(|e| ast.get_node_in_sub_range(e).as_namespace())
          .map(|e| e.name(ast).span(ast))
          .expect("ExportAllDeclaration::NamedAll (export * as x from 'm') must one specifier"),
      ),
    }
  }

  pub fn exported_name(&self, ast: &Ast) -> Option<Atom> {
    match self {
      ExportAllDeclaration::All(_) => None,
      ExportAllDeclaration::NamedAll(e) => Some(
        e.specifiers(ast)
          .first()
          .and_then(|e| ast.get_node_in_sub_range(e).as_namespace())
          .map(|e| match e.name(ast) {
            ModuleExportName::Ident(ident) => ast.get_atom(ident.sym(ast)),
            ModuleExportName::Str(s) => {
              ast.get_wtf8_atom(s.value(ast)).to_atom_lossy().into_owned()
            }
          })
          .expect("ExportAllDeclaration::NamedAll (export * as x from 'm') must one specifier"),
      ),
    }
  }

  pub fn get_with_obj(&self, ast: &Ast) -> Option<ObjectLit> {
    match self {
      ExportAllDeclaration::All(e) => e.with(ast),
      ExportAllDeclaration::NamedAll(e) => e.with(ast),
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportNamedDeclaration {
  /// `export var x = 1`
  /// `export class X {}`
  Decl(ExportDecl),
  /// `export { x } from 'm'`
  /// `export { x }`
  Specifiers(NamedExport),
}

impl Spanned for ExportNamedDeclaration {
  #[inline]
  fn span(&self, ast: &Ast) -> Span {
    match self {
      ExportNamedDeclaration::Decl(decl) => decl.span(ast),
      ExportNamedDeclaration::Specifiers(export) => export.span(ast),
    }
  }

  #[inline]
  fn set_span(&mut self, ast: &mut Ast, span: Span) {
    match self {
      ExportNamedDeclaration::Decl(decl) => decl.set_span(ast, span),
      ExportNamedDeclaration::Specifiers(export) => export.set_span(ast, span),
    }
  }
}

impl ExportNamedDeclaration {
  pub fn source(&self, ast: &Ast) -> Option<Atom> {
    match self {
      Self::Decl(_) => None,
      Self::Specifiers(e) => e
        .src(ast)
        .map(|s| ast.get_wtf8_atom(s.value(ast)).to_atom_lossy().into_owned()),
    }
  }

  pub fn source_span(&self, ast: &Ast) -> Option<Span> {
    match self {
      ExportNamedDeclaration::Decl(_) => None,
      ExportNamedDeclaration::Specifiers(e) => e.src(ast).as_ref().map(|s| s.span(ast)),
    }
  }

  pub fn declaration_span(&self, ast: &Ast) -> Option<Span> {
    match self {
      ExportNamedDeclaration::Decl(decl) => Some(decl.decl(ast).span(ast)),
      ExportNamedDeclaration::Specifiers(_) => None,
    }
  }

  pub fn get_with_obj(&self, ast: &Ast) -> Option<ObjectLit> {
    match self {
      ExportNamedDeclaration::Decl(_) => None,
      ExportNamedDeclaration::Specifiers(e) => e.with(ast),
    }
  }

  pub fn named_export_specifiers<F: FnMut(&mut JavascriptParser<'_>, Atom, Atom, Span)>(
    parser: &mut JavascriptParser<'_>,
    named: NamedExport,
    f: F,
  ) {
    for spec in named.specifiers(&parser.ast).iter() {
      let spec = parser.ast.get_node_in_sub_range(spec);
      match spec {
        ExportSpecifier::Namespace(_) => unreachable!(
          "should handle ExportSpecifier::Namespace by ExportAllOrNamedAll::NamedAll in block_pre_walk_export_all_declaration"
        ),
        ExportSpecifier::Default(s) => {
          let exported = parser.ast.get_atom(s.exported(ast).sym(ast));
          f(
            parser,
            JS_DEFAULT_KEYWORD.clone(),
            exported,
            s.exported(ast).span(ast),
          );
        }
        ExportSpecifier::Named(n) => {
          let ast = &parser.ast;
          let exported_name = n.exported(ast).unwrap_or(n.orig(ast));
          let orig = match n.orig(ast) {
            ModuleExportName::Ident(ident) => ast.get_atom(ident.sym(ast)),
            ModuleExportName::Str(s) => {
              ast.get_wtf8_atom(s.value(ast)).to_atom_lossy().into_owned()
            }
          };
          let exported_name_str = match exported_name {
            ModuleExportName::Ident(ident) => ast.get_atom(ident.sym(ast)),
            ModuleExportName::Str(s) => {
              ast.get_wtf8_atom(s.value(ast)).to_atom_lossy().into_owned()
            }
          };
          f(parser, orig, exported_name_str, exported_name.span(ast));
        }
      }
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportDefaultDeclaration {
  /// `export default class X {}`
  /// `export default class {}`
  /// `export default function x() {}`
  /// `export default function () {}`
  Decl(ExportDefaultDecl),
  /// `export default (class X {})`
  /// `export default 'x'`
  Expr(ExportDefaultExpr),
}

impl Spanned for ExportDefaultDeclaration {
  #[inline]
  fn span(&self, ast: &Ast) -> Span {
    match self {
      ExportDefaultDeclaration::Decl(decl) => decl.span(ast),
      ExportDefaultDeclaration::Expr(expr) => expr.span(ast),
    }
  }

  #[inline]
  fn set_span(&mut self, ast: &mut Ast, span: Span) {
    match self {
      ExportDefaultDeclaration::Decl(decl) => decl.set_span(ast, span),
      ExportDefaultDeclaration::Expr(expr) => expr.set_span(ast, span),
    }
  }
}

impl ExportDefaultDeclaration {
  pub fn declaration_span(&self, ast: &Ast) -> Span {
    match self {
      ExportDefaultDeclaration::Decl(decl) => decl.decl(ast).span(ast),
      ExportDefaultDeclaration::Expr(expr) => expr.expr(ast).span(ast),
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportDefaultExpression {
  /// `export default function () {}`
  FnDecl(FnExpr),
  /// `export default class {}`
  ClassDecl(ClassExpr),
  /// `export default (class {})`
  /// `export default 'x'`
  Expr(Expr),
}

impl Spanned for ExportDefaultExpression {
  #[inline]
  fn span(&self, ast: &Ast) -> Span {
    match self {
      ExportDefaultExpression::FnDecl(f) => f.span(ast),
      ExportDefaultExpression::ClassDecl(c) => c.span(ast),
      ExportDefaultExpression::Expr(e) => e.span(ast),
    }
  }

  #[inline]
  fn set_span(&mut self, ast: &mut Ast, span: Span) {
    match self {
      ExportDefaultExpression::FnDecl(f) => f.set_span(ast, span),
      ExportDefaultExpression::ClassDecl(c) => c.set_span(ast, span),
      ExportDefaultExpression::Expr(e) => e.set_span(ast, span),
    }
  }
}

impl ExportDefaultExpression {
  pub fn ident(&self, ast: &Ast) -> Option<Utf8Ref> {
    match self {
      ExportDefaultExpression::FnDecl(f) => f.ident(ast).map(|ident| ident.sym(ast)),
      ExportDefaultExpression::ClassDecl(c) => c.ident(ast).map(|ident| ident.sym(ast)),
      ExportDefaultExpression::Expr(_) => None,
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportImport {
  All(ExportAllDeclaration),
  Named(ExportNamedDeclaration),
}

impl Spanned for ExportImport {
  #[inline]
  fn span(&self, ast: &Ast) -> Span {
    match self {
      ExportImport::All(all) => all.span(ast),
      ExportImport::Named(named) => named.span(ast),
    }
  }

  #[inline]
  fn set_span(&mut self, ast: &mut Ast, span: Span) {
    match self {
      ExportImport::All(all) => all.set_span(ast, span),
      ExportImport::Named(named) => named.set_span(ast, span),
    }
  }
}

impl ExportImport {
  pub fn source(&self, ast: &Ast) -> Atom {
    match self {
      ExportImport::All(e) => e.source(ast).into(),
      ExportImport::Named(e) => e
        .source(ast)
        .expect("ExportImport::Named (export { x } from 'm') should have src"),
    }
  }

  pub fn source_span(&self, ast: &Ast) -> Span {
    match self {
      ExportImport::All(all) => all.source_span(ast),
      ExportImport::Named(named) => named
        .source_span(ast)
        .expect("ExportImport::Named (export { x } from 'm') should have src"),
    }
  }

  pub fn get_with_obj(&self, ast: &Ast) -> Option<ObjectLit> {
    match self {
      ExportImport::All(e) => e.get_with_obj(ast),
      ExportImport::Named(e) => e.get_with_obj(ast),
    }
  }

  pub fn is_star_export(&self) -> bool {
    matches!(self, ExportImport::All(ExportAllDeclaration::All(_)))
  }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportLocal {
  Named(ExportNamedDeclaration),
  Default(ExportDefaultDeclaration),
}

impl Spanned for ExportLocal {
  #[inline]
  fn span(&self, ast: &Ast) -> Span {
    match self {
      ExportLocal::Named(decl) => decl.span(ast),
      ExportLocal::Default(decl) => decl.span(ast),
    }
  }

  #[inline]
  fn set_span(&mut self, ast: &mut Ast, span: Span) {
    match self {
      ExportLocal::Named(decl) => decl.set_span(ast, span),
      ExportLocal::Default(decl) => decl.set_span(ast, span),
    }
  }
}

impl ExportLocal {
  pub fn declaration_span(&self, ast: &Ast) -> Option<Span> {
    match self {
      ExportLocal::Named(named) => named.declaration_span(ast),
      ExportLocal::Default(default) => Some(default.declaration_span(ast)),
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct MaybeNamedFunctionDecl {
  span: Span,
  ident: Option<Ident>,
  function: Function,
}

impl Spanned for MaybeNamedFunctionDecl {
  #[inline]
  fn span(&self, _ast: &Ast) -> Span {
    self.span
  }

  #[inline]
  fn set_span(&mut self, _ast: &mut Ast, span: Span) {
    self.span = span;
  }
}

impl MaybeNamedFunctionDecl {
  pub(crate) fn from_fn_decl(value: FnDecl, ast: &Ast) -> Self {
    Self {
      span: value.span(ast),
      ident: Some(value.ident(ast)),
      function: value.function(ast),
    }
  }
}

impl MaybeNamedFunctionDecl {
  pub(crate) fn from_fn_expr(f: FnExpr, ast: &Ast) -> Self {
    Self {
      span: f.span(ast),
      ident: f.ident(ast),
      function: f.function(ast),
    }
  }
}

impl MaybeNamedFunctionDecl {
  pub fn ident(&self) -> Option<Ident> {
    self.ident
  }

  pub fn function(&self) -> Function {
    self.function
  }
}

#[derive(Debug, Clone, Copy)]
pub struct MaybeNamedClassDecl {
  span: Span,
  ident: Option<Ident>,
  class: Class,
}

impl Spanned for MaybeNamedClassDecl {
  #[inline]
  fn span(&self, _ast: &Ast) -> Span {
    self.span
  }

  #[inline]
  fn set_span(&mut self, _ast: &mut Ast, span: Span) {
    self.span = span;
  }
}

impl MaybeNamedClassDecl {
  pub(crate) fn from_class_decl(value: ClassDecl, ast: &Ast) -> Self {
    Self {
      span: value.span(ast),
      ident: Some(value.ident(ast)),
      class: value.class(ast),
    }
  }
}

impl MaybeNamedClassDecl {
  pub(crate) fn from_class_expr(value: ClassExpr, ast: &Ast) -> Self {
    Self {
      span: value.span(ast),
      ident: value.ident(ast),
      class: value.class(ast),
    }
  }
}

impl MaybeNamedClassDecl {
  pub fn ident(&self) -> Option<Ident> {
    self.ident
  }

  pub fn class(&self) -> Class {
    self.class
  }
}

#[derive(Debug, Clone, Copy)]
pub enum Statement {
  Block(BlockStmt),
  Empty(EmptyStmt),
  Debugger(DebuggerStmt),
  With(WithStmt),
  Return(ReturnStmt),
  Labeled(LabeledStmt),
  Break(BreakStmt),
  Continue(ContinueStmt),
  If(IfStmt),
  Switch(SwitchStmt),
  Throw(ThrowStmt),
  Try(TryStmt),
  While(WhileStmt),
  DoWhile(DoWhileStmt),
  For(ForStmt),
  ForIn(ForInStmt),
  ForOf(ForOfStmt),
  Expr(ExprStmt),
  // ClassDecl, don't put ClassExpr into it, unless it's DefaultDecl::ClassExpr
  // which is represented by ClassExpr but it actually is a ClassDecl without ident
  Class(MaybeNamedClassDecl),
  // FnDecl, don't put FnExpr into it, unless it's DefaultDecl::FnExpr
  // which is represented by FnExpr but it actually is a FnDecl without ident
  Fn(MaybeNamedFunctionDecl),
  Var(VariableDeclaration),
}

impl Spanned for Statement {
  #[inline]
  fn span(&self, ast: &Ast) -> Span {
    use Statement::*;
    match self {
      Block(d) => d.span(ast),
      Empty(d) => d.span(ast),
      Debugger(d) => d.span(ast),
      With(d) => d.span(ast),
      Return(d) => d.span(ast),
      Labeled(d) => d.span(ast),
      Break(d) => d.span(ast),
      Continue(d) => d.span(ast),
      If(d) => d.span(ast),
      Switch(d) => d.span(ast),
      Throw(d) => d.span(ast),
      Try(d) => d.span(ast),
      While(d) => d.span(ast),
      DoWhile(d) => d.span(ast),
      For(d) => d.span(ast),
      ForIn(d) => d.span(ast),
      ForOf(d) => d.span(ast),
      Expr(d) => d.span(ast),
      Class(d) => d.span(ast),
      Fn(d) => d.span(ast),
      Var(d) => d.span(ast),
    }
  }

  #[inline]
  fn set_span(&mut self, ast: &mut Ast, span: Span) {
    use Statement::*;
    match self {
      Block(d) => d.set_span(ast, span),
      Empty(d) => d.set_span(ast, span),
      Debugger(d) => d.set_span(ast, span),
      With(d) => d.set_span(ast, span),
      Return(d) => d.set_span(ast, span),
      Labeled(d) => d.set_span(ast, span),
      Break(d) => d.set_span(ast, span),
      Continue(d) => d.set_span(ast, span),
      If(d) => d.set_span(ast, span),
      Switch(d) => d.set_span(ast, span),
      Throw(d) => d.set_span(ast, span),
      Try(d) => d.set_span(ast, span),
      While(d) => d.set_span(ast, span),
      DoWhile(d) => d.set_span(ast, span),
      For(d) => d.set_span(ast, span),
      ForIn(d) => d.set_span(ast, span),
      ForOf(d) => d.set_span(ast, span),
      Expr(d) => d.set_span(ast, span),
      Class(d) => d.set_span(ast, span),
      Fn(d) => d.set_span(ast, span),
      Var(d) => d.set_span(ast, span),
    }
  }
}

impl Statement {
  pub(crate) fn from_stmt(value: Stmt, ast: &Ast) -> Self {
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
      Stmt::Decl(d) => Self::from_decl(d, ast),
    }
  }
}

impl Statement {
  pub(crate) fn from_decl(value: Decl, ast: &Ast) -> Self {
    use Statement::*;
    match value {
      Decl::Class(d) => Class(MaybeNamedClassDecl::from_class_decl(d, ast)),
      Decl::Fn(d) => Fn(MaybeNamedFunctionDecl::from_fn_decl(d, ast)),
      Decl::Var(d) => Var(VariableDeclaration::VarDecl(d)),
      Decl::Using(d) => Var(VariableDeclaration::UsingDecl(d)),
    }
  }
}

impl Statement {
  pub fn as_function_decl(&self) -> Option<MaybeNamedFunctionDecl> {
    match self {
      Statement::Fn(f) => Some(*f),
      _ => None,
    }
  }

  pub fn as_class_decl(&self) -> Option<MaybeNamedClassDecl> {
    match self {
      Statement::Class(c) => Some(*c),
      _ => None,
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum VariableDeclaration {
  VarDecl(VarDecl),
  UsingDecl(UsingDecl),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VariableDeclarationKind {
  Var,
  Let,
  Const,
  Using,
  AwaitUsing,
}

impl Spanned for VariableDeclaration {
  #[inline]
  fn span(&self, ast: &Ast) -> Span {
    match self {
      VariableDeclaration::VarDecl(var_decl) => var_decl.span(ast),
      VariableDeclaration::UsingDecl(using_decl) => using_decl.span(ast),
    }
  }

  #[inline]
  fn set_span(&mut self, ast: &mut Ast, span: Span) {
    match self {
      VariableDeclaration::VarDecl(var_decl) => var_decl.set_span(ast, span),
      VariableDeclaration::UsingDecl(using_decl) => using_decl.set_span(ast, span),
    }
  }
}

impl VariableDeclaration {
  pub fn kind(&self, ast: &Ast) -> VariableDeclarationKind {
    match self {
      VariableDeclaration::VarDecl(v) => match v.kind(ast) {
        VarDeclKind::Var => VariableDeclarationKind::Var,
        VarDeclKind::Let => VariableDeclarationKind::Let,
        VarDeclKind::Const => VariableDeclarationKind::Const,
      },
      VariableDeclaration::UsingDecl(u) => {
        if u.is_await(ast) {
          VariableDeclarationKind::AwaitUsing
        } else {
          VariableDeclarationKind::Using
        }
      }
    }
  }

  pub fn declarators(&self, ast: &Ast) -> TypedSubRange<VarDeclarator> {
    match self {
      VariableDeclaration::VarDecl(v) => v.decls(ast),
      VariableDeclaration::UsingDecl(u) => u.decls(ast),
    }
  }
}
