//! Domain-level classifications used by the JavaScript parser hooks.
//!
//! Every payload is a SWC Next typed handle. Callers must use the parser-owned
//! [`Ast`] passed alongside the handle to read fields.

use rspack_intern::Atom;
use swc_next_ecma_ast::{
  Ast, BindingIdentifier, BlockStatement, BreakStatement, Class, ContinueStatement,
  DebuggerStatement, DeclData, DoWhileStatement, EmptyStatement,
  ExportAllDeclaration as SwcExportAllDeclaration,
  ExportDefaultDeclaration as SwcExportDefaultDeclaration, ExportDefaultDeclarationKindData,
  ExportNamedDeclaration as SwcExportNamedDeclaration, Expr, ExpressionStatement, ForInStatement,
  ForOfStatement, ForStatement, Function, GetSpan, IfStatement, ImportAttribute, LabeledStatement,
  ModuleExportName, ModuleExportNameData, ReturnStatement, Span, Stmt, StmtData, SwitchStatement,
  ThrowStatement, TryStatement, TypedSubRange, VariableDeclaration as SwcVariableDeclaration,
  VariableDeclarator, VariableKind, WhileStatement, WithStatement,
};

fn wtf8_to_atom(value: &swc_next_allocator::wtf8::Wtf8) -> Atom {
  Atom::from(value.to_string_lossy().as_ref())
}

#[derive(Debug, Clone, Copy)]
pub enum ClassDeclOrExpr {
  Decl(Class),
  Expr(Class),
}

impl ClassDeclOrExpr {
  pub fn node(self) -> Class {
    match self {
      Self::Decl(node) | Self::Expr(node) => node,
    }
  }

  pub fn span(self, ast: &Ast<'_>) -> Span {
    self.node().span(ast)
  }

  pub fn ident(self, ast: &Ast<'_>) -> Option<BindingIdentifier> {
    self.node().id(ast)
  }
}

#[derive(Debug, Clone, Copy)]
pub struct ExportAllDeclaration(pub SwcExportAllDeclaration);

impl ExportAllDeclaration {
  pub fn span(self, ast: &Ast<'_>) -> Span {
    self.0.span(ast)
  }

  pub fn source(self, ast: &Ast<'_>) -> Atom {
    let source = self.0.source(ast);
    wtf8_to_atom(ast.get_wtf8(source.value(ast)))
  }

  pub fn exported_name_span(self, ast: &Ast<'_>) -> Option<Span> {
    self.0.exported(ast).map(|name| name.span(ast))
  }

  pub fn exported_name(self, ast: &Ast<'_>) -> Option<Atom> {
    self
      .0
      .exported(ast)
      .map(|name| module_export_name_to_atom(ast, name))
  }

  pub fn attributes(self, ast: &Ast<'_>) -> TypedSubRange<ImportAttribute> {
    self.0.attributes(ast)
  }

  pub fn is_star_export(self, ast: &Ast<'_>) -> bool {
    self.0.exported(ast).is_none()
  }
}

#[derive(Debug, Clone, Copy)]
pub struct ExportNamedDeclaration(pub SwcExportNamedDeclaration);

impl ExportNamedDeclaration {
  pub fn span(self, ast: &Ast<'_>) -> Span {
    self.0.span(ast)
  }

  pub fn source(self, ast: &Ast<'_>) -> Option<Atom> {
    self
      .0
      .source(ast)
      .map(|source| wtf8_to_atom(ast.get_wtf8(source.value(ast))))
  }

  pub fn declaration_span(self, ast: &Ast<'_>) -> Option<Span> {
    self
      .0
      .declaration(ast)
      .map(|declaration| declaration.span(ast))
  }

  pub fn attributes(self, ast: &Ast<'_>) -> TypedSubRange<ImportAttribute> {
    self.0.attributes(ast)
  }

  pub fn named_export_specifiers(self, ast: &Ast<'_>) -> Vec<(Atom, Atom, Span)> {
    self
      .0
      .specifiers(ast)
      .iter()
      .map(|slot| {
        let specifier = ast.get_node_in_sub_range(slot);
        let local = specifier.local(ast);
        let exported = specifier.exported(ast);
        (
          module_export_name_to_atom(ast, local),
          module_export_name_to_atom(ast, exported),
          exported.span(ast),
        )
      })
      .collect()
  }
}

#[derive(Debug, Clone, Copy)]
pub struct ExportDefaultDeclaration(pub SwcExportDefaultDeclaration);

impl ExportDefaultDeclaration {
  pub fn span(self, ast: &Ast<'_>) -> Span {
    self.0.span(ast)
  }

  pub fn declaration_span(self, ast: &Ast<'_>) -> Span {
    self.0.declaration(ast).span(ast)
  }

  pub fn expression(self, ast: &Ast<'_>) -> ExportDefaultExpression {
    match ast.export_default_declaration_kind_data(self.0.declaration(ast)) {
      ExportDefaultDeclarationKindData::Function(node) => ExportDefaultExpression::FnDecl(node),
      ExportDefaultDeclarationKindData::Class(node) => ExportDefaultExpression::ClassDecl(node),
      ExportDefaultDeclarationKindData::Expr(node) => ExportDefaultExpression::Expr(node),
      _ => ExportDefaultExpression::Other(self.0.declaration(ast).node_id()),
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportDefaultExpression {
  FnDecl(Function),
  ClassDecl(Class),
  Expr(Expr),
  Other(swc_next_ecma_ast::NodeId),
}

impl ExportDefaultExpression {
  pub fn span(self, ast: &Ast<'_>) -> Span {
    match self {
      Self::FnDecl(node) => node.span(ast),
      Self::ClassDecl(node) => node.span(ast),
      Self::Expr(node) => node.span(ast),
      Self::Other(node) => ast.span(node),
    }
  }

  pub fn ident(self, ast: &Ast<'_>) -> Option<Atom> {
    match self {
      Self::FnDecl(node) => node.id(ast),
      Self::ClassDecl(node) => node.id(ast),
      Self::Expr(_) | Self::Other(_) => None,
    }
    .map(|identifier| Atom::from(ast.get_utf8(identifier.name(ast))))
  }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportImport {
  All(ExportAllDeclaration),
  Named(ExportNamedDeclaration),
}

impl ExportImport {
  pub fn span(self, ast: &Ast<'_>) -> Span {
    match self {
      Self::All(node) => node.span(ast),
      Self::Named(node) => node.span(ast),
    }
  }

  pub fn source(self, ast: &Ast<'_>) -> Atom {
    match self {
      Self::All(node) => node.source(ast),
      Self::Named(node) => node
        .source(ast)
        .expect("re-export declaration must have a source"),
    }
  }

  pub fn attributes(self, ast: &Ast<'_>) -> TypedSubRange<ImportAttribute> {
    match self {
      Self::All(node) => node.attributes(ast),
      Self::Named(node) => node.attributes(ast),
    }
  }

  pub fn is_star_export(self, ast: &Ast<'_>) -> bool {
    matches!(self, Self::All(node) if node.is_star_export(ast))
  }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportLocal {
  Named(ExportNamedDeclaration),
  Default(ExportDefaultDeclaration),
}

impl ExportLocal {
  pub fn span(self, ast: &Ast<'_>) -> Span {
    match self {
      Self::Named(node) => node.span(ast),
      Self::Default(node) => node.span(ast),
    }
  }

  pub fn declaration_span(self, ast: &Ast<'_>) -> Option<Span> {
    match self {
      Self::Named(node) => node.declaration_span(ast),
      Self::Default(node) => Some(node.declaration_span(ast)),
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct MaybeNamedFunctionDecl(pub Function);

impl MaybeNamedFunctionDecl {
  pub fn span(self, ast: &Ast<'_>) -> Span {
    self.0.span(ast)
  }

  pub fn ident(self, ast: &Ast<'_>) -> Option<BindingIdentifier> {
    self.0.id(ast)
  }

  pub fn function(self) -> Function {
    self.0
  }
}

#[derive(Debug, Clone, Copy)]
pub struct MaybeNamedClassDecl(pub Class);

impl MaybeNamedClassDecl {
  pub fn span(self, ast: &Ast<'_>) -> Span {
    self.0.span(ast)
  }

  pub fn ident(self, ast: &Ast<'_>) -> Option<BindingIdentifier> {
    self.0.id(ast)
  }

  pub fn class(self) -> Class {
    self.0
  }
}

#[derive(Debug, Clone, Copy)]
pub enum Statement {
  Block(BlockStatement),
  Empty(EmptyStatement),
  Debugger(DebuggerStatement),
  With(WithStatement),
  Return(ReturnStatement),
  Labeled(LabeledStatement),
  Break(BreakStatement),
  Continue(ContinueStatement),
  If(IfStatement),
  Switch(SwitchStatement),
  Throw(ThrowStatement),
  Try(TryStatement),
  While(WhileStatement),
  DoWhile(DoWhileStatement),
  For(ForStatement),
  ForIn(ForInStatement),
  ForOf(ForOfStatement),
  Expr(ExpressionStatement),
  Class(MaybeNamedClassDecl),
  Fn(MaybeNamedFunctionDecl),
  Var(VariableDeclaration),
  Other(Stmt),
}

impl Statement {
  pub fn from_stmt(ast: &Ast<'_>, statement: Stmt) -> Self {
    match ast.stmt_data(statement) {
      StmtData::BlockStatement(node) => Self::Block(node),
      StmtData::EmptyStatement(node) => Self::Empty(node),
      StmtData::DebuggerStatement(node) => Self::Debugger(node),
      StmtData::WithStatement(node) => Self::With(node),
      StmtData::ReturnStatement(node) => Self::Return(node),
      StmtData::LabeledStatement(node) => Self::Labeled(node),
      StmtData::BreakStatement(node) => Self::Break(node),
      StmtData::ContinueStatement(node) => Self::Continue(node),
      StmtData::IfStatement(node) => Self::If(node),
      StmtData::SwitchStatement(node) => Self::Switch(node),
      StmtData::ThrowStatement(node) => Self::Throw(node),
      StmtData::TryStatement(node) => Self::Try(node),
      StmtData::WhileStatement(node) => Self::While(node),
      StmtData::DoWhileStatement(node) => Self::DoWhile(node),
      StmtData::ForStatement(node) => Self::For(node),
      StmtData::ForInStatement(node) => Self::ForIn(node),
      StmtData::ForOfStatement(node) => Self::ForOf(node),
      StmtData::ExpressionStatement(node) => Self::Expr(node),
      StmtData::Declaration(declaration) => match ast.decl_data(declaration) {
        DeclData::Class(node) => Self::Class(MaybeNamedClassDecl(node)),
        DeclData::Function(node) => Self::Fn(MaybeNamedFunctionDecl(node)),
        DeclData::VariableDeclaration(node) => Self::Var(VariableDeclaration(node)),
        _ => Self::Other(statement),
      },
      _ => Self::Other(statement),
    }
  }

  pub fn span(self, ast: &Ast<'_>) -> Span {
    match self {
      Self::Block(node) => node.span(ast),
      Self::Empty(node) => node.span(ast),
      Self::Debugger(node) => node.span(ast),
      Self::With(node) => node.span(ast),
      Self::Return(node) => node.span(ast),
      Self::Labeled(node) => node.span(ast),
      Self::Break(node) => node.span(ast),
      Self::Continue(node) => node.span(ast),
      Self::If(node) => node.span(ast),
      Self::Switch(node) => node.span(ast),
      Self::Throw(node) => node.span(ast),
      Self::Try(node) => node.span(ast),
      Self::While(node) => node.span(ast),
      Self::DoWhile(node) => node.span(ast),
      Self::For(node) => node.span(ast),
      Self::ForIn(node) => node.span(ast),
      Self::ForOf(node) => node.span(ast),
      Self::Expr(node) => node.span(ast),
      Self::Class(node) => node.span(ast),
      Self::Fn(node) => node.span(ast),
      Self::Var(node) => node.span(ast),
      Self::Other(node) => node.span(ast),
    }
  }

  pub fn as_function_decl(self) -> Option<MaybeNamedFunctionDecl> {
    match self {
      Self::Fn(node) => Some(node),
      _ => None,
    }
  }

  pub fn as_class_decl(self) -> Option<MaybeNamedClassDecl> {
    match self {
      Self::Class(node) => Some(node),
      _ => None,
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct VariableDeclaration(pub SwcVariableDeclaration);

pub type VariableDeclarationKind = VariableKind;

impl VariableDeclaration {
  pub fn span(self, ast: &Ast<'_>) -> Span {
    self.0.span(ast)
  }

  pub fn kind(self, ast: &Ast<'_>) -> VariableKind {
    self.0.kind(ast)
  }

  pub fn declarators(self, ast: &Ast<'_>) -> Vec<VariableDeclarator> {
    self
      .0
      .declarators(ast)
      .iter()
      .map(|slot| ast.get_node_in_sub_range(slot))
      .collect()
  }
}

pub fn module_export_name_to_atom(ast: &Ast<'_>, name: ModuleExportName) -> Atom {
  match ast.module_export_name_data(name) {
    ModuleExportNameData::IdentifierName(identifier) => {
      Atom::from(ast.get_utf8(identifier.name(ast)))
    }
    ModuleExportNameData::StringLiteral(string) => wtf8_to_atom(ast.get_wtf8(string.value(ast))),
  }
}
