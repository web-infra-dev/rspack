use std::sync::LazyLock;

use rspack_core::{
  DeferredPureCheck, Dependency, DependencyRange, ModuleDependency, SideEffectsBailoutItemWithSpan,
};
use rspack_util::{SpanExt, swc::RspackComments};
use rustc_hash::{FxHashMap, FxHashSet};
use swc_atoms::Atom;
use swc_next_ecma_ast::{
  ArgumentData, ArrowFunctionBodyData, ArrowFunctionExpression, Ast, BindingPattern,
  BindingPatternData, CallExpression, Class, ClassElement, ClassElementData, CommentKind, Decl,
  DeclData, ExportDefaultDeclarationKindData, Expr, ExprData, ForStatementInitData,
  FormalParameterItemData, FormalParameterPatternData, Function, GetSpan,
  ImportDeclarationSpecifierData, MethodDefinitionKind, ModuleExportName, ModuleExportNameData,
  ObjectPropertyKindData, Program, PropertyKey, PropertyKeyData, PropertyKind, Span, Stmt,
  StmtData, UnaryOperator, VariableDeclaration as AstVariableDeclaration, VariableKind,
};

use crate::{
  JavascriptParserPlugin,
  dependency::ESMImportSideEffectDependency,
  parser_plugin::esm_import_dependency_parser_plugin::{ESM_SPECIFIER_TAG, ESMSpecifierData},
  visitors::{JavascriptParser, Statement, TagInfoData},
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

fn atom_from_identifier(ast: &Ast<'_>, ident: swc_next_ecma_ast::IdentifierReference) -> Atom {
  Atom::from(ast.get_utf8(ident.name(ast)))
}

fn atom_from_module_export_name(ast: &Ast<'_>, name: ModuleExportName) -> Atom {
  match ast.module_export_name_data(name) {
    ModuleExportNameData::IdentifierName(identifier) => {
      Atom::from(ast.get_utf8(identifier.name(ast)))
    }
    ModuleExportNameData::StringLiteral(string) => {
      Atom::from(ast.get_wtf8(string.value(ast)).to_string_lossy().as_ref())
    }
  }
}

fn has_no_side_effects_notation(comments: &RspackComments<'_>, span: Span) -> bool {
  comments.has_flag(span.start, "NO_SIDE_EFFECTS")
}

fn has_pure_comment(comments: &RspackComments<'_>, pos: u32) -> bool {
  comments.leading.get(&pos).is_some_and(|comment_list| {
    comment_list
      .iter()
      .any(|comment| comment.kind == CommentKind::Block && PURE_COMMENTS.is_match(comment.text))
  })
}

fn visit_pattern_binding_names(ast: &Ast<'_>, pattern: BindingPattern, f: &mut impl FnMut(Atom)) {
  match ast.binding_pattern_data(pattern) {
    BindingPatternData::BindingIdentifier(identifier) => f(atom_from_binding(ast, identifier)),
    BindingPatternData::ArrayPattern(array) => {
      for slot in array.elements(ast).iter() {
        if let Some(element) = ast.get_node_in_sub_range(slot) {
          visit_pattern_binding_names(ast, element, f);
        }
      }
      if let Some(rest) = array.rest(ast) {
        visit_pattern_binding_names(ast, rest.argument(ast), f);
      }
    }
    BindingPatternData::ObjectPattern(object) => {
      for slot in object.properties(ast).iter() {
        let property = ast.get_node_in_sub_range(slot);
        visit_pattern_binding_names(ast, property.value(ast), f);
      }
      if let Some(rest) = object.rest(ast) {
        visit_pattern_binding_names(ast, rest.argument(ast), f);
      }
    }
    BindingPatternData::AssignmentPattern(assignment) => {
      visit_pattern_binding_names(ast, assignment.left(ast), f);
    }
    BindingPatternData::BindingRestElement(rest) => {
      visit_pattern_binding_names(ast, rest.argument(ast), f);
    }
    BindingPatternData::SimpleAssignmentTarget(_) => {}
  }
}

fn visit_decl_binding_names(ast: &Ast<'_>, declaration: Decl, f: &mut impl FnMut(Atom)) {
  match ast.decl_data(declaration) {
    DeclData::Function(function) => {
      if let Some(identifier) = function.id(ast) {
        f(atom_from_binding(ast, identifier));
      }
    }
    DeclData::Class(class) => {
      if let Some(identifier) = class.id(ast) {
        f(atom_from_binding(ast, identifier));
      }
    }
    DeclData::VariableDeclaration(variable) => {
      for slot in variable.declarators(ast).iter() {
        let declarator = ast.get_node_in_sub_range(slot);
        visit_pattern_binding_names(ast, declarator.id(ast), f);
      }
    }
    _ => {}
  }
}

fn visit_stmt_defined_binding_names(ast: &Ast<'_>, statement: Stmt, f: &mut impl FnMut(Atom)) {
  match ast.stmt_data(statement) {
    StmtData::Declaration(declaration) => visit_decl_binding_names(ast, declaration, f),
    StmtData::ImportDeclaration(import) => {
      for slot in import.specifiers(ast).iter() {
        let specifier = ast.get_node_in_sub_range(slot);
        let local = match ast.import_declaration_specifier_data(specifier) {
          ImportDeclarationSpecifierData::ImportSpecifier(specifier) => specifier.local(ast),
          ImportDeclarationSpecifierData::ImportDefaultSpecifier(specifier) => specifier.local(ast),
          ImportDeclarationSpecifierData::ImportNamespaceSpecifier(specifier) => {
            specifier.local(ast)
          }
        };
        f(atom_from_binding(ast, local));
      }
    }
    StmtData::ExportNamedDeclaration(export) => {
      if let Some(declaration) = export.declaration(ast) {
        visit_decl_binding_names(ast, declaration, f);
      }
    }
    StmtData::ExportDefaultDeclaration(export) => {
      match ast.export_default_declaration_kind_data(export.declaration(ast)) {
        ExportDefaultDeclarationKindData::Function(function) => {
          if let Some(identifier) = function.id(ast) {
            f(atom_from_binding(ast, identifier));
          }
        }
        ExportDefaultDeclarationKindData::Class(class) => {
          if let Some(identifier) = class.id(ast) {
            f(atom_from_binding(ast, identifier));
          }
        }
        ExportDefaultDeclarationKindData::Expr(expression) => match ast.expr_data(expression) {
          ExprData::Function(function) => {
            if let Some(identifier) = function.id(ast) {
              f(atom_from_binding(ast, identifier));
            }
          }
          ExprData::Class(class) => {
            if let Some(identifier) = class.id(ast) {
              f(atom_from_binding(ast, identifier));
            }
          }
          _ => {}
        },
        _ => {}
      }
    }
    _ => {}
  }
}

fn collect_pure_function_acceptable_names(ast: &Ast<'_>, program: Program) -> FxHashSet<Atom> {
  let statements = program.body(ast);
  let mut names = FxHashSet::default();
  for statement in statements
    .iter()
    .map(|slot| ast.get_node_in_sub_range(slot))
  {
    visit_stmt_defined_binding_names(ast, statement, &mut |name| {
      names.insert(name);
    });
  }

  let local_bindings = names.clone();
  for statement in statements
    .iter()
    .map(|slot| ast.get_node_in_sub_range(slot))
  {
    match ast.stmt_data(statement) {
      StmtData::ExportNamedDeclaration(export) if export.source(ast).is_none() => {
        for slot in export.specifiers(ast).iter() {
          let specifier = ast.get_node_in_sub_range(slot);
          let local = atom_from_module_export_name(ast, specifier.local(ast));
          if local_bindings.contains(&local) {
            names.insert(atom_from_module_export_name(ast, specifier.exported(ast)));
          }
        }
      }
      StmtData::ExportDefaultDeclaration(export) => {
        let is_function = match ast.export_default_declaration_kind_data(export.declaration(ast)) {
          ExportDefaultDeclarationKindData::Function(_) => true,
          ExportDefaultDeclarationKindData::Expr(expression) => matches!(
            ast.expr_data(expression),
            ExprData::Function(_) | ExprData::ArrowFunctionExpression(_)
          ),
          _ => false,
        };
        if is_function {
          names.insert(Atom::from("default"));
        }
      }
      _ => {}
    }
  }
  names
}

fn collect_defined_configured_side_effects_free(
  ast: &Ast<'_>,
  program: Program,
  configured_side_effects_free: &[String],
) -> FxHashSet<Atom> {
  let acceptable = collect_pure_function_acceptable_names(ast, program);
  configured_side_effects_free
    .iter()
    .filter_map(|name| {
      let atom = Atom::from(name.clone());
      acceptable.contains(&atom).then_some(atom)
    })
    .collect()
}

fn collect_duplicate_top_level_names(ast: &Ast<'_>, program: Program) -> FxHashSet<Atom> {
  let mut counts = FxHashMap::<Atom, usize>::default();
  for statement in program
    .body(ast)
    .iter()
    .map(|slot| ast.get_node_in_sub_range(slot))
  {
    visit_stmt_defined_binding_names(ast, statement, &mut |name| {
      *counts.entry(name).or_default() += 1;
    });
  }
  counts
    .into_iter()
    .filter_map(|(name, count)| (count > 1).then_some(name))
    .collect()
}

fn collect_annotation_from_variable(
  ast: &Ast<'_>,
  comments: &RspackComments<'_>,
  variable: AstVariableDeclaration,
  container_span: Option<Span>,
  side_effects_free: &mut FxHashSet<Atom>,
) {
  if variable.kind(ast) != VariableKind::Const || variable.declarators(ast).len() != 1 {
    return;
  }
  let Some(declarator) = variable.declarators(ast).get_node(ast, 0) else {
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
) -> FxHashSet<Atom> {
  let mut side_effects_free = FxHashSet::default();
  for statement in program
    .body(ast)
    .iter()
    .map(|slot| ast.get_node_in_sub_range(slot))
  {
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

fn mark_side_effects_free(parser: &mut JavascriptParser, name: &Atom, export_name: Option<&Atom>) {
  let side_effects_free = parser.build_info.side_effects_free.get_or_insert_default();
  side_effects_free.insert(name.clone());
  if let Some(export_name) = export_name {
    side_effects_free.insert(export_name.clone());
  }
}

fn already_marked_or_duplicate(
  parser: &JavascriptParser,
  name: &Atom,
  duplicate_names: &FxHashSet<Atom>,
) -> bool {
  duplicate_names.contains(name)
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
  export_name: Option<&Atom>,
  duplicate_names: &FxHashSet<Atom>,
) {
  let ast = parser.ast.ast;
  if variable.kind(ast) != VariableKind::Const {
    return;
  }
  for declarator in variable
    .declarators(ast)
    .iter()
    .map(|slot| ast.get_node_in_sub_range(slot))
  {
    let BindingPatternData::BindingIdentifier(identifier) =
      ast.binding_pattern_data(declarator.id(ast))
    else {
      continue;
    };
    let name = atom_from_binding(ast, identifier);
    if already_marked_or_duplicate(parser, &name, duplicate_names) {
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
      mark_side_effects_free(parser, &name, export_name);
    }
  }
}

fn try_mark_auto_side_effects_free_decl(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  declaration: Decl,
  export_name: Option<&Atom>,
  duplicate_names: &FxHashSet<Atom>,
) {
  let ast = parser.ast.ast;
  match ast.decl_data(declaration) {
    DeclData::Function(function) => {
      let Some(identifier) = function.id(ast) else {
        return;
      };
      let name = atom_from_binding(ast, identifier);
      if !already_marked_or_duplicate(parser, &name, duplicate_names)
        && is_side_effects_free_function_body(parser, analyze_side_effects_free, function)
      {
        mark_side_effects_free(parser, &name, export_name);
      }
    }
    DeclData::VariableDeclaration(variable) => try_mark_auto_side_effects_free_variable(
      parser,
      analyze_side_effects_free,
      variable,
      export_name,
      duplicate_names,
    ),
    _ => {}
  }
}

fn mark_auto_side_effects_free_program(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  program: Program,
  duplicate_names: &FxHashSet<Atom>,
) {
  let ast = parser.ast.ast;
  for statement in program
    .body(ast)
    .iter()
    .map(|slot| ast.get_node_in_sub_range(slot))
  {
    match ast.stmt_data(statement) {
      StmtData::Declaration(declaration) => try_mark_auto_side_effects_free_decl(
        parser,
        analyze_side_effects_free,
        declaration,
        None,
        duplicate_names,
      ),
      StmtData::ExportNamedDeclaration(export) => {
        if let Some(declaration) = export.declaration(ast) {
          try_mark_auto_side_effects_free_decl(
            parser,
            analyze_side_effects_free,
            declaration,
            None,
            duplicate_names,
          );
        }
      }
      StmtData::ExportDefaultDeclaration(export) => {
        let default_name = Atom::from("default");
        match ast.export_default_declaration_kind_data(export.declaration(ast)) {
          ExportDefaultDeclarationKindData::Function(function) => {
            let Some(identifier) = function.id(ast) else {
              continue;
            };
            let name = atom_from_binding(ast, identifier);
            if !already_marked_or_duplicate(parser, &name, duplicate_names)
              && is_side_effects_free_function_body(parser, analyze_side_effects_free, function)
            {
              mark_side_effects_free(parser, &name, Some(&default_name));
            }
          }
          ExportDefaultDeclarationKindData::Expr(expression) => {
            if let ExprData::Function(function) = ast.expr_data(expression)
              && let Some(identifier) = function.id(ast)
            {
              let name = atom_from_binding(ast, identifier);
              if !already_marked_or_duplicate(parser, &name, duplicate_names)
                && is_side_effects_free_function_body(parser, analyze_side_effects_free, function)
              {
                mark_side_effects_free(parser, &name, Some(&default_name));
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
        let defined = collect_defined_configured_side_effects_free(ast, program, configured);
        if !defined.is_empty() {
          parser
            .build_info
            .side_effects_free
            .get_or_insert_default()
            .extend(defined);
        }
      }

      let duplicate_names = collect_duplicate_top_level_names(ast, program);
      loop {
        let previous_len = parser
          .build_info
          .side_effects_free
          .as_ref()
          .map_or(0, FxHashSet::len);
        mark_auto_side_effects_free_program(
          parser,
          self.analyze_side_effects_free,
          program,
          &duplicate_names,
        );
        let next_len = parser
          .build_info
          .side_effects_free
          .as_ref()
          .map_or(0, FxHashSet::len);
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
        let atom = Atom::from(name.clone());
        if !defined.is_some_and(|defined| defined.contains(&atom)) {
          not_defined.push(atom);
        }
      }
    }
    if !not_defined.is_empty() {
      if let Some(side_effects_free) = parser.build_info.side_effects_free.as_mut() {
        for name in &not_defined {
          side_effects_free.remove(name);
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
    if let Some(deferred_check) = try_extract_deferred_check(parser, callee, span) {
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
  ident: &Atom,
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

  if try_extract_deferred_check(parser, ident.clone(), span).is_some() {
    let is_user_configured = parser
      .javascript_options
      .side_effects_free
      .as_ref()
      .is_some_and(|names| names.iter().any(|name| name.as_str() == ident.as_str()));
    if !is_user_configured {
      return ExplicitSideEffectsFreeCallee::Deferred;
    }
  }

  if let Some((declared_scope, is_free)) = parser
    .get_variable_info(ident)
    .map(|info| (info.declared_scope, info.is_free()))
  {
    if !is_free && declared_scope == parser.definitions {
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

fn try_extract_deferred_check(
  parser: &mut JavascriptParser,
  ident: Atom,
  span: Span,
) -> Option<DeferredPureCheck> {
  let info = parser.get_variable_info(&ident)?;
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
      dependency.request() == &data.source
        && data.attributes.as_ref() == dependency.get_attributes()
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
  for argument in arguments.iter().map(|slot| ast.get_node_in_sub_range(slot)) {
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

fn is_global_reference_to(
  parser: &JavascriptParser,
  identifier: swc_next_ecma_ast::IdentifierReference,
  expected: &str,
) -> bool {
  let ast = parser.ast.ast;
  if ast.get_utf8(identifier.name(ast)) != expected {
    return false;
  }
  parser
    .ast
    .semantic
    .reference_of(identifier.node_id())
    .map(|reference| parser.ast.semantic.reference(reference))
    .is_some_and(|reference| reference.symbol.is_none() && !reference.flags.is_dynamic())
}

fn identifier_expression_name(ast: &Ast<'_>, expression: Expr) -> Option<Atom> {
  let ExprData::IdentifierReference(identifier) = ast.expr_data(expression) else {
    return None;
  };
  Some(atom_from_identifier(ast, identifier))
}

fn property_key_name(ast: &Ast<'_>, key: PropertyKey) -> Option<Atom> {
  match ast.property_key_data(key) {
    PropertyKeyData::IdentifierName(identifier) => {
      Some(Atom::from(ast.get_utf8(identifier.name(ast))))
    }
    PropertyKeyData::StringLiteral(string) => Some(Atom::from(
      ast.get_wtf8(string.value(ast)).to_string_lossy().as_ref(),
    )),
    _ => None,
  }
}

fn is_pure_string_method(name: &str) -> bool {
  matches!(
    name,
    "charAt"
      | "charCodeAt"
      | "concat"
      | "endsWith"
      | "includes"
      | "indexOf"
      | "lastIndexOf"
      | "localeCompare"
      | "slice"
      | "split"
      | "startsWith"
      | "substr"
      | "substring"
      | "toLocaleLowerCase"
      | "toLocaleUpperCase"
      | "toLowerCase"
      | "toString"
      | "toUpperCase"
      | "trim"
      | "trimEnd"
      | "trimStart"
  )
}

fn parameters_are_simple_identifiers(ast: &Ast<'_>, function: Function) -> bool {
  let parameters = function.params(ast);
  if parameters.rest(ast).is_some() {
    return false;
  }
  parameters.items(ast).iter().all(|slot| {
    let item = ast.get_node_in_sub_range(slot);
    let FormalParameterItemData::FormalParameter(parameter) = ast.formal_parameter_item_data(item)
    else {
      return false;
    };
    let FormalParameterPatternData::BindingPattern(pattern) =
      ast.formal_parameter_pattern_data(parameter.pattern(ast))
    else {
      return false;
    };
    matches!(
      ast.binding_pattern_data(pattern),
      BindingPatternData::BindingIdentifier(_)
    )
  })
}

fn is_empty_function(parser: &JavascriptParser, function: Function) -> bool {
  let ast = parser.ast.ast;
  parameters_are_simple_identifiers(ast, function) && function.body(ast).body(ast).is_empty()
}

fn is_pure_callee(parser: &mut JavascriptParser, expression: Expr) -> bool {
  let ast = parser.ast.ast;
  match ast.expr_data(expression) {
    ExprData::IdentifierReference(identifier) => is_global_reference_to(parser, identifier, "Date"),
    ExprData::MemberExpression(member) if !member.computed(ast) => {
      let Some(property) = property_key_name(ast, member.property(ast)) else {
        return false;
      };
      let object = member.object(ast);
      match ast.expr_data(object) {
        ExprData::IdentifierReference(identifier) => {
          ast.get_utf8(identifier.name(ast)) == "Math"
            || is_global_reference_to(parser, identifier, "Math")
        }
        ExprData::StringLiteral(_) => is_pure_string_method(property.as_str()),
        ExprData::TemplateLiteral(template) if template.expressions(ast).is_empty() => {
          is_pure_string_method(property.as_str())
        }
        _ => false,
      }
    }
    ExprData::Function(function) => is_empty_function(parser, function),
    _ => false,
  }
}

fn is_pure_new_callee(parser: &mut JavascriptParser, expression: Expr) -> bool {
  let ast = parser.ast.ast;
  match ast.expr_data(expression) {
    ExprData::Function(function) => is_empty_function(parser, function),
    ExprData::Class(class) => {
      if class.super_class(ast).is_some() || !is_pure_class_definition_expression(parser, class) {
        return false;
      }
      for element in class
        .body(ast)
        .body(ast)
        .iter()
        .map(|slot| ast.get_node_in_sub_range(slot))
      {
        match ast.class_element_data(element) {
          ClassElementData::PropertyDefinition(property) if !property.r#static(ast) => {
            return false;
          }
          ClassElementData::MethodDefinition(method)
            if method.kind(ast) == MethodDefinitionKind::Constructor
              && !method.value(ast).body(ast).body(ast).is_empty() =>
          {
            return false;
          }
          _ => {}
        }
      }
      true
    }
    _ => false,
  }
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

  if analyze_side_effects_free && let Some(name) = identifier_expression_name(ast, callee) {
    match resolve_explicit_side_effects_free_callee(
      parser,
      &name,
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
        callees.push((name, callee.span(ast)));
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
      callees.push((name, callee.span(ast)));
      return arguments_are_pure(
        parser,
        analyze_side_effects_free,
        call.arguments(ast),
        comments,
        Some(callees),
      );
    }
  }

  is_pure_callee(parser, callee)
    && arguments_are_pure(
      parser,
      analyze_side_effects_free,
      call.arguments(ast),
      comments,
      None,
    )
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

fn is_pure_object_expression(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  object: swc_next_ecma_ast::ObjectExpression,
  comments: &RspackComments<'_>,
  mut callees: Option<&mut Vec<(Atom, Span)>>,
) -> bool {
  let ast = parser.ast.ast;
  for property in object
    .properties(ast)
    .iter()
    .map(|slot| ast.get_node_in_sub_range(slot))
  {
    let ObjectPropertyKindData::ObjectProperty(property) = ast.object_property_kind_data(property)
    else {
      return false;
    };
    if !is_pure_property_key(
      parser,
      analyze_side_effects_free,
      property.key(ast),
      comments,
      callees.as_deref_mut(),
    ) || !is_pure_expression(
      parser,
      analyze_side_effects_free,
      property.value(ast),
      comments,
      callees.as_deref_mut(),
    ) {
      return false;
    }
  }
  true
}

fn object_member_access_is_safe(parser: &mut JavascriptParser, object: Expr) -> bool {
  let ast = parser.ast.ast;
  let ExprData::ObjectExpression(object) = ast.expr_data(object) else {
    return true;
  };
  object.properties(ast).iter().all(|slot| {
    let property = ast.get_node_in_sub_range(slot);
    let ObjectPropertyKindData::ObjectProperty(property) = ast.object_property_kind_data(property)
    else {
      return false;
    };
    if property.computed(ast) || property.method(ast) || property.kind(ast) != PropertyKind::Init {
      return false;
    }
    property_key_name(ast, property.key(ast)).as_deref() != Some("__proto__")
  })
}

fn class_member_access_is_safe(parser: &JavascriptParser, class: Class) -> bool {
  let ast = parser.ast.ast;
  class.body(ast).body(ast).iter().all(|slot| {
    let element = ast.get_node_in_sub_range(slot);
    match ast.class_element_data(element) {
      ClassElementData::MethodDefinition(method) => {
        !(method.r#static(ast)
          && matches!(
            method.kind(ast),
            MethodDefinitionKind::Get | MethodDefinitionKind::Set
          ))
      }
      ClassElementData::TsMethodDefinition(method) => {
        !(method.r#static(ast)
          && matches!(
            method.kind(ast),
            MethodDefinitionKind::Get | MethodDefinitionKind::Set
          ))
      }
      _ => true,
    }
  })
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
    ExprData::IdentifierReference(_)
    | ExprData::ThisExpression(_)
    | ExprData::StringLiteral(_)
    | ExprData::NumericLiteral(_)
    | ExprData::BigIntLiteral(_)
    | ExprData::BooleanLiteral(_)
    | ExprData::NullLiteral(_)
    | ExprData::RegExpLiteral(_)
    | ExprData::PrivateIdentifier(_)
    | ExprData::Function(_)
    | ExprData::ArrowFunctionExpression(_) => true,
    ExprData::ParenthesizedExpression(parenthesized) => is_pure_expression(
      parser,
      analyze_side_effects_free,
      parenthesized.expression(ast),
      comments,
      callees,
    ),
    ExprData::ArrayExpression(array) => {
      for element in array
        .elements(ast)
        .iter()
        .filter_map(|slot| ast.get_node_in_sub_range(slot))
      {
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
    ExprData::ObjectExpression(object) => {
      is_pure_object_expression(parser, analyze_side_effects_free, object, comments, callees)
    }
    ExprData::UnaryExpression(unary) => {
      unary.operator(ast) != UnaryOperator::Delete
        && is_pure_expression(
          parser,
          analyze_side_effects_free,
          unary.argument(ast),
          comments,
          callees,
        )
    }
    ExprData::BinaryExpression(binary) => {
      is_pure_expression(
        parser,
        analyze_side_effects_free,
        binary.left(ast),
        comments,
        callees.as_deref_mut(),
      ) && is_pure_expression(
        parser,
        analyze_side_effects_free,
        binary.right(ast),
        comments,
        callees,
      )
    }
    ExprData::LogicalExpression(logical) => {
      is_pure_expression(
        parser,
        analyze_side_effects_free,
        logical.left(ast),
        comments,
        callees.as_deref_mut(),
      ) && is_pure_expression(
        parser,
        analyze_side_effects_free,
        logical.right(ast),
        comments,
        callees,
      )
    }
    ExprData::ConditionalExpression(conditional) => {
      is_pure_expression(
        parser,
        analyze_side_effects_free,
        conditional.test(ast),
        comments,
        callees.as_deref_mut(),
      ) && is_pure_expression(
        parser,
        analyze_side_effects_free,
        conditional.consequent(ast),
        comments,
        callees.as_deref_mut(),
      ) && is_pure_expression(
        parser,
        analyze_side_effects_free,
        conditional.alternate(ast),
        comments,
        callees,
      )
    }
    ExprData::SequenceExpression(sequence) => sequence
      .expressions(ast)
      .iter()
      .map(|slot| ast.get_node_in_sub_range(slot))
      .all(|expression| {
        is_pure_expression(
          parser,
          analyze_side_effects_free,
          expression,
          comments,
          callees.as_deref_mut(),
        )
      }),
    ExprData::CallExpression(call) => is_pure_call_expression(
      parser,
      analyze_side_effects_free,
      expression,
      call,
      comments,
      callees,
    ),
    ExprData::NewExpression(new_expression) => {
      let pure_annotation = has_pure_comment(comments, expression.span(ast).start);
      (pure_annotation || is_pure_new_callee(parser, new_expression.callee(ast)))
        && arguments_are_pure(
          parser,
          analyze_side_effects_free,
          new_expression.arguments(ast),
          comments,
          None,
        )
    }
    ExprData::Class(class) => is_pure_class_definition_expression(parser, class),
    ExprData::MemberExpression(member) => {
      let object = member.object(ast);
      let object_kind_is_safe = matches!(
        ast.expr_data(object),
        ExprData::ObjectExpression(_)
          | ExprData::Function(_)
          | ExprData::ArrowFunctionExpression(_)
          | ExprData::Class(_)
      );
      if object_kind_is_safe
        && is_pure_expression(
          parser,
          analyze_side_effects_free,
          object,
          comments,
          callees.as_deref_mut(),
        )
        && object_member_access_is_safe(parser, object)
        && match ast.expr_data(object) {
          ExprData::Class(class) => class_member_access_is_safe(parser, class),
          _ => true,
        }
        && is_pure_property_key(
          parser,
          analyze_side_effects_free,
          member.property(ast),
          comments,
          callees,
        )
      {
        true
      } else {
        evaluated_expression_is_pure(parser, expression)
      }
    }
    ExprData::ChainExpression(chain) => {
      let inner = chain.expression(ast);
      match ast.expr_data(inner) {
        ExprData::CallExpression(call) => is_pure_call_expression(
          parser,
          analyze_side_effects_free,
          inner,
          call,
          comments,
          callees,
        ),
        _ => evaluated_expression_is_pure(parser, expression),
      }
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
    _ => evaluated_expression_is_pure(parser, expression),
  }
}

pub fn is_pure_pat(parser: &mut JavascriptParser, pattern: BindingPattern) -> bool {
  let ast = parser.ast.ast;
  match ast.binding_pattern_data(pattern) {
    BindingPatternData::BindingIdentifier(_) | BindingPatternData::BindingRestElement(_) => true,
    BindingPatternData::ArrayPattern(array) => array
      .elements(ast)
      .iter()
      .filter_map(|slot| ast.get_node_in_sub_range(slot))
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
  for item in parameters
    .items(ast)
    .iter()
    .map(|slot| ast.get_node_in_sub_range(slot))
  {
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

  for member in class
    .body(ast)
    .body(ast)
    .iter()
    .map(|slot| ast.get_node_in_sub_range(slot))
  {
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

fn is_pure_class_definition_expression(parser: &mut JavascriptParser, class: Class) -> bool {
  let ast = parser.ast.ast;
  let comments = parser.ast.comments;
  if let Some(super_class) = class.super_class(ast)
    && !is_pure_expression(parser, false, super_class, comments, None)
  {
    return false;
  }
  for member in class
    .body(ast)
    .body(ast)
    .iter()
    .map(|slot| ast.get_node_in_sub_range(slot))
  {
    match ast.class_element_data(member) {
      ClassElementData::MethodDefinition(method) => {
        if !is_pure_property_key(parser, false, method.key(ast), comments, None) {
          return false;
        }
      }
      ClassElementData::TsMethodDefinition(method) => {
        if !is_pure_property_key(parser, false, method.key(ast), comments, None) {
          return false;
        }
      }
      ClassElementData::PropertyDefinition(property) => {
        if !is_pure_property_key(parser, false, property.key(ast), comments, None)
          || property
            .value(ast)
            .is_some_and(|value| !is_pure_expression(parser, false, value, comments, None))
        {
          return false;
        }
      }
      ClassElementData::StaticBlock(block) if !block.body(ast).is_empty() => return false,
      ClassElementData::StaticBlock(_) | ClassElementData::TsIndexSignature(_) => {}
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
  variable
    .declarators(ast)
    .iter()
    .map(|slot| ast.get_node_in_sub_range(slot))
    .all(|declarator| {
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
  for declarator in variable
    .declarators(ast)
    .iter()
    .map(|slot| ast.get_node_in_sub_range(slot))
  {
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

fn is_side_effects_free_stmt(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  statement: Stmt,
) -> bool {
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
  if !parameters_are_simple_identifiers(ast, function) {
    return false;
  }
  function
    .body(ast)
    .body(ast)
    .iter()
    .map(|slot| ast.get_node_in_sub_range(slot))
    .all(|statement| is_side_effects_free_stmt(parser, analyze_side_effects_free, statement))
}

fn is_side_effects_free_arrow_body(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  arrow: ArrowFunctionExpression,
) -> bool {
  let ast = parser.ast.ast;
  let parameters = arrow.params(ast);
  if parameters.rest(ast).is_some()
    || !parameters.items(ast).iter().all(|slot| {
      let item = ast.get_node_in_sub_range(slot);
      let FormalParameterItemData::FormalParameter(parameter) =
        ast.formal_parameter_item_data(item)
      else {
        return false;
      };
      let FormalParameterPatternData::BindingPattern(pattern) =
        ast.formal_parameter_pattern_data(parameter.pattern(ast))
      else {
        return false;
      };
      matches!(
        ast.binding_pattern_data(pattern),
        BindingPatternData::BindingIdentifier(_)
      )
    })
  {
    return false;
  }
  match ast.arrow_function_body_data(arrow.body(ast)) {
    ArrowFunctionBodyData::FunctionBody(body) => body
      .body(ast)
      .iter()
      .map(|slot| ast.get_node_in_sub_range(slot))
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
              ForStatementInitData::VariableDeclaration(variable) => is_pure_var_decl(
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
      Statement::Expr(statement) => is_pure_expression(
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
      Statement::Var(statement) => is_pure_var_decl(
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
