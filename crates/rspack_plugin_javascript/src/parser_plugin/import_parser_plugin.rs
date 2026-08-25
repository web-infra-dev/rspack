use rspack_core::{
  AsyncDependenciesBlock, BoxDependency, ChunkGroupOptions, ContextDependency,
  ContextNameSpaceObject, ContextOptions, DependencyCategory, DependencyRange, DependencyType,
  DynamicImportFetchPriority, DynamicImportMode, GroupOptions, ImportAttributes,
  ReferencedSpecifier, get_context,
};
use rspack_error::{Error, Severity};
use rspack_util::{SpanExt, swc::get_swc_next_comments};
use rustc_hash::FxHashMap;
use swc_next_ecma_ast::{
  ArrowFunctionBodyData, BindingPattern, BindingPatternData, CallExpression, Expr, ExprData,
  FormalParameters, GetSpan, ImportExpression, ObjectPattern, Span, VariableDeclarator,
};

use super::{JavascriptParserPlugin, import_phase::get_import_phase};
use crate::{
  Atom,
  dependency::{
    ImportContextDependency, ImportDependency, ImportEagerDependency, ImportWeakDependency,
  },
  magic_comment::try_extract_magic_comment,
  utils::object_properties::{get_attributes, get_value_by_obj_prop},
  visitors::{
    ContextModuleScanResult, DestructuringAssignmentProperties, HookMemberExpression, Identifier,
    JavascriptParser, PatRef, TagInfoData, TopLevelScope, VariableDeclaration,
    VariableDeclarationKind, context_reg_exp, create_context_dependency, create_traceable_error,
    get_non_optional_part, parse_order_string,
  },
};

const DYNAMIC_IMPORT_TAG: &str = "dynamic import";

fn tag_dynamic_import_referenced(
  parser: &mut JavascriptParser,
  import_call: ImportExpression,
  variable_name: Atom,
) {
  let import_span = import_call.span(parser.ast.ast);
  parser.dynamic_import_references.add_import(import_span);
  parser
    .dynamic_import_references
    .get_import_mut_expect(&import_span)
    .variable_name = Some(variable_name.clone());
  parser.tag_variable(
    variable_name,
    DYNAMIC_IMPORT_TAG,
    Some(ImportTagData { import_span }),
  );
}

fn collect_destructuring_references(
  properties: &DestructuringAssignmentProperties,
) -> Vec<Vec<Atom>> {
  let mut references = Vec::new();
  properties.traverse_on_leaf(&mut |stack| {
    references.push(stack.iter().map(|property| property.id.clone()).collect());
  });
  references
}

fn add_destructuring_import_references(
  parser: &mut JavascriptParser,
  import_call: ImportExpression,
  pattern: ObjectPattern,
) {
  let Some(properties) =
    parser.collect_destructuring_assignment_properties_from_object_pattern(pattern)
  else {
    return;
  };
  let references = collect_destructuring_references(&properties);
  let import_span = import_call.span(parser.ast.ast);
  parser.dynamic_import_references.add_import(import_span);
  let import_references = parser
    .dynamic_import_references
    .get_import_mut_expect(&import_span);
  for reference in references {
    import_references.add_reference(reference);
  }
}

fn is_unbound_promise_all(parser: &mut JavascriptParser, call: CallExpression) -> bool {
  let ast = parser.ast.ast;
  let Some(member) = call.callee(ast).as_member_expression(ast) else {
    return false;
  };
  member
    .object(ast)
    .as_identifier_reference(ast)
    .is_some_and(|ident| ast.get_utf8(ident.name(ast)) == "Promise")
    && member
      .property(ast)
      .as_identifier_name(ast)
      .is_some_and(|ident| ast.get_utf8(ident.name(ast)) == "all")
    && parser.get_variable_info("Promise").is_none()
}

fn track_dynamic_import_pattern(
  parser: &mut JavascriptParser,
  import_call: ImportExpression,
  pattern: BindingPattern,
) {
  let ast = parser.ast.ast;
  match ast.binding_pattern_data(pattern) {
    BindingPatternData::BindingIdentifier(binding) => {
      let name = Atom::from(ast.get_utf8(binding.name(ast)));
      parser.define_variable(name.clone());
      tag_dynamic_import_referenced(parser, import_call, name);
    }
    BindingPatternData::ObjectPattern(pattern) => {
      add_destructuring_import_references(parser, import_call, pattern);
    }
    BindingPatternData::AssignmentPattern(pattern) => {
      track_dynamic_import_pattern(parser, import_call, pattern.left(ast));
    }
    _ => {}
  }
}

fn track_dynamic_imports_in_promise_all(
  parser: &mut JavascriptParser,
  declarator: VariableDeclarator,
) {
  let ast = parser.ast.ast;
  let Some(pattern) = declarator.id(ast).as_array_pattern(ast) else {
    return;
  };
  let Some(init) = declarator.init(ast) else {
    return;
  };
  let Some(await_expr) = init.as_await_expression(ast) else {
    return;
  };
  let Some(promise_all) = await_expr.argument(ast).as_call_expression(ast) else {
    return;
  };
  if !is_unbound_promise_all(parser, promise_all) {
    return;
  }
  let arguments = promise_all.arguments(ast);
  if arguments.len() != 1 {
    return;
  }
  let Some(argument) = arguments.get_node(ast, 0) else {
    return;
  };
  let Some(imports) = argument
    .as_expr(ast)
    .and_then(|expr| expr.as_array_expression(ast))
  else {
    return;
  };
  if imports.elements(ast).iter().any(|slot| {
    ast
      .get_node_in_sub_range(slot)
      .is_some_and(|argument| argument.is_spread_element(ast))
  }) {
    return;
  }

  for (pattern_slot, import_slot) in pattern
    .elements(ast)
    .iter()
    .zip(imports.elements(ast).iter())
  {
    let (Some(pattern), Some(import)) = (
      ast.get_node_in_sub_range(pattern_slot),
      ast.get_node_in_sub_range(import_slot),
    ) else {
      continue;
    };
    let Some(import_call) = import
      .as_expr(ast)
      .and_then(|expr| expr.as_import_expression(ast))
    else {
      continue;
    };
    track_dynamic_import_pattern(parser, import_call, pattern);
  }
}

#[derive(Debug, Default)]
pub struct ImportsReferencesState {
  inner: FxHashMap<Span, ImportReferences>,
}

impl ImportsReferencesState {
  pub fn add_import(&mut self, import: Span) {
    self.inner.entry(import).or_default();
  }

  fn get_import(&self, import: &Span) -> Option<&ImportReferences> {
    self.inner.get(import)
  }

  fn get_import_mut(&mut self, import: &Span) -> Option<&mut ImportReferences> {
    self.inner.get_mut(import)
  }

  fn get_import_mut_expect(&mut self, import: &Span) -> &mut ImportReferences {
    self.get_import_mut(import).expect("should get import")
  }

  fn take_all_import_references(
    &mut self,
  ) -> impl Iterator<
    Item = (
      ImportDependencyLocator,
      Option<Atom>,
      Vec<ReferencedSpecifier>,
    ),
  > + use<> {
    let inner = std::mem::take(&mut self.inner);
    inner.into_values().filter_map(|value| {
      value
        .dep_locator
        .map(|locator| (locator, value.variable_name, value.references))
    })
  }
}

#[derive(Debug, Default)]
struct ImportReferences {
  dep_locator: Option<ImportDependencyLocator>,
  variable_name: Option<Atom>,
  references: Vec<ReferencedSpecifier>,
}

impl ImportReferences {
  pub fn add_reference(&mut self, reference: Vec<Atom>) {
    self.references.push(ReferencedSpecifier::new(reference));
  }

  pub fn add_call_reference(&mut self, reference: Vec<Atom>, namespace_object_as_context: bool) {
    self.references.push(ReferencedSpecifier::new_call(
      reference,
      namespace_object_as_context,
    ));
  }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct ImportDependencyLocator {
  block_idx: Option<usize>,
  dep_idx: usize,
  dep_type: DependencyType,
}

#[derive(Debug, Clone)]
struct ImportTagData {
  import_span: Span,
}

pub struct ImportParserPlugin;

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for ImportParserPlugin {
  fn can_collect_destructuring_assignment_properties(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: Expr,
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    if expr.as_import_expression(ast).is_some() {
      return Some(true);
    }
    if let Some(ident) = expr.as_identifier_reference(ast)
      && let Some(name_info) = parser.get_name_info_from_variable(ast.get_utf8(ident.name(ast)))
      && let Some(info) = name_info.info
      && let Some(name) = info.name.clone()
      && parser
        .get_tag_data::<ImportTagData>(&name, DYNAMIC_IMPORT_TAG)
        .is_some()
    {
      return Some(true);
    }
    None
  }

  fn pre_declarator(
    &self,
    parser: &mut JavascriptParser<'p>,
    declarator: VariableDeclarator,
    declaration: VariableDeclaration,
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    if declaration.kind(ast) != VariableDeclarationKind::Var {
      if let Some(init) = declarator.init(ast)
        && let Some(expr) = init.as_await_expression(ast)
        && let Some(import) = expr.argument(ast).as_import_expression(ast)
        && let Some(binding) = declarator.id(ast).as_binding_identifier(ast)
      {
        let name = Atom::from(ast.get_utf8(binding.name(ast)));
        parser.define_variable(name.clone());
        tag_dynamic_import_referenced(parser, import, name);
      }
      track_dynamic_imports_in_promise_all(parser, declarator);
    }
    None
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    ident: &Identifier,
    for_name: &str,
  ) -> Option<bool> {
    if for_name != DYNAMIC_IMPORT_TAG {
      return None;
    }
    let tag_info = parser
      .definitions_db
      .expect_get_tag_info(parser.current_tag_info?);
    let data = ImportTagData::downcast(tag_info.data.clone()?);
    if let Some(keys) = parser
      .destructuring_assignment_properties
      .get(&ident.span())
    {
      for ids in collect_destructuring_references(keys) {
        parser
          .dynamic_import_references
          .get_import_mut_expect(&data.import_span)
          .add_reference(ids);
      }
    } else {
      parser
        .dynamic_import_references
        .get_import_mut_expect(&data.import_span)
        .add_reference(vec![]);
    }
    Some(true)
  }

  fn member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    _expr: HookMemberExpression,
    for_name: &str,
    members: &[Atom],
    members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    if for_name != DYNAMIC_IMPORT_TAG {
      return None;
    }
    let tag_info = parser
      .definitions_db
      .expect_get_tag_info(parser.current_tag_info?);
    let data = ImportTagData::downcast(tag_info.data.clone()?);
    let ids = get_non_optional_part(members, members_optionals);
    parser
      .dynamic_import_references
      .get_import_mut_expect(&data.import_span)
      .add_reference(ids.to_vec());
    Some(true)
  }

  fn call_member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: CallExpression,
    for_name: &str,
    members: &[Atom],
    members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    if for_name != DYNAMIC_IMPORT_TAG {
      return None;
    }
    let tag_info = parser
      .definitions_db
      .expect_get_tag_info(parser.current_tag_info?);
    let data = ImportTagData::downcast(tag_info.data.clone()?);
    let ids = get_non_optional_part(members, members_optionals);
    let direct_import = members.is_empty();
    parser
      .dynamic_import_references
      .get_import_mut_expect(&data.import_span)
      .add_call_reference(
        ids.to_vec(),
        parser
          .javascript_options
          .strict_this_context_on_imports
          .unwrap_or(false)
          && !direct_import,
      );
    let ast = parser.ast.ast;
    parser.walk_arguments(
      expr
        .arguments(ast)
        .iter()
        .map(|id| ast.get_node_in_sub_range(id)),
    );
    Some(true)
  }

  fn import_call(
    &self,
    parser: &mut JavascriptParser<'p>,
    node: ImportExpression,
    import_then: Option<CallExpression>,
    referenced_in_members: Option<(&[Atom], bool)>,
  ) -> Option<bool> {
    // Skip unreachable dynamic imports that are placed after a terminating
    // statement like `return` / `throw` (non top-level). This relies on
    // `parser.terminated` which mirrors webpack's `scope.terminated` logic.
    if parser.terminated.is_some() && !parser.is_top_level_scope() {
      return Some(true);
    }

    let ast = parser.ast.ast;
    let dyn_imported = node.source(ast);
    let import_call_span = node.span(ast);
    let dyn_imported_span = dyn_imported.span(ast);
    let dynamic_import_mode = parser.javascript_options.dynamic_import_mode;
    let dynamic_import_preload = parser
      .javascript_options
      .dynamic_import_preload
      .expect("should have dynamic_import_preload")
      .get_order();
    let dynamic_import_prefetch = parser
      .javascript_options
      .dynamic_import_prefetch
      .expect("should have dynamic_import_prefetch")
      .get_order();
    let dynamic_import_fetch_priority = parser.javascript_options.dynamic_import_fetch_priority;

    let magic_comment_options =
      try_extract_magic_comment(parser, import_call_span, dyn_imported_span);
    if magic_comment_options.get_ignore().unwrap_or_default() {
      return None;
    }

    let mode = magic_comment_options.get_mode().map_or(
      dynamic_import_mode.expect("should have dynamic_import_mode"),
      |x| DynamicImportMode::from(x.as_str()),
    );
    let chunk_name = magic_comment_options.get_chunk_name().map(|x| x.to_owned());
    let chunk_prefetch = magic_comment_options
      .get_prefetch()
      .and_then(|x| parse_order_string(x.as_ref()))
      .or(dynamic_import_prefetch);
    let chunk_preload = magic_comment_options
      .get_preload()
      .and_then(|x| parse_order_string(x.as_ref()))
      .or(dynamic_import_preload);
    let fetch_priority = magic_comment_options
      .get_fetch_priority()
      .map(|x| DynamicImportFetchPriority::from(x.as_str()))
      .or(dynamic_import_fetch_priority);
    let include = magic_comment_options.get_include();
    let exclude = magic_comment_options.get_exclude();
    let mut exports = magic_comment_options.get_exports().map(|x| {
      x.iter()
        .map(|name| ReferencedSpecifier::new(vec![Atom::from(name.as_str())]))
        .collect::<Vec<_>>()
    });
    let has_exports_magic_comment = exports.is_some();

    let referenced_in_destructuring = parser
      .destructuring_assignment_properties
      .get(&import_call_span);
    let referenced_in_variable = parser
      .dynamic_import_references
      .get_import(&import_call_span);
    let referenced_fulfilled_ns_obj = import_then
      .and_then(|import_then| get_fulfilled_callback_namespace_obj(parser.ast.ast, import_then));
    if let Some(keys) = referenced_in_destructuring {
      exports = Some(
        collect_destructuring_references(keys)
          .into_iter()
          .map(ReferencedSpecifier::new)
          .collect(),
      );
    }
    if let Some((referenced_in_members, is_call)) = referenced_in_members {
      let referenced = if is_call {
        ReferencedSpecifier::new_call(
          referenced_in_members.to_vec(),
          parser
            .javascript_options
            .strict_this_context_on_imports
            .unwrap_or(false)
            && !referenced_in_members.is_empty(),
        )
      } else {
        ReferencedSpecifier::new(referenced_in_members.to_vec())
      };
      exports = Some(vec![referenced]);
    }

    let is_statical = referenced_in_destructuring.is_some()
      || referenced_in_variable.is_some()
      || referenced_fulfilled_ns_obj.is_some()
      || referenced_in_members.is_some();
    if is_statical && has_exports_magic_comment {
      let mut error: Error = create_traceable_error(
        "Useless magic comments".into(),
        "You don't need `webpackExports` if the usage of dynamic import is statically analyse-able. You can safely remove the `webpackExports` magic comment.".into(),
        parser.source.to_string(),
        import_call_span.into(),
      );
      error.severity = Severity::Warning;
      error.hide_stack = Some(true);
      parser.add_warning(error.into());
    }

    let phase = get_import_phase(parser, node.phase(parser.ast.ast));
    if phase.is_defer() && !parser.compiler_options.experiments.defer_import {
      parser.add_error(rspack_error::error!("deferImport is still an experimental feature. To continue using it, please enable 'experiments.deferImport'.").into());
    }
    if phase.is_source() && !parser.compiler_options.experiments.source_import {
      parser.add_error(rspack_error::error!("sourceImport is still an experimental feature. To continue using it, please enable 'experiments.sourceImport'.").into());
    }

    let attributes = get_attributes_from_import_expr(parser.ast.ast, node);
    let param = parser.evaluate_expression(dyn_imported);

    let dep_locator = if param.is_string() {
      if matches!(mode, DynamicImportMode::Eager) {
        let mut dep = ImportEagerDependency::new(
          param.string().as_str().into(),
          import_call_span.into(),
          attributes,
          phase,
        );
        if let Some(exports) = exports {
          dep.set_referenced_specifiers(exports, !is_statical && has_exports_magic_comment);
        }
        let dep_idx = parser.next_dependency_idx();
        parser.add_dependency(BoxDependency::new(dep));
        ImportDependencyLocator {
          block_idx: None,
          dep_idx,
          dep_type: DependencyType::DynamicImportEager,
        }
      } else if matches!(mode, DynamicImportMode::Weak) {
        let mut dep = ImportWeakDependency::new(
          param.string().as_str().into(),
          import_call_span.into(),
          attributes,
          phase,
          parser.in_try,
        );
        if let Some(exports) = exports {
          dep.set_referenced_specifiers(exports, !is_statical && has_exports_magic_comment);
        }
        let dep_idx = parser.next_dependency_idx();
        parser.add_dependency(BoxDependency::new(dep));
        ImportDependencyLocator {
          block_idx: None,
          dep_idx,
          dep_type: DependencyType::DynamicImportWeak,
        }
      } else {
        let mut dep = ImportDependency::new(
          param.string().as_str().into(),
          import_call_span.into(),
          attributes,
          phase,
          parser.in_try,
          {
            get_swc_next_comments(
              parser.ast.comments,
              dyn_imported_span.start,
              dyn_imported_span.end,
            )
          },
        );
        if let Some(export) = exports {
          dep.set_referenced_specifiers(export, !is_statical && has_exports_magic_comment);
        }
        let range = DependencyRange::from(import_call_span);
        let loc = parser.to_dependency_location(range);
        let mut block = AsyncDependenciesBlock::new(
          *parser.module_identifier,
          loc,
          None,
          vec![BoxDependency::new(dep)],
          Some(param.string().clone()),
        );
        block.set_group_options(GroupOptions::ChunkGroup(ChunkGroupOptions::new(
          chunk_name,
          chunk_preload,
          chunk_prefetch,
          fetch_priority,
        )));
        let block_idx = parser.next_block_idx();
        parser.add_block(Box::new(block));
        ImportDependencyLocator {
          block_idx: Some(block_idx),
          dep_idx: 0,
          dep_type: DependencyType::DynamicImport,
        }
      }
    } else {
      if matches!(parser.javascript_options.import_dynamic, Some(false)) {
        return None;
      }

      let result = create_context_dependency(&param, parser);
      let request = result.request();
      let ContextModuleScanResult {
        reg,
        replaces,
        critical,
        ..
      } = result;

      let reg_exp = context_reg_exp(&reg, "", Some(dyn_imported_span.into()), parser);
      let mut dep = ImportContextDependency::new(
        ContextOptions {
          mode: mode.into(),
          recursive: true,
          pattern: reg_exp.into(),
          include,
          exclude,
          category: DependencyCategory::Esm,
          request,
          context: get_context(parser.resource_data).to_string(),
          compiler_context: parser.compiler_options.context.clone(),
          namespace_object: if parser.build_meta.strict_esm_module() {
            ContextNameSpaceObject::Strict
          } else {
            ContextNameSpaceObject::Bool(true)
          },
          group_options: Some(GroupOptions::ChunkGroup(ChunkGroupOptions::new(
            chunk_name,
            chunk_preload,
            chunk_prefetch,
            fetch_priority,
          ))),
          replaces,
          start: import_call_span.real_lo(),
          end: import_call_span.real_hi(),
          referenced_specifiers: None,
          glob_import: None,
          glob_exhaustive: false,
          glob_case_sensitive: true,
          attributes,
          phase: Some(phase),
        },
        import_call_span.into(),
        dyn_imported_span.into(),
        parser.in_try,
      );
      if let Some(export) = exports {
        dep.set_referenced_specifiers(export, !is_statical && has_exports_magic_comment);
      }
      dep.set_critical(critical);
      let dep_idx = parser.next_dependency_idx();
      parser.add_dependency(BoxDependency::new(dep));
      ImportDependencyLocator {
        block_idx: None,
        dep_idx,
        dep_type: DependencyType::ImportContext,
      }
    };

    if let Some(import_then) = import_then {
      if let Some(ns_obj) = referenced_fulfilled_ns_obj {
        let ast = parser.ast.ast;
        let arguments = import_then.arguments(ast);
        let fulfilled_callback = arguments
          .get_node(ast, 0)
          .and_then(|argument| argument.as_expr(ast))
          .expect("fulfilled callback should be an expression");
        walk_import_then_fulfilled_callback(parser, node, fulfilled_callback, ns_obj);
        parser.walk_arguments(
          arguments
            .iter()
            .skip(1)
            .map(|id| ast.get_node_in_sub_range(id)),
        );
      } else {
        let ast = parser.ast.ast;
        parser.walk_arguments(
          import_then
            .arguments(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id)),
        );
      }
    }

    if let Some(import_references) = parser
      .dynamic_import_references
      .get_import_mut(&import_call_span)
    {
      import_references.dep_locator = Some(dep_locator);
    }

    Some(true)
  }

  fn finish(&self, parser: &mut JavascriptParser<'p>) -> Option<bool> {
    for (locator, variable_name, mut references) in parser
      .dynamic_import_references
      .take_all_import_references()
    {
      // If the import result is assigned to a variable that is also an ESM
      // named export, importers may access arbitrary properties on it. In that
      // case the entire module must be considered referenced.
      if let Some(variable_name) = variable_name
        && parser.build_info.esm_named_exports.contains(&variable_name)
      {
        references.push(ReferencedSpecifier::new(vec![]));
      }
      let dep = if let Some(block_idx) = locator.block_idx
        && let Some(block) = parser.get_block_mut(block_idx)
      {
        block.get_dependency_mut(locator.dep_idx)
      } else {
        parser.get_dependency_mut(locator.dep_idx)
      };
      let Some(dep) = dep else {
        continue;
      };
      match locator.dep_type {
        DependencyType::DynamicImport => {
          let dep = dep
            .downcast_mut::<ImportDependency>()
            .expect("Failed to downcast to ImportDependency");
          dep.set_referenced_specifiers(references, false);
        }
        DependencyType::DynamicImportEager => {
          let dep = dep
            .downcast_mut::<ImportEagerDependency>()
            .expect("Failed to downcast to ImportEagerDependency");
          dep.set_referenced_specifiers(references, false);
        }
        DependencyType::DynamicImportWeak => {
          let dep = dep
            .downcast_mut::<ImportWeakDependency>()
            .expect("Failed to downcast to ImportWeakDependency");
          dep.set_referenced_specifiers(references, false);
        }
        DependencyType::ImportContext => {
          let dep = dep
            .downcast_mut::<ImportContextDependency>()
            .expect("Failed to downcast to ImportContextDependency");
          dep.set_referenced_specifiers(references, false);
        }
        _ => unreachable!(),
      };
    }
    None
  }
}

fn get_attributes_from_import_expr(
  ast: &swc_next_ecma_ast::Ast<'_>,
  node: ImportExpression,
) -> Option<ImportAttributes> {
  let options = node.options(ast)?.as_object_expression(ast)?;
  get_value_by_obj_prop(ast, options, "with")
    .and_then(|expr| expr.as_object_expression(ast))
    .map(|object| get_attributes(ast, object))
}

fn formal_parameter_patterns<'a>(
  ast: &'a swc_next_ecma_ast::Ast<'_>,
  params: FormalParameters,
) -> impl Iterator<Item = BindingPattern> + 'a {
  let rest = params.rest(ast).map(BindingPattern::BindingRestElement);
  params
    .items(ast)
    .iter()
    .filter_map(move |id| ast.get_node_in_sub_range(id).as_formal_parameter(ast))
    .filter_map(move |parameter| parameter.pattern(ast).as_binding_pattern(ast))
    .chain(rest)
}

fn get_fulfilled_callback_namespace_obj(
  ast: &swc_next_ecma_ast::Ast<'_>,
  import_then: CallExpression,
) -> Option<BindingPattern> {
  let fulfilled_callback = import_then.arguments(ast).get_node(ast, 0)?.as_expr(ast)?;
  let params = match ast.expr_data(fulfilled_callback) {
    ExprData::ArrowFunctionExpression(function) => function.params(ast),
    ExprData::Function(function) => function.params(ast),
    _ => return None,
  };
  let ns_obj = formal_parameter_patterns(ast, params).next()?;
  if ns_obj.as_binding_identifier(ast).is_some() || ns_obj.as_object_pattern(ast).is_some() {
    return Some(ns_obj);
  }
  None
}

fn walk_import_then_fulfilled_callback(
  parser: &mut JavascriptParser,
  import_call: ImportExpression,
  fulfilled_callback: Expr,
  namespace_obj_arg: BindingPattern,
) {
  let ast = parser.ast.ast;
  let (params, function_id, is_function) = match ast.expr_data(fulfilled_callback) {
    ExprData::Function(function) => (function.params(ast), function.id(ast), true),
    ExprData::ArrowFunctionExpression(function) => (function.params(ast), None, false),
    _ => unreachable!(),
  };
  let scope_params = formal_parameter_patterns(ast, params)
    .map(PatRef::Borrowed)
    // Add the function name to the scope for recursive calls.
    .chain(
      function_id
        .map(BindingPattern::BindingIdentifier)
        .map(PatRef::Owned),
    );

  let was_top_level_scope = parser.top_level_scope;
  parser.top_level_scope = if !matches!(was_top_level_scope, TopLevelScope::False) && !is_function {
    TopLevelScope::ArrowFunction
  } else {
    TopLevelScope::False
  };

  parser.in_function_scope(is_function, scope_params, |parser| {
    let ast = parser.ast.ast;
    if let Some(ns_obj) = namespace_obj_arg.as_binding_identifier(ast) {
      tag_dynamic_import_referenced(
        parser,
        import_call,
        Atom::from(ast.get_utf8(ns_obj.name(ast))),
      );
    } else if let Some(ns_obj) = namespace_obj_arg.as_object_pattern(ast) {
      if let Some(keys) =
        parser.collect_destructuring_assignment_properties_from_object_pattern(ns_obj)
      {
        let import_span = import_call.span(parser.ast.ast);
        parser.dynamic_import_references.add_import(import_span);
        let import_references = parser
          .dynamic_import_references
          .get_import_mut_expect(&import_span);
        let mut refs = Vec::new();
        keys.traverse_on_leaf(&mut |stack| {
          refs.push(stack.iter().map(|p| p.id.clone()).collect::<Vec<Atom>>());
        });
        for ids in refs {
          import_references.add_reference(ids);
        }
      }
    } else {
      unreachable!()
    }
    for pattern in formal_parameter_patterns(parser.ast.ast, params) {
      parser.walk_pattern(pattern);
    }
    match parser.ast.ast.expr_data(fulfilled_callback) {
      ExprData::Function(function) => {
        parser.walk_function_body(function.body(parser.ast.ast));
      }
      ExprData::ArrowFunctionExpression(function) => {
        match parser
          .ast
          .ast
          .arrow_function_body_data(function.body(parser.ast.ast))
        {
          ArrowFunctionBodyData::FunctionBody(body) => parser.walk_function_body(body),
          ArrowFunctionBodyData::Expr(expression) => parser.walk_expression(expression),
        }
      }
      _ => unreachable!(),
    }
  });
  parser.top_level_scope = was_top_level_scope;
}
