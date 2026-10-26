//! Domain-level classifications used by the JavaScript parser hooks.
//!
//! Every payload is a SWC Next typed handle. Callers must use the parser-owned
//! [`Ast`] passed alongside the handle to read fields.

use rspack_intern::Atom;
use rspack_util::swc::AstSubRangeExt;
use swc_next_ecma_ast::{
  Ast, BindingIdentifier, BindingPattern, ExportAllDeclaration as SwcExportAllDeclaration,
  ExportDefaultDeclaration as SwcExportDefaultDeclaration, ExportDefaultDeclarationKind,
  ExportNamedDeclaration as SwcExportNamedDeclaration, Expr, ExprData, FormalParameterPattern,
  FormalParameterPatternData, FormalParameters, GetSpan, IdentifierReference, ImportAttribute,
  JsxMemberExpression, MemberExpression, ModuleExportName, ModuleExportNameData,
  SimpleAssignmentTarget, SimpleAssignmentTargetData, Span, SubRange,
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

  pub fn declaration(self, ast: &Ast<'_>) -> ExportDefaultDeclarationKind {
    self.0.declaration(ast)
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

pub fn module_export_name_to_atom(ast: &Ast<'_>, name: ModuleExportName) -> Atom {
  match ast.module_export_name_data(name) {
    ModuleExportNameData::IdentifierName(identifier) => {
      Atom::from(ast.get_utf8(identifier.name(ast)))
    }
    ModuleExportNameData::StringLiteral(string) => wtf8_to_atom(ast.get_wtf8(string.value(ast))),
  }
}

/// Unwraps TypeScript assertion wrappers (`as`, `satisfies`, type assertion,
/// non-null) to the underlying identifier reference, if any.
///
/// Assignment and update walkers must use this when recording mutations so
/// typed targets such as `(x as number) = 1` still mark `x` as mutated.
pub fn simple_assignment_target_identifier(
  ast: &Ast<'_>,
  target: SimpleAssignmentTarget,
) -> Option<IdentifierReference> {
  match ast.simple_assignment_target_data(target) {
    SimpleAssignmentTargetData::IdentifierReference(identifier) => Some(identifier),
    SimpleAssignmentTargetData::MemberExpression(_) => None,
    SimpleAssignmentTargetData::TsAsExpression(expression) => {
      expr_assignment_target_identifier(ast, expression.expression(ast))
    }
    SimpleAssignmentTargetData::TsSatisfiesExpression(expression) => {
      expr_assignment_target_identifier(ast, expression.expression(ast))
    }
    SimpleAssignmentTargetData::TsTypeAssertion(expression) => {
      expr_assignment_target_identifier(ast, expression.expression(ast))
    }
    SimpleAssignmentTargetData::TsNonNullExpression(expression) => {
      expr_assignment_target_identifier(ast, expression.expression(ast))
    }
  }
}

fn expr_assignment_target_identifier(ast: &Ast<'_>, expr: Expr) -> Option<IdentifierReference> {
  match ast.expr_data(expr) {
    ExprData::IdentifierReference(identifier) => Some(identifier),
    ExprData::TsAsExpression(expression) => {
      expr_assignment_target_identifier(ast, expression.expression(ast))
    }
    ExprData::TsSatisfiesExpression(expression) => {
      expr_assignment_target_identifier(ast, expression.expression(ast))
    }
    ExprData::TsTypeAssertion(expression) => {
      expr_assignment_target_identifier(ast, expression.expression(ast))
    }
    ExprData::TsNonNullExpression(expression) => {
      expr_assignment_target_identifier(ast, expression.expression(ast))
    }
    ExprData::ParenthesizedExpression(expression) => {
      expr_assignment_target_identifier(ast, expression.expression(ast))
    }
    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use rspack_util::swc::AstSubRangeExt;
  use swc_next_allocator::Allocator;
  use swc_next_ecma_ast::{ExprData, Lang, SourceType, StmtData};
  use swc_next_ecma_parser::{CommentMode, Options, Parser, TokenParserConfig};

  use super::simple_assignment_target_identifier;

  #[test]
  fn unwraps_typed_assignment_target_identifiers() {
    let allocator = Allocator::new();
    let source = "\
let x = 0;
(x as number) = 1;
(x!) = 2;
(<number>x) = 3;
(x satisfies number) = 4;
";
    let parsed = Parser::init(
      &allocator,
      source,
      Options {
        source_type: SourceType::Script,
        lang: Lang::Ts,
        preserve_parens: false,
        comments: CommentMode::Flat,
      },
      TokenParserConfig,
    )
    .parse();
    assert!(
      parsed.diagnostics.is_empty(),
      "parse diagnostics: {:?}",
      parsed.diagnostics
    );

    let ast = &parsed.ast;
    let program = ast.root_program();
    let mut names = Vec::new();
    for statement in ast.nodes(program.body(ast)) {
      let StmtData::ExpressionStatement(expression_statement) = ast.stmt_data(statement) else {
        continue;
      };
      let ExprData::AssignmentExpression(assignment) =
        ast.expr_data(expression_statement.expression(ast))
      else {
        continue;
      };
      let Some(simple) = assignment.left(ast).as_simple_assignment_target(ast) else {
        continue;
      };
      // Direct identifier lookup must fail for typed wrappers…
      assert!(
        simple.as_identifier_reference(ast).is_none(),
        "typed target should not be a bare identifier"
      );
      // …but unwrapping recovers the binding name.
      let identifier = simple_assignment_target_identifier(ast, simple)
        .expect("typed assignment target should unwrap to an identifier");
      names.push(ast.get_utf8(identifier.name(ast)));
    }

    assert_eq!(names, ["x", "x", "x", "x"]);
  }
}
