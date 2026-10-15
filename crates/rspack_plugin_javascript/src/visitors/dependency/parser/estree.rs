//! Domain-level classifications used by the JavaScript parser hooks.
//!
//! Every payload is a SWC Next typed handle. Callers must use the parser-owned
//! [`Ast`] passed alongside the handle to read fields.

use rspack_intern::Atom;
use rspack_util::swc::AstSubRangeExt;
use swc_next_ecma_ast::{
  Ast, BindingPattern, BlockStatement, BreakStatement, Class, ContinueStatement, DebuggerStatement,
  DoWhileStatement, EmptyStatement, ExportAllDeclaration as SwcExportAllDeclaration,
  ExportDefaultDeclaration as SwcExportDefaultDeclaration, ExportDefaultDeclarationKindData,
  ExportNamedDeclaration as SwcExportNamedDeclaration, Expr, ExpressionStatement, ForInStatement,
  ForOfStatement, ForStatement, FormalParameterPattern, FormalParameterPatternData,
  FormalParameters, Function, GetSpan, IfStatement, ImportAttribute, LabeledStatement,
  ModuleExportName, ModuleExportNameData, NodeKindData, ReturnStatement, Span, Stmt, SubRange,
  SwitchStatement, ThrowStatement, TryStatement, VariableDeclaration, WhileStatement,
  WithStatement,
};

fn wtf8_to_atom(value: &swc_next_allocator::wtf8::Wtf8) -> Atom {
  Atom::from(value.to_string_lossy().as_ref())
}

/// Convert a formal parameter pattern into the `BindingPattern` shape used by
/// the parser plugins.
///
/// SWC Next models formal parameters with their own node kinds, so the binding
/// forms have to be projected back into `BindingPattern`.
pub fn formal_parameter_pattern(ast: &Ast<'_>, pattern: FormalParameterPattern) -> BindingPattern {
  match ast.formal_parameter_pattern_data(pattern) {
    FormalParameterPatternData::BindingIdentifier(node) => BindingPattern::BindingIdentifier(node),
    FormalParameterPatternData::ArrayPattern(node) => BindingPattern::ArrayPattern(node),
    FormalParameterPatternData::ObjectPattern(node) => BindingPattern::ObjectPattern(node),
  }
}

/// Iterate the binding patterns represented by a function's formal parameters,
/// together with the default value of each parameter.
///
/// SWC Next stores ordinary parameters and the rest parameter separately. Keeping
/// this detail here avoids reproducing the same reconstruction in parser plugins.
/// The rest parameter is represented by its argument pattern because callers only
/// consumed the former rest element to reach that pattern.
pub fn formal_parameter_patterns_with_defaults<'a>(
  ast: &'a Ast<'_>,
  params: FormalParameters,
) -> impl Iterator<Item = (BindingPattern, Option<Expr>)> + 'a {
  let rest = params
    .rest(ast)
    .map(|rest| (formal_parameter_pattern(ast, rest.argument(ast)), None));
  ast
    .nodes(params.items(ast))
    .filter_map(move |item| item.as_formal_parameter(ast))
    .map(move |parameter| {
      (
        formal_parameter_pattern(ast, parameter.pattern(ast)),
        parameter.initializer(ast),
      )
    })
    .chain(rest)
}

/// Iterate the binding patterns represented by a function's formal parameters.
pub fn formal_parameter_patterns<'a>(
  ast: &'a Ast<'_>,
  params: FormalParameters,
) -> impl Iterator<Item = BindingPattern> + 'a {
  formal_parameter_patterns_with_defaults(ast, params).map(|(pattern, _)| pattern)
}

/// Returns whether all formal parameters are plain identifier bindings.
///
/// This intentionally rejects rest parameters, destructuring and default values.
/// Those cases need their parameter initialization semantics to be analyzed
/// separately.
pub fn formal_parameters_are_simple_identifiers(ast: &Ast<'_>, params: FormalParameters) -> bool {
  params.rest(ast).is_none()
    && ast.nodes(params.items(ast)).all(|item| {
      item.as_formal_parameter(ast).is_some_and(|parameter| {
        parameter.initializer(ast).is_none()
          && parameter.pattern(ast).as_binding_identifier(ast).is_some()
      })
    })
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

  pub fn attributes(self, ast: &Ast<'_>) -> SubRange<ImportAttribute> {
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

  pub fn attributes(self, ast: &Ast<'_>) -> SubRange<ImportAttribute> {
    self.0.attributes(ast)
  }

  pub fn named_export_specifiers<'a>(
    self,
    ast: &'a Ast<'_>,
  ) -> impl Iterator<Item = (Atom, Atom, Span)> + 'a {
    ast.nodes(self.0.specifiers(ast)).map(move |specifier| {
      let local = specifier.local(ast);
      let exported = specifier.exported(ast);
      (
        module_export_name_to_atom(ast, local),
        module_export_name_to_atom(ast, exported),
        exported.span(ast),
      )
    })
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

  pub fn attributes(self, ast: &Ast<'_>) -> SubRange<ImportAttribute> {
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
  Class(Class),
  Fn(Function),
  Var(VariableDeclaration),
  Other(Stmt),
}

impl Statement {
  /// Identifies the underlying AST node without decoding its source span.
  pub fn node_id(self) -> swc_next_ecma_ast::NodeId {
    match self {
      Self::Block(node) => node.node_id(),
      Self::Empty(node) => node.node_id(),
      Self::Debugger(node) => node.node_id(),
      Self::With(node) => node.node_id(),
      Self::Return(node) => node.node_id(),
      Self::Labeled(node) => node.node_id(),
      Self::Break(node) => node.node_id(),
      Self::Continue(node) => node.node_id(),
      Self::If(node) => node.node_id(),
      Self::Switch(node) => node.node_id(),
      Self::Throw(node) => node.node_id(),
      Self::Try(node) => node.node_id(),
      Self::While(node) => node.node_id(),
      Self::DoWhile(node) => node.node_id(),
      Self::For(node) => node.node_id(),
      Self::ForIn(node) => node.node_id(),
      Self::ForOf(node) => node.node_id(),
      Self::Expr(node) => node.node_id(),
      Self::Class(node) => node.node_id(),
      Self::Fn(node) => node.node_id(),
      Self::Var(node) => node.node_id(),
      Self::Other(node) => node.node_id(),
    }
  }

  pub fn from_stmt(ast: &Ast<'_>, statement: Stmt) -> Self {
    match ast.kind_data(statement.node_id()) {
      NodeKindData::BlockStatement(node) => Self::Block(node),
      NodeKindData::EmptyStatement(node) => Self::Empty(node),
      NodeKindData::DebuggerStatement(node) => Self::Debugger(node),
      NodeKindData::WithStatement(node) => Self::With(node),
      NodeKindData::ReturnStatement(node) => Self::Return(node),
      NodeKindData::LabeledStatement(node) => Self::Labeled(node),
      NodeKindData::BreakStatement(node) => Self::Break(node),
      NodeKindData::ContinueStatement(node) => Self::Continue(node),
      NodeKindData::IfStatement(node) => Self::If(node),
      NodeKindData::SwitchStatement(node) => Self::Switch(node),
      NodeKindData::ThrowStatement(node) => Self::Throw(node),
      NodeKindData::TryStatement(node) => Self::Try(node),
      NodeKindData::WhileStatement(node) => Self::While(node),
      NodeKindData::DoWhileStatement(node) => Self::DoWhile(node),
      NodeKindData::ForStatement(node) => Self::For(node),
      NodeKindData::ForInStatement(node) => Self::ForIn(node),
      NodeKindData::ForOfStatement(node) => Self::ForOf(node),
      NodeKindData::ExpressionStatement(node) => Self::Expr(node),
      NodeKindData::Class(node) => Self::Class(node),
      NodeKindData::Function(node) => Self::Fn(node),
      NodeKindData::VariableDeclaration(node) => Self::Var(node),
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

  pub fn as_function_decl(self) -> Option<Function> {
    match self {
      Self::Fn(node) => Some(node),
      _ => None,
    }
  }

  pub fn as_class_decl(self) -> Option<Class> {
    match self {
      Self::Class(node) => Some(node),
      _ => None,
    }
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
