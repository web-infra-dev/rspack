use std::{borrow::Cow, sync::LazyLock};

use rspack_core::{
  DeferredPureCheck, Dependency, DependencyRange, ModuleDependency, SideEffectsBailoutItemWithSpan,
};
use rspack_intern::{AtomRef, AtomSet};
use rspack_util::{
  SpanExt,
  swc::{AstSubRangeExt, RspackComments},
};
use rustc_hash::FxHashSet;
use swc_next_ecma_ast::{
  ArgumentData, ArrowFunctionBodyData, ArrowFunctionExpression, AssignmentExpression,
  AssignmentOperator, AssignmentTargetData, Ast, BindingPattern, BindingPatternData,
  CallExpression, Class, ClassElement, ClassElementData, CommentKind, Decl, DeclData,
  ExportDefaultDeclarationKindData, Expr, ExprData, ForStatementInitData, FormalParameterItemData,
  FormalParameterPatternData, Function, GetSpan, MethodDefinitionKind, ModuleExportName,
  ModuleExportNameData, Program, PropertyKey, PropertyKeyData, ScopeId, SimpleAssignmentTargetData,
  SourceType, Span, Stmt, StmtData, VariableDeclaration as AstVariableDeclaration, VariableKind,
};
use swc_next_ecma_semantic::Semantic;

use super::side_effects_analysis::{SideEffectsContext, may_have_side_effects};
use crate::{
  Atom, JavascriptParserPlugin,
  dependency::ESMImportSideEffectDependency,
  parser_plugin::esm_import_dependency_parser_plugin::{ESM_SPECIFIER_TAG, ESMSpecifierData},
  visitors::{JavascriptParser, Statement, TagInfoData, formal_parameters_are_simple_identifiers},
};

static PURE_COMMENTS: LazyLock<regex::Regex> = LazyLock::new(|| {
  regex::Regex::new("(?s)^\\s*(#|@)__PURE__(?:\\s|$)").expect("Should create the regex")
});

pub struct SideEffectsParserPlugin {
  analyze_side_effects_free: bool,
}

impl SideEffectsParserPlugin {
  pub fn new(analyze_side_effects_free: bool) -> Self {
    Self {
      analyze_side_effects_free,
    }
  }
}

fn atom_from_binding(ast: &Ast<'_>, ident: swc_next_ecma_ast::BindingIdentifier) -> Atom {
  Atom::from(ast.get_utf8(ident.name(ast)))
}

fn module_export_name<'ast>(ast: &'ast Ast<'_>, name: ModuleExportName) -> Cow<'ast, str> {
  match ast.module_export_name_data(name) {
    ModuleExportNameData::IdentifierName(identifier) => {
      Cow::Borrowed(ast.get_utf8(identifier.name(ast)))
    }
    ModuleExportNameData::StringLiteral(string) => {
      ast.get_wtf8(string.value(ast)).to_string_lossy()
    }
  }
}

fn has_no_side_effects_notation(comments: &RspackComments<'_>, span: Span) -> bool {
  comments.has_flag(span.start, "NO_SIDE_EFFECTS")
}

fn expr_ctx<'a, 'parser>(
  parser: &'a JavascriptParser<'parser>,
  is_unresolved_ref_safe: bool,
) -> SideEffectsContext<'a, 'parser> {
  SideEffectsContext::new(parser.ast.ast, parser.ast.semantic, is_unresolved_ref_safe)
}

fn has_pure_comment(comments: &RspackComments<'_>, pos: u32) -> bool {
  comments.leading.get(&pos).is_some_and(|comment_list| {
    comment_list
      .iter()
      .any(|comment| comment.kind == CommentKind::Block && PURE_COMMENTS.is_match(comment.text))
  })
}

fn top_level_scope(ast: &Ast<'_>) -> ScopeId {
  match ast.source_type() {
    SourceType::Script => ScopeId::ROOT,
    SourceType::Module | SourceType::CommonJs => ScopeId::MODULE,
    SourceType::Unambiguous => unreachable!("parser should resolve unambiguous source type"),
  }
}

fn collect_defined_configured_side_effects_free(
  ast: &Ast<'_>,
  program: Program,
  configured: &[String],
  semantic: &Semantic<'_>,
) -> AtomSet {
  let scope = top_level_scope(ast);
  let configured_names = configured
    .iter()
    .map(String::as_str)
    .collect::<FxHashSet<_>>();
  let mut defined = configured
    .iter()
    .filter(|name| semantic.binding(scope, name.as_bytes()).is_some())
    .map(|name| Atom::from(name.as_str()))
    .collect::<AtomSet>();
  if configured_names.is_empty() {
    return defined;
  }
  for statement in ast.nodes(program.body(ast)) {
    match ast.stmt_data(statement) {
      StmtData::ExportNamedDeclaration(export) if export.source(ast).is_none() => {
        for specifier in ast.nodes(export.specifiers(ast)) {
          let exported = module_export_name(ast, specifier.exported(ast));
          if !configured_names.contains(exported.as_ref()) {
            continue;
          }
          let local = module_export_name(ast, specifier.local(ast));
          if semantic.binding(scope, local.as_bytes()).is_some() {
            defined.insert(Atom::from(exported));
          }
        }
      }
      StmtData::ExportDefaultDeclaration(export) if configured_names.contains("default") => {
        let is_function = match ast.export_default_declaration_kind_data(export.declaration(ast)) {
          ExportDefaultDeclarationKindData::Function(_) => true,
          ExportDefaultDeclarationKindData::Expr(expression) => matches!(
            ast.expr_data(expression),
            ExprData::Function(_) | ExprData::ArrowFunctionExpression(_)
          ),
          _ => false,
        };
        if is_function {
          defined.insert(Atom::from("default"));
        }
      }
      _ => {}
    }
  }
  defined
}

fn collect_annotation_from_variable(
  ast: &Ast<'_>,
  comments: &RspackComments<'_>,
  variable: AstVariableDeclaration,
  container_span: Option<Span>,
  side_effects_free: &mut AtomSet,
) {
  if variable.kind(ast) != VariableKind::Const || variable.declarators(ast).len() != 1 {
    return;
  }
  let Some(declarator) = ast.first(variable.declarators(ast)) else {
    return;
  };
  let BindingPatternData::BindingIdentifier(identifier) =
    ast.binding_pattern_data(declarator.id(ast))
  else {
    return;
  };
  let Some(initializer) = declarator.init(ast) else {
    return;
  };
  if !matches!(
    ast.expr_data(initializer),
    ExprData::Function(_) | ExprData::ArrowFunctionExpression(_)
  ) {
    return;
  }
  if has_no_side_effects_notation(comments, variable.span(ast))
    || has_no_side_effects_notation(comments, initializer.span(ast))
    || container_span.is_some_and(|span| has_no_side_effects_notation(comments, span))
  {
    side_effects_free.insert(atom_from_binding(ast, identifier));
  }
}

fn collect_pure_annotations(
  ast: &Ast<'_>,
  comments: &RspackComments<'_>,
  program: Program,
) -> AtomSet {
  let mut side_effects_free = AtomSet::default();
  for statement in ast.nodes(program.body(ast)) {
    match ast.stmt_data(statement) {
      StmtData::Declaration(declaration) => match ast.decl_data(declaration) {
        DeclData::Function(function) => {
          if has_no_side_effects_notation(comments, function.span(ast))
            && let Some(identifier) = function.id(ast)
          {
            side_effects_free.insert(atom_from_binding(ast, identifier));
          }
        }
        DeclData::VariableDeclaration(variable) => {
          collect_annotation_from_variable(ast, comments, variable, None, &mut side_effects_free)
        }
        _ => {}
      },
      StmtData::ExportNamedDeclaration(export) => {
        let Some(declaration) = export.declaration(ast) else {
          continue;
        };
        match ast.decl_data(declaration) {
          DeclData::Function(function) => {
            if (has_no_side_effects_notation(comments, export.span(ast))
              || has_no_side_effects_notation(comments, function.span(ast)))
              && let Some(identifier) = function.id(ast)
            {
              side_effects_free.insert(atom_from_binding(ast, identifier));
            }
          }
          DeclData::VariableDeclaration(variable) => collect_annotation_from_variable(
            ast,
            comments,
            variable,
            Some(export.span(ast)),
            &mut side_effects_free,
          ),
          _ => {}
        }
      }
      StmtData::ExportDefaultDeclaration(export) => {
        let default_name = Atom::from("default");
        match ast.export_default_declaration_kind_data(export.declaration(ast)) {
          ExportDefaultDeclarationKindData::Function(function)
            if has_no_side_effects_notation(comments, export.span(ast))
              || has_no_side_effects_notation(comments, function.span(ast)) =>
          {
            if let Some(identifier) = function.id(ast) {
              side_effects_free.insert(atom_from_binding(ast, identifier));
            }
            side_effects_free.insert(default_name);
          }
          ExportDefaultDeclarationKindData::Expr(expression) => match ast.expr_data(expression) {
            ExprData::Function(function)
              if has_no_side_effects_notation(comments, export.span(ast))
                || has_no_side_effects_notation(comments, function.span(ast)) =>
            {
              if let Some(identifier) = function.id(ast) {
                side_effects_free.insert(atom_from_binding(ast, identifier));
              }
              side_effects_free.insert(default_name);
            }
            ExprData::ArrowFunctionExpression(arrow)
              if has_no_side_effects_notation(comments, export.span(ast))
                || has_no_side_effects_notation(comments, arrow.span(ast)) =>
            {
              side_effects_free.insert(default_name);
            }
            _ => {}
          },
          _ => {}
        }
      }
      _ => {}
    }
  }
  side_effects_free
}

fn mark_side_effects_free(parser: &mut JavascriptParser, name: &str, export_name: Option<&str>) {
  let side_effects_free = parser.build_info.side_effects_free.get_or_insert_default();
  side_effects_free.insert(Atom::from(name));
  if let Some(export_name) = export_name {
    side_effects_free.insert(Atom::from(export_name));
  }
}

fn already_marked_or_duplicate(
  parser: &JavascriptParser,
  name: &str,
  identifier: swc_next_ecma_ast::BindingIdentifier,
) -> bool {
  parser
    .ast
    .semantic
    .symbol_of(identifier.node_id())
    .is_some_and(|symbol| parser.ast.semantic.declarations(symbol).len() > 1)
    || parser
      .build_info
      .side_effects_free
      .as_ref()
      .is_some_and(|side_effects_free| side_effects_free.contains(name))
}

fn try_mark_auto_side_effects_free_variable(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  variable: AstVariableDeclaration,
  export_name: Option<&str>,
) {
  let ast = parser.ast.ast;
  if variable.kind(ast) != VariableKind::Const {
    return;
  }
  for declarator in ast.nodes(variable.declarators(ast)) {
    let BindingPatternData::BindingIdentifier(identifier) =
      ast.binding_pattern_data(declarator.id(ast))
    else {
      continue;
    };
    let name = ast.get_utf8(identifier.name(ast));
    if already_marked_or_duplicate(parser, name, identifier) {
      continue;
    }
    let Some(initializer) = declarator.init(ast) else {
      continue;
    };
    let is_side_effects_free = match ast.expr_data(initializer) {
      ExprData::Function(function) => {
        is_side_effects_free_function_body(parser, analyze_side_effects_free, function)
      }
      ExprData::ArrowFunctionExpression(arrow) => {
        is_side_effects_free_arrow_body(parser, analyze_side_effects_free, arrow)
      }
      _ => false,
    };
    if is_side_effects_free {
      mark_side_effects_free(parser, name, export_name);
    }
  }
}

fn try_mark_auto_side_effects_free_decl(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  declaration: Decl,
  export_name: Option<&str>,
) {
  let ast = parser.ast.ast;
  match ast.decl_data(declaration) {
    DeclData::Function(function) => {
      let Some(identifier) = function.id(ast) else {
        return;
      };
      let name = ast.get_utf8(identifier.name(ast));
      if !already_marked_or_duplicate(parser, name, identifier)
        && is_side_effects_free_function_body(parser, analyze_side_effects_free, function)
      {
        mark_side_effects_free(parser, name, export_name);
      }
    }
    DeclData::VariableDeclaration(variable) => try_mark_auto_side_effects_free_variable(
      parser,
      analyze_side_effects_free,
      variable,
      export_name,
    ),
    _ => {}
  }
}

fn mark_auto_side_effects_free_program(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  program: Program,
) {
  let ast = parser.ast.ast;
  for statement in ast.nodes(program.body(ast)) {
    match ast.stmt_data(statement) {
      StmtData::Declaration(declaration) => {
        try_mark_auto_side_effects_free_decl(parser, analyze_side_effects_free, declaration, None)
      }
      StmtData::ExportNamedDeclaration(export) => {
        if let Some(declaration) = export.declaration(ast) {
          try_mark_auto_side_effects_free_decl(
            parser,
            analyze_side_effects_free,
            declaration,
            None,
          );
        }
      }
      StmtData::ExportDefaultDeclaration(export) => {
        match ast.export_default_declaration_kind_data(export.declaration(ast)) {
          ExportDefaultDeclarationKindData::Function(function) => {
            let Some(identifier) = function.id(ast) else {
              continue;
            };
            let name = ast.get_utf8(identifier.name(ast));
            if !already_marked_or_duplicate(parser, name, identifier)
              && is_side_effects_free_function_body(parser, analyze_side_effects_free, function)
            {
              mark_side_effects_free(parser, name, Some("default"));
            }
          }
          ExportDefaultDeclarationKindData::Expr(expression) => {
            if let ExprData::Function(function) = ast.expr_data(expression)
              && let Some(identifier) = function.id(ast)
            {
              let name = ast.get_utf8(identifier.name(ast));
              if !already_marked_or_duplicate(parser, name, identifier)
                && is_side_effects_free_function_body(parser, analyze_side_effects_free, function)
              {
                mark_side_effects_free(parser, name, Some("default"));
              }
            }
          }
          _ => {}
        }
      }
      _ => {}
    }
  }
}

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for SideEffectsParserPlugin {
  fn program(&self, parser: &mut JavascriptParser<'p>, program: Program) -> Option<bool> {
    parser.build_info.side_effects_free = None;
    parser.build_info.deferred_pure_checks.clear();

    if self.analyze_side_effects_free {
      let ast = parser.ast.ast;
      let detected = collect_pure_annotations(ast, parser.ast.comments, program);
      if !detected.is_empty() {
        parser
          .build_info
          .side_effects_free
          .get_or_insert_default()
          .extend(detected);
      }

      if let Some(configured) = &parser.javascript_options.side_effects_free {
        let defined = collect_defined_configured_side_effects_free(
          ast,
          program,
          configured,
          parser.ast.semantic,
        );
        if !defined.is_empty() {
          parser
            .build_info
            .side_effects_free
            .get_or_insert_default()
            .extend(defined);
        }
      }

      loop {
        let previous_len = parser
          .build_info
          .side_effects_free
          .as_ref()
          .map_or(0, |side_effects_free| side_effects_free.len());
        mark_auto_side_effects_free_program(parser, self.analyze_side_effects_free, program);
        let next_len = parser
          .build_info
          .side_effects_free
          .as_ref()
          .map_or(0, |side_effects_free| side_effects_free.len());
        if next_len == previous_len {
          break;
        }
      }
    }
    None
  }

  fn module_declaration(&self, parser: &mut JavascriptParser<'p>, statement: Stmt) -> Option<bool> {
    let ast = parser.ast.ast;
    match ast.stmt_data(statement) {
      StmtData::ExportDefaultDeclaration(export) => {
        if let ExportDefaultDeclarationKindData::Expr(expression) =
          ast.export_default_declaration_kind_data(export.declaration(ast))
        {
          let mut callees = Vec::new();
          if !is_pure_expression(
            parser,
            self.analyze_side_effects_free,
            expression,
            parser.ast.comments,
            Some(&mut callees),
          ) {
            set_side_effects_bailout(parser, export.span(ast), "ExportDefaultExpr");
          } else {
            process_deferred_callees(parser, callees, "ExportDefaultExpr");
          }
        }
      }
      StmtData::ExportNamedDeclaration(export) => {
        if let Some(declaration) = export.declaration(ast) {
          let mut callees = Vec::new();
          if !is_pure_decl(
            parser,
            self.analyze_side_effects_free,
            declaration,
            parser.ast.comments,
            Some(&mut callees),
          ) {
            set_side_effects_bailout(parser, declaration.span(ast), "Decl");
          }
          if parser.side_effects_item.is_none() {
            process_deferred_callees(parser, callees, "Decl");
          }
        }
      }
      _ => {}
    }
    None
  }

  fn statement(&self, parser: &mut JavascriptParser<'p>, statement: Statement) -> Option<bool> {
    if parser.is_top_level_scope() {
      self.analyze_stmt_side_effects(statement, parser);
    }
    None
  }

  fn finish(&self, parser: &mut JavascriptParser<'p>) -> Option<bool> {
    if !self.analyze_side_effects_free {
      return None;
    }
    let mut not_defined = Vec::new();
    if let Some(configured) = &parser.javascript_options.side_effects_free {
      let mut configured = configured.iter().collect::<Vec<_>>();
      configured.sort();
      let defined = parser.build_info.side_effects_free.as_ref();
      for name in configured {
        if !defined.is_some_and(|defined| defined.contains(name)) {
          not_defined.push(name.as_str());
        }
      }
    }
    if !not_defined.is_empty() {
      if let Some(side_effects_free) = parser.build_info.side_effects_free.as_mut() {
        for name in &not_defined {
          side_effects_free.remove(*name);
        }
      }
      let resource = parser.resource_data.resource();
      parser.add_warning(rspack_error::Diagnostic::warn(
        "PURE_FUNCTION_NOT_FOUND".into(),
        format!(
          "Following pure functions are not found in {resource}:\n[{}]\nRemove it from `module.rules[*].parser.pureFunctions`",
          not_defined
            .iter()
            .map(|name| format!("`{name}`"))
            .collect::<Vec<_>>()
            .join(", ")
        ),
      ));
    }
    None
  }
}

fn set_side_effects_bailout(parser: &mut JavascriptParser, span: Span, kind: &str) {
  let range = DependencyRange::from(span);
  let location = parser.to_dependency_location(range);
  parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
    range,
    location,
    kind.to_string(),
  ));
}

fn process_deferred_callees(parser: &mut JavascriptParser, callees: Vec<(Atom, Span)>, kind: &str) {
  for (callee, span) in callees {
    if let Some(deferred_check) = try_extract_deferred_check(parser, &callee, span) {
      parser
        .build_info
        .deferred_pure_checks
        .insert(deferred_check);
    } else {
      set_side_effects_bailout(parser, span, kind);
      break;
    }
  }
}

enum ExplicitSideEffectsFreeCallee {
  Direct,
  Deferred,
  Invalid,
  NotMarked,
}

fn resolve_explicit_side_effects_free_callee(
  parser: &mut JavascriptParser,
  identifier: swc_next_ecma_ast::IdentifierReference,
  ident: &str,
  span: Span,
  allow_unresolved_marked: bool,
) -> ExplicitSideEffectsFreeCallee {
  let is_marked = parser
    .build_info
    .side_effects_free
    .as_ref()
    .is_some_and(|side_effects_free| side_effects_free.contains(ident));
  if !is_marked {
    return ExplicitSideEffectsFreeCallee::NotMarked;
  }

  if try_extract_deferred_check(parser, ident, span).is_some() {
    let is_user_configured = parser
      .javascript_options
      .side_effects_free
      .as_ref()
      .is_some_and(|names| names.iter().any(|name| name == ident));
    if !is_user_configured {
      return ExplicitSideEffectsFreeCallee::Deferred;
    }
  }

  if parser.active_synthetic_ast.is_none() {
    let reference = parser
      .ast
      .semantic
      .reference_of(identifier.node_id())
      .map(|reference| parser.ast.semantic.reference(reference));
    return match reference {
      Some(reference) if reference.flags.is_dynamic() => ExplicitSideEffectsFreeCallee::Invalid,
      Some(reference) if reference.symbol.is_some() => {
        if reference.symbol.is_some_and(|symbol| {
          parser.ast.semantic.scope_of(symbol) == top_level_scope(parser.ast.ast)
        }) {
          ExplicitSideEffectsFreeCallee::Direct
        } else {
          ExplicitSideEffectsFreeCallee::Invalid
        }
      }
      _ if allow_unresolved_marked => ExplicitSideEffectsFreeCallee::Direct,
      _ => ExplicitSideEffectsFreeCallee::Invalid,
    };
  }

  // Separately parsed replacement expressions resolve names in the caller's environment.
  if let Some((declared_scope, is_free)) = parser
    .get_variable_info(ident)
    .map(|info| (info.declared_scope, info.is_free()))
  {
    if !is_free && declared_scope == parser.definitions_db.current_scope() {
      return ExplicitSideEffectsFreeCallee::Direct;
    }
    return ExplicitSideEffectsFreeCallee::Invalid;
  }
  if allow_unresolved_marked {
    ExplicitSideEffectsFreeCallee::Direct
  } else {
    ExplicitSideEffectsFreeCallee::Invalid
  }
}

fn try_extract_deferred_check<'key>(
  parser: &mut JavascriptParser,
  ident: impl Into<AtomRef<'key>>,
  span: Span,
) -> Option<DeferredPureCheck> {
  let info = parser.get_variable_info(ident)?;
  let tag_info_id = info.tag_info?;
  let tag_info = parser.definitions_db.expect_get_tag_info(tag_info_id);
  if tag_info.tag != ESM_SPECIFIER_TAG {
    return None;
  }
  let data = ESMSpecifierData::downcast(tag_info.data.clone()?);
  parser
    .get_dependencies()
    .iter()
    .find(|dependency| {
      let Some(dependency) = dependency.downcast_ref::<ESMImportSideEffectDependency>() else {
        return false;
      };
      dependency.request() == data.source && data.attributes.as_ref() == dependency.get_attributes()
    })
    .map(|dependency| DeferredPureCheck {
      atom: data
        .ids
        .first()
        .cloned()
        .unwrap_or_else(|| data.name.clone()),
      dep_id: *dependency.id(),
      start: span.real_lo(),
      end: span.real_hi(),
    })
}

fn arguments_are_pure(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  arguments: swc_next_ecma_ast::TypedSubRange<swc_next_ecma_ast::Argument>,
  comments: &RspackComments<'_>,
  mut callees: Option<&mut Vec<(Atom, Span)>>,
) -> bool {
  let ast = parser.ast.ast;
  for argument in ast.nodes(arguments) {
    let ArgumentData::Expr(expression) = ast.argument_data(argument) else {
      return false;
    };
    if !is_pure_expression(
      parser,
      analyze_side_effects_free,
      expression,
      comments,
      callees.as_deref_mut(),
    ) {
      return false;
    }
  }
  true
}

fn is_pure_call_expression(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  expression: Expr,
  call: CallExpression,
  comments: &RspackComments<'_>,
  callees: Option<&mut Vec<(Atom, Span)>>,
) -> bool {
  let ast = parser.ast.ast;
  let callee = call.callee(ast);
  if has_pure_comment(comments, expression.span(ast).start)
    || has_pure_comment(comments, callee.span(ast).start)
  {
    return arguments_are_pure(
      parser,
      analyze_side_effects_free,
      call.arguments(ast),
      comments,
      callees,
    );
  }

  if analyze_side_effects_free && let Some(identifier) = callee.as_identifier_reference(ast) {
    let name = ast.get_utf8(identifier.name(ast));
    match resolve_explicit_side_effects_free_callee(
      parser,
      identifier,
      name,
      callee.span(ast),
      callees.is_none(),
    ) {
      ExplicitSideEffectsFreeCallee::Direct => {
        return arguments_are_pure(
          parser,
          analyze_side_effects_free,
          call.arguments(ast),
          comments,
          callees,
        );
      }
      ExplicitSideEffectsFreeCallee::Deferred => {
        let Some(callees) = callees else {
          return false;
        };
        callees.push((Atom::from(name), callee.span(ast)));
        return arguments_are_pure(
          parser,
          analyze_side_effects_free,
          call.arguments(ast),
          comments,
          Some(callees),
        );
      }
      ExplicitSideEffectsFreeCallee::Invalid => return false,
      ExplicitSideEffectsFreeCallee::NotMarked => {}
    }

    if let Some(callees) = callees {
      callees.push((Atom::from(name), callee.span(ast)));
      return arguments_are_pure(
        parser,
        analyze_side_effects_free,
        call.arguments(ast),
        comments,
        Some(callees),
      );
    }
  }

  !may_have_side_effects(expression, expr_ctx(parser, false))
}

fn is_pure_property_key(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  key: PropertyKey,
  comments: &RspackComments<'_>,
  callees: Option<&mut Vec<(Atom, Span)>>,
) -> bool {
  match parser.ast.ast.property_key_data(key) {
    PropertyKeyData::Expr(expression) => is_pure_expression(
      parser,
      analyze_side_effects_free,
      expression,
      comments,
      callees,
    ),
    _ => true,
  }
}

fn evaluated_expression_is_pure(parser: &mut JavascriptParser, expression: Expr) -> bool {
  !parser
    .evaluate_expression(expression)
    .could_have_side_effects()
}

#[inline(never)]
pub fn is_pure_expression(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  expression: Expr,
  comments: &RspackComments<'_>,
  mut callees: Option<&mut Vec<(Atom, Span)>>,
) -> bool {
  if let Some(result) = parser.plugin_drive.clone().is_pure(parser, expression) {
    return result;
  }

  let ast = parser.ast.ast;
  match ast.expr_data(expression) {
    ExprData::ArrayExpression(array) => {
      for element in ast.nodes(array.elements(ast)).flatten() {
        let ArgumentData::Expr(element) = ast.argument_data(element) else {
          return false;
        };
        if !is_pure_expression(
          parser,
          analyze_side_effects_free,
          element,
          comments,
          callees.as_deref_mut(),
        ) {
          return false;
        }
      }
      true
    }
    ExprData::CallExpression(call) => is_pure_call_expression(
      parser,
      analyze_side_effects_free,
      expression,
      call,
      comments,
      callees,
    ),
    ExprData::NewExpression(new_expression) => {
      if has_pure_comment(comments, expression.span(ast).start) {
        arguments_are_pure(
          parser,
          analyze_side_effects_free,
          new_expression.arguments(ast),
          comments,
          None,
        )
      } else {
        !may_have_side_effects(expression, expr_ctx(parser, false))
      }
    }
    ExprData::ParenthesizedExpression(parenthesized) => is_pure_expression(
      parser,
      analyze_side_effects_free,
      parenthesized.expression(ast),
      comments,
      callees,
    ),
    ExprData::SequenceExpression(sequence) => {
      ast.nodes(sequence.expressions(ast)).all(|expression| {
        is_pure_expression(
          parser,
          analyze_side_effects_free,
          expression,
          comments,
          callees.as_deref_mut(),
        )
      })
    }
    ExprData::TsAsExpression(ts) => is_pure_expression(
      parser,
      analyze_side_effects_free,
      ts.expression(ast),
      comments,
      callees,
    ),
    ExprData::TsSatisfiesExpression(ts) => is_pure_expression(
      parser,
      analyze_side_effects_free,
      ts.expression(ast),
      comments,
      callees,
    ),
    ExprData::TsTypeAssertion(ts) => is_pure_expression(
      parser,
      analyze_side_effects_free,
      ts.expression(ast),
      comments,
      callees,
    ),
    ExprData::TsNonNullExpression(ts) => is_pure_expression(
      parser,
      analyze_side_effects_free,
      ts.expression(ast),
      comments,
      callees,
    ),
    ExprData::TsInstantiationExpression(ts) => is_pure_expression(
      parser,
      analyze_side_effects_free,
      ts.expression(ast),
      comments,
      callees,
    ),
    _ => {
      if !may_have_side_effects(expression, expr_ctx(parser, true)) {
        true
      } else {
        evaluated_expression_is_pure(parser, expression)
      }
    }
  }
}

pub fn is_pure_pat(parser: &mut JavascriptParser, pattern: BindingPattern) -> bool {
  let ast = parser.ast.ast;
  match ast.binding_pattern_data(pattern) {
    BindingPatternData::BindingIdentifier(_) | BindingPatternData::BindingRestElement(_) => true,
    BindingPatternData::ArrayPattern(array) => ast
      .nodes(array.elements(ast))
      .flatten()
      .all(|element| is_pure_pat(parser, element)),
    BindingPatternData::SimpleAssignmentTarget(target) => {
      target.as_identifier_reference(ast).is_some()
    }
    BindingPatternData::AssignmentPattern(_) | BindingPatternData::ObjectPattern(_) => false,
  }
}

pub fn is_pure_function(parser: &mut JavascriptParser, function: Function) -> bool {
  let ast = parser.ast.ast;
  let parameters = function.params(ast);
  for item in ast.nodes(parameters.items(ast)) {
    let FormalParameterItemData::FormalParameter(parameter) = ast.formal_parameter_item_data(item)
    else {
      return false;
    };
    let FormalParameterPatternData::BindingPattern(pattern) =
      ast.formal_parameter_pattern_data(parameter.pattern(ast))
    else {
      return false;
    };
    if !is_pure_pat(parser, pattern) {
      return false;
    }
  }
  true
}

pub fn is_pure_class_member(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  member: ClassElement,
  comments: &RspackComments<'_>,
  mut callees: Option<&mut Vec<(Atom, Span)>>,
) -> bool {
  let ast = parser.ast.ast;
  match ast.class_element_data(member) {
    ClassElementData::MethodDefinition(method) => is_pure_property_key(
      parser,
      analyze_side_effects_free,
      method.key(ast),
      comments,
      callees,
    ),
    ClassElementData::TsMethodDefinition(method) => is_pure_property_key(
      parser,
      analyze_side_effects_free,
      method.key(ast),
      comments,
      callees,
    ),
    ClassElementData::PropertyDefinition(property) => {
      if !is_pure_property_key(
        parser,
        analyze_side_effects_free,
        property.key(ast),
        comments,
        callees.as_deref_mut(),
      ) {
        return false;
      }
      !property.r#static(ast)
        || property.value(ast).is_none_or(|value| {
          is_pure_expression(parser, analyze_side_effects_free, value, comments, callees)
        })
    }
    ClassElementData::StaticBlock(_) => false,
    ClassElementData::TsIndexSignature(_) => true,
  }
}

pub fn is_pure_class(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  class: Class,
  comments: &RspackComments<'_>,
  mut callees: Option<&mut Vec<(Atom, Span)>>,
) -> bool {
  let ast = parser.ast.ast;
  if let Some(super_class) = class.super_class(ast)
    && !is_pure_expression(
      parser,
      analyze_side_effects_free,
      super_class,
      comments,
      callees.as_deref_mut(),
    )
  {
    return false;
  }

  for member in ast.nodes(class.body(ast).body(ast)) {
    let pure = match ast.class_element_data(member) {
      ClassElementData::MethodDefinition(method) => {
        (method.kind(ast) != MethodDefinitionKind::Constructor || class.super_class(ast).is_none())
          && is_pure_property_key(
            parser,
            analyze_side_effects_free,
            method.key(ast),
            comments,
            callees.as_deref_mut(),
          )
      }
      ClassElementData::TsMethodDefinition(method) => is_pure_property_key(
        parser,
        analyze_side_effects_free,
        method.key(ast),
        comments,
        callees.as_deref_mut(),
      ),
      ClassElementData::PropertyDefinition(property) => {
        is_pure_property_key(
          parser,
          analyze_side_effects_free,
          property.key(ast),
          comments,
          callees.as_deref_mut(),
        ) && (!property.r#static(ast)
          || property.value(ast).is_none_or(|value| {
            is_pure_expression(
              parser,
              analyze_side_effects_free,
              value,
              comments,
              callees.as_deref_mut(),
            )
          }))
      }
      ClassElementData::StaticBlock(_) => false,
      ClassElementData::TsIndexSignature(_) => true,
    };
    if !pure {
      return false;
    }
  }
  true
}

pub fn is_pure_decl(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  declaration: Decl,
  comments: &RspackComments<'_>,
  callees: Option<&mut Vec<(Atom, Span)>>,
) -> bool {
  match parser.ast.ast.decl_data(declaration) {
    DeclData::Class(class) => {
      is_pure_class(parser, analyze_side_effects_free, class, comments, callees)
    }
    DeclData::Function(_) => true,
    DeclData::VariableDeclaration(variable) => is_pure_var_decl(
      parser,
      analyze_side_effects_free,
      variable,
      comments,
      callees,
    ),
    DeclData::TsFunction(_)
    | DeclData::TsTypeAliasDeclaration(_)
    | DeclData::TsInterfaceDeclaration(_)
    | DeclData::TsGlobalDeclaration(_) => true,
    _ => false,
  }
}

fn is_pure_var_decl(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  variable: AstVariableDeclaration,
  comments: &RspackComments<'_>,
  mut callees: Option<&mut Vec<(Atom, Span)>>,
) -> bool {
  if matches!(
    variable.kind(parser.ast.ast),
    VariableKind::Using | VariableKind::AwaitUsing
  ) {
    return false;
  }
  let ast = parser.ast.ast;
  ast.nodes(variable.declarators(ast)).all(|declarator| {
    declarator.init(ast).is_none_or(|initializer| {
      is_pure_expression(
        parser,
        analyze_side_effects_free,
        initializer,
        comments,
        callees.as_deref_mut(),
      )
    })
  })
}

/// Recognize a top-level CommonJS export assignment whose target write is
/// ignored only for module-evaluation side-effect analysis. The RHS is still
/// judged by the regular parser purity behavior.
fn is_common_js_export_assignment(
  parser: &mut JavascriptParser,
  assignment: AssignmentExpression,
) -> bool {
  let ast = parser.ast.ast;
  if parser.is_esm
    || !parser.is_top_level_scope()
    || assignment.operator(ast) != AssignmentOperator::Assign
  {
    return false;
  }

  let AssignmentTargetData::SimpleAssignmentTarget(target) =
    ast.assignment_target_data(assignment.left(ast))
  else {
    return false;
  };
  let SimpleAssignmentTargetData::MemberExpression(member) =
    ast.simple_assignment_target_data(target)
  else {
    return false;
  };
  let Some(identifier) = member.object(ast).as_identifier_reference(ast) else {
    return false;
  };
  let name = ast.get_utf8(identifier.name(ast));

  let property_name = match ast.property_key_data(member.property(ast)) {
    PropertyKeyData::IdentifierName(property) if !member.computed(ast) => {
      Some(ast.get_utf8(property.name(ast)))
    }
    PropertyKeyData::StringLiteral(property) if member.computed(ast) => {
      ast.get_wtf8(property.value(ast)).as_str()
    }
    _ => None,
  };

  let property_is_side_effect_free = match name {
    "exports" => property_name.is_some_and(|property| property != "__proto__"),
    "module" => property_name == Some("exports"),
    _ => false,
  };

  property_is_side_effect_free
    && parser
      .get_variable_info(name)
      .is_none_or(|info| info.is_free())
}

/// Keep the CommonJS export-write exception out of `is_pure_expression`,
/// because innerGraph also consumes that shared function.
fn is_module_eval_pure_expression(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  mut expression: Expr,
  comments: &RspackComments<'_>,
  callees: Option<&mut Vec<(Atom, Span)>>,
) -> bool {
  while let ExprData::AssignmentExpression(assignment) = parser.ast.ast.expr_data(expression) {
    if !is_common_js_export_assignment(parser, assignment) {
      break;
    }
    expression = assignment.right(parser.ast.ast);
  }
  is_pure_expression(
    parser,
    analyze_side_effects_free,
    expression,
    comments,
    callees,
  )
}

fn is_module_eval_pure_var_decl(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  variable: AstVariableDeclaration,
  comments: &RspackComments<'_>,
  mut callees: Option<&mut Vec<(Atom, Span)>>,
) -> bool {
  if matches!(
    variable.kind(parser.ast.ast),
    VariableKind::Using | VariableKind::AwaitUsing
  ) {
    return false;
  }
  let ast = parser.ast.ast;
  ast.nodes(variable.declarators(ast)).all(|declarator| {
    declarator.init(ast).is_none_or(|initializer| {
      is_module_eval_pure_expression(
        parser,
        analyze_side_effects_free,
        initializer,
        comments,
        callees.as_deref_mut(),
      )
    })
  })
}

fn is_side_effects_free_var_decl(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  variable: AstVariableDeclaration,
) -> bool {
  let ast = parser.ast.ast;
  if matches!(
    variable.kind(ast),
    VariableKind::Using | VariableKind::AwaitUsing
  ) {
    return false;
  }
  let comments = parser.ast.comments;
  for declarator in ast.nodes(variable.declarators(ast)) {
    if !matches!(
      ast.binding_pattern_data(declarator.id(ast)),
      BindingPatternData::BindingIdentifier(_)
    ) {
      return false;
    }
    if let Some(initializer) = declarator.init(ast)
      && !is_pure_expression(
        parser,
        analyze_side_effects_free,
        initializer,
        comments,
        None,
      )
    {
      return false;
    }
  }
  true
}

fn stmt_may_have_side_effects(parser: &JavascriptParser, statement: Stmt) -> bool {
  let ast = parser.ast.ast;
  let expr_ctx = expr_ctx(parser, true);
  match ast.stmt_data(statement) {
    StmtData::EmptyStatement(_) => false,
    StmtData::ExpressionStatement(expression) => {
      may_have_side_effects(expression.expression(ast), expr_ctx)
    }
    StmtData::ReturnStatement(return_statement) => return_statement
      .argument(ast)
      .is_some_and(|argument| may_have_side_effects(argument, expr_ctx)),
    StmtData::Declaration(declaration) => match ast.decl_data(declaration) {
      DeclData::VariableDeclaration(variable) => {
        if matches!(
          variable.kind(ast),
          VariableKind::Using | VariableKind::AwaitUsing
        ) {
          return true;
        }
        ast.nodes(variable.declarators(ast)).any(|declarator| {
          !matches!(
            ast.binding_pattern_data(declarator.id(ast)),
            BindingPatternData::BindingIdentifier(_)
          ) || declarator
            .init(ast)
            .is_some_and(|initializer| may_have_side_effects(initializer, expr_ctx))
        })
      }
      _ => true,
    },
    _ => true,
  }
}

fn is_side_effects_free_stmt(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  statement: Stmt,
) -> bool {
  if !stmt_may_have_side_effects(parser, statement) {
    return true;
  }

  let ast = parser.ast.ast;
  let comments = parser.ast.comments;
  match ast.stmt_data(statement) {
    StmtData::EmptyStatement(_) => true,
    StmtData::ExpressionStatement(expression) => is_pure_expression(
      parser,
      analyze_side_effects_free,
      expression.expression(ast),
      comments,
      None,
    ),
    StmtData::ReturnStatement(return_statement) => {
      return_statement.argument(ast).is_none_or(|argument| {
        is_pure_expression(parser, analyze_side_effects_free, argument, comments, None)
      })
    }
    StmtData::Declaration(declaration) => match ast.decl_data(declaration) {
      DeclData::VariableDeclaration(variable) => {
        is_side_effects_free_var_decl(parser, analyze_side_effects_free, variable)
      }
      _ => false,
    },
    _ => false,
  }
}

fn is_side_effects_free_function_body(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  function: Function,
) -> bool {
  let ast = parser.ast.ast;
  if !formal_parameters_are_simple_identifiers(ast, function.params(ast)) {
    return false;
  }
  ast
    .nodes(function.body(ast).body(ast))
    .all(|statement| is_side_effects_free_stmt(parser, analyze_side_effects_free, statement))
}

fn is_side_effects_free_arrow_body(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  arrow: ArrowFunctionExpression,
) -> bool {
  let ast = parser.ast.ast;
  if !formal_parameters_are_simple_identifiers(ast, arrow.params(ast)) {
    return false;
  }
  match ast.arrow_function_body_data(arrow.body(ast)) {
    ArrowFunctionBodyData::FunctionBody(body) => ast
      .nodes(body.body(ast))
      .all(|statement| is_side_effects_free_stmt(parser, analyze_side_effects_free, statement)),
    ArrowFunctionBodyData::Expr(expression) => is_pure_expression(
      parser,
      analyze_side_effects_free,
      expression,
      parser.ast.comments,
      None,
    ),
  }
}

impl SideEffectsParserPlugin {
  fn analyze_stmt_side_effects(&self, statement: Statement, parser: &mut JavascriptParser) {
    if parser.side_effects_item.is_some() {
      return;
    }
    let ast = parser.ast.ast;
    let comments = parser.ast.comments;
    let mut callees = Vec::new();
    let pure = match statement {
      Statement::If(statement) => is_pure_expression(
        parser,
        self.analyze_side_effects_free,
        statement.test(ast),
        comments,
        Some(&mut callees),
      ),
      Statement::While(statement) => is_pure_expression(
        parser,
        self.analyze_side_effects_free,
        statement.test(ast),
        comments,
        Some(&mut callees),
      ),
      Statement::DoWhile(statement) => is_pure_expression(
        parser,
        self.analyze_side_effects_free,
        statement.test(ast),
        comments,
        Some(&mut callees),
      ),
      Statement::For(statement) => {
        let init_pure =
          statement
            .init(ast)
            .is_none_or(|init| match ast.for_statement_init_data(init) {
              ForStatementInitData::VariableDeclaration(variable) => is_module_eval_pure_var_decl(
                parser,
                self.analyze_side_effects_free,
                variable,
                comments,
                Some(&mut callees),
              ),
              ForStatementInitData::Expr(expression) => is_pure_expression(
                parser,
                self.analyze_side_effects_free,
                expression,
                comments,
                Some(&mut callees),
              ),
            });
        init_pure
          && statement.test(ast).is_none_or(|test| {
            is_pure_expression(
              parser,
              self.analyze_side_effects_free,
              test,
              comments,
              Some(&mut callees),
            )
          })
          && statement.update(ast).is_none_or(|update| {
            is_pure_expression(
              parser,
              self.analyze_side_effects_free,
              update,
              comments,
              Some(&mut callees),
            )
          })
      }
      Statement::Expr(statement) => is_module_eval_pure_expression(
        parser,
        self.analyze_side_effects_free,
        statement.expression(ast),
        comments,
        Some(&mut callees),
      ),
      Statement::Switch(statement) => is_pure_expression(
        parser,
        self.analyze_side_effects_free,
        statement.discriminant(ast),
        comments,
        Some(&mut callees),
      ),
      Statement::Class(statement) => is_pure_class(
        parser,
        self.analyze_side_effects_free,
        statement.class(),
        comments,
        Some(&mut callees),
      ),
      Statement::Var(statement) => is_module_eval_pure_var_decl(
        parser,
        self.analyze_side_effects_free,
        statement.0,
        comments,
        Some(&mut callees),
      ),
      Statement::Empty(_) | Statement::Labeled(_) | Statement::Block(_) | Statement::Fn(_) => true,
      _ => false,
    };
    if !pure {
      set_side_effects_bailout(parser, statement.span(ast), "Statement");
    } else {
      process_deferred_callees(parser, callees, "Statement");
    }
  }
}
