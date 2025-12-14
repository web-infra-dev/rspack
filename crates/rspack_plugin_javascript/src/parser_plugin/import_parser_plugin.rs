use std::borrow::Cow;

use rspack_core::{
  AsyncDependenciesBlock, ChunkGroupOptions, ContextDependency, ContextNameSpaceObject,
  ContextOptions, DependencyCategory, DependencyRange, DependencyType, DynamicImportFetchPriority,
  DynamicImportMode, GroupOptions, ImportAttributes, SharedSourceMap,
};
use rspack_error::{Error, Severity};
use rspack_util::{SpanExt, swc::get_swc_comments};
use rustc_hash::FxHashMap;
use swc_core::{
  common::{Span, Spanned},
  ecma::{
    ast::{BlockStmtOrExpr, CallExpr, Expr, Ident, MemberExpr, Pat, VarDeclarator},
    atoms::Atom,
  },
};

use super::JavascriptParserPlugin;
use crate::{
  dependency::{ImportContextDependency, ImportDependency, ImportEagerDependency},
  magic_comment::try_extract_magic_comment,
  utils::object_properties::{get_attributes, get_value_by_obj_prop},
  visitors::{
    AllowedMemberTypes, ContextModuleScanResult, ExportedVariableInfo, JavascriptParser,
    MemberExpressionInfo, Statement, TagInfoData, TopLevelScope, VariableDeclaration,
    context_reg_exp, create_context_dependency, create_traceable_error, get_non_optional_part,
    parse_order_string,
  },
};

const DYNAMIC_IMPORT_TAG: &str = "dynamic import";

fn tag_dynamic_import_referenced(
  parser: &mut JavascriptParser,
  import_call: &CallExpr,
  variable_name: Atom,
) {
  let import_span = import_call.span();
  parser.dynamic_import_references.add_import(import_span);
  parser.tag_variable(
    variable_name,
    DYNAMIC_IMPORT_TAG,
    Some(ImportTagData { import_span }),
  );
}

#[derive(Debug, Default)]
pub struct ImportsReferencesState {
  inner: FxHashMap<Span, ImportReferences>,
}

impl ImportsReferencesState {
  pub fn add_import(&mut self, import: Span) {
    self.inner.insert(import, ImportReferences::default());
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
  ) -> impl Iterator<Item = (ImportDependencyLocator, Vec<Vec<Atom>>)> + use<> {
    let inner = std::mem::take(&mut self.inner);
    inner
      .into_values()
      .filter_map(|value| value.dep_locator.map(|locator| (locator, value.references)))
  }
}

#[derive(Debug, Default)]
struct ImportReferences {
  dep_locator: Option<ImportDependencyLocator>,
  references: Vec<Vec<Atom>>,
}

impl ImportReferences {
  pub fn add_reference(&mut self, reference: Vec<Atom>) {
    self.references.push(reference);
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

impl JavascriptParserPlugin for ImportParserPlugin {
  fn can_collect_destructuring_assignment_properties(
    &self,
    parser: &mut JavascriptParser,
    expr: &Expr,
  ) -> Option<bool> {
    if let Some(call) = expr.as_call()
      && call.callee.is_import()
    {
      return Some(true);
    }
    if let MemberExpressionInfo::Expression(info) =
      parser.get_member_expression_info_from_expr(expr, AllowedMemberTypes::Expression)?
      && let ExportedVariableInfo::VariableInfo(id) = &info.root_info
      && let Some(name) = &parser.definitions_db.expect_get_variable(*id).name
      && parser
        .get_tag_data(&name.clone(), DYNAMIC_IMPORT_TAG)
        .is_some()
    {
      return Some(true);
    }
    None
  }

  fn pre_declarator(
    &self,
    parser: &mut JavascriptParser,
    declarator: &VarDeclarator,
    _declaration: VariableDeclaration<'_>,
  ) -> Option<bool> {
    if let Some(init) = &declarator.init
      && let Some(expr) = init.as_await_expr()
      && let Some(call) = expr.arg.as_call()
      && call.callee.is_import()
      && let Some(binding) = declarator.name.as_ident()
    {
      parser.define_variable(binding.id.sym.clone());
      tag_dynamic_import_referenced(parser, call, binding.id.sym.clone());
    }
    None
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &Ident,
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
      let mut refs = Vec::new();
      keys.traverse_on_leaf(&mut |stack| {
        refs.push(stack.iter().map(|p| p.id.clone()).collect());
      });
      for ids in refs {
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
    parser: &mut JavascriptParser,
    _expr: &MemberExpr,
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
    parser: &mut JavascriptParser,
    expr: &CallExpr,
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
    let mut ids = get_non_optional_part(members, members_optionals);
    let direct_import = members.is_empty();
    if !direct_import && ids.len() > 1 {
      // remove last one
      ids = &ids[..ids.len() - 1];
    }
    parser
      .dynamic_import_references
      .get_import_mut_expect(&data.import_span)
      .add_reference(ids.to_vec());
    parser.walk_expr_or_spread(&expr.args);
    Some(true)
  }

  fn import_call(
    &self,
    parser: &mut JavascriptParser,
    node: &CallExpr,
    import_then: Option<&CallExpr>,
  ) -> Option<bool> {
    let dyn_imported = node.args.first()?;
    if dyn_imported.spread.is_some() {
      return None;
    }
    let import_call_span = node.span();
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

    let magic_comment_options = try_extract_magic_comment(parser, node.span, dyn_imported.span());
    if magic_comment_options.get_ignore().unwrap_or_default() {
      return None;
    }

    let mode = magic_comment_options
      .get_mode()
      .map(|x| DynamicImportMode::from(x.as_str()))
      .unwrap_or(dynamic_import_mode.expect("should have dynamic_import_mode"));
    let chunk_name = magic_comment_options.get_chunk_name().map(|x| x.to_owned());
    let chunk_prefetch = magic_comment_options
      .get_prefetch()
      .and_then(|x| parse_order_string(x.as_str()))
      .or(dynamic_import_prefetch);
    let chunk_preload = magic_comment_options
      .get_preload()
      .and_then(|x| parse_order_string(x.as_str()))
      .or(dynamic_import_preload);
    let fetch_priority = magic_comment_options
      .get_fetch_priority()
      .map(|x| DynamicImportFetchPriority::from(x.as_str()))
      .or(dynamic_import_fetch_priority);
    let include = magic_comment_options.get_include();
    let exclude = magic_comment_options.get_exclude();
    let mut exports = magic_comment_options.get_exports().map(|x| {
      x.iter()
        .map(|name| vec![Atom::from(name.as_str())])
        .collect::<Vec<_>>()
    });
    let has_exports_magic_comment = exports.is_some();

    let referenced_in_destructuring = parser
      .destructuring_assignment_properties
      .get(&import_call_span);
    let referenced_in_member = parser
      .dynamic_import_references
      .get_import(&import_call_span);
    let referenced_fulfilled_ns_obj =
      import_then.and_then(|import_then| get_fulfilled_callback_namespace_obj(import_then));
    if let Some(keys) = referenced_in_destructuring {
      let mut refs = Vec::new();
      keys.traverse_on_leaf(&mut |stack| {
        refs.push(stack.iter().map(|p| p.id.clone()).collect());
      });
      exports = Some(refs);
    }

    let is_statical = referenced_in_destructuring.is_some()
      || referenced_in_member.is_some()
      || referenced_fulfilled_ns_obj.is_some();
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

    let attributes = get_attributes_from_call_expr(node);
    let param = parser.evaluate_expression(dyn_imported.expr.as_ref());

    let dep_locator = if param.is_string() {
      if matches!(mode, DynamicImportMode::Eager) {
        let dep = ImportEagerDependency::new(
          param.string().as_str().into(),
          import_call_span.into(),
          exports,
          attributes,
        );
        let dep_idx = parser.next_dependency_idx();
        parser.add_dependency(Box::new(dep));
        ImportDependencyLocator {
          block_idx: None,
          dep_idx,
          dep_type: DependencyType::DynamicImportEager,
        }
      } else {
        let dep = Box::new(ImportDependency::new(
          param.string().as_str().into(),
          import_call_span.into(),
          exports,
          attributes,
          parser.in_try,
          get_swc_comments(
            parser.comments,
            dyn_imported.span().lo,
            dyn_imported.span().hi,
          ),
        ));
        let source_map: SharedSourceMap = parser.source().clone();
        let mut block = AsyncDependenciesBlock::new(
          *parser.module_identifier,
          Into::<DependencyRange>::into(import_call_span).to_loc(Some(source_map.as_ref())),
          None,
          vec![dep],
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

      let ContextModuleScanResult {
        context,
        reg,
        query,
        fragment,
        replaces,
        critical,
      } = create_context_dependency(&param, parser);

      let reg_exp = context_reg_exp(&reg, "", Some(dyn_imported.span().into()), parser);
      let mut dep = ImportContextDependency::new(
        ContextOptions {
          mode: mode.into(),
          recursive: true,
          reg_exp,
          include,
          exclude,
          category: DependencyCategory::Esm,
          request: format!("{}{}{}", context.clone(), query, fragment),
          context,
          namespace_object: if parser.build_meta.strict_esm_module {
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
          referenced_exports: exports,
          attributes,
        },
        import_call_span.into(),
        dyn_imported.span().into(),
        parser.in_try,
      );
      *dep.critical_mut() = critical;
      let dep_idx = parser.next_dependency_idx();
      parser.add_dependency(Box::new(dep));
      ImportDependencyLocator {
        block_idx: None,
        dep_idx,
        dep_type: DependencyType::ImportContext,
      }
    };

    if let Some(import_then) = import_then {
      if let Some(ns_obj) = referenced_fulfilled_ns_obj {
        walk_import_then_fulfilled_callback(parser, node, &import_then.args[0].expr, ns_obj);
        parser.walk_expr_or_spread(&import_then.args[1..]);
      } else {
        parser.walk_expr_or_spread(&import_then.args);
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

  fn finish(&self, parser: &mut JavascriptParser) -> Option<bool> {
    for (locator, references) in parser
      .dynamic_import_references
      .take_all_import_references()
    {
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
          dep.set_referenced_exports(references);
        }
        DependencyType::DynamicImportEager => {
          let dep = dep
            .downcast_mut::<ImportEagerDependency>()
            .expect("Failed to downcast to ImportEagerDependency");
          dep.set_referenced_exports(references);
        }
        DependencyType::ImportContext => {
          let dep = dep
            .downcast_mut::<ImportContextDependency>()
            .expect("Failed to downcast to ImportContextDependency");
          dep.set_referenced_exports(references);
        }
        _ => unreachable!(),
      };
    }
    None
  }
}

fn get_attributes_from_call_expr(node: &CallExpr) -> Option<ImportAttributes> {
  node
    .args
    .get(1)
    .and_then(|arg| arg.expr.as_object())
    .and_then(|obj| get_value_by_obj_prop(obj, "with"))
    .and_then(|expr| expr.as_object())
    .map(get_attributes)
}

fn get_fulfilled_callback_namespace_obj(import_then: &CallExpr) -> Option<&Pat> {
  let fulfilled_callback = import_then.args.first()?;
  if fulfilled_callback.spread.is_some() {
    return None;
  }
  let fulfilled_callback = &*fulfilled_callback.expr;
  let ns_obj = match fulfilled_callback {
    Expr::Arrow(f) => f.params.first()?,
    Expr::Fn(f) => &f.function.params.first()?.pat,
    _ => return None,
  };
  if ns_obj.is_ident() || ns_obj.is_object() {
    return Some(ns_obj);
  }
  None
}

fn walk_import_then_fulfilled_callback(
  parser: &mut JavascriptParser,
  import_call: &CallExpr,
  fulfilled_callback: &Expr,
  namespace_obj_arg: &Pat,
) {
  let mut scope_params: Vec<Cow<Pat>> = if let Some(fn_expr) = fulfilled_callback.as_fn_expr() {
    fn_expr
      .function
      .params
      .iter()
      .map(|p| Cow::Borrowed(&p.pat))
      .collect()
  } else if let Some(arrow_expr) = fulfilled_callback.as_arrow() {
    arrow_expr.params.iter().map(Cow::Borrowed).collect()
  } else {
    unreachable!()
  };

  // Add function name in scope for recursive calls
  if let Some(expr) = fulfilled_callback.as_fn_expr()
    && let Some(ident) = &expr.ident
  {
    scope_params.push(Cow::Owned(Pat::Ident(ident.clone().into())));
  }

  let was_top_level_scope = parser.top_level_scope;
  parser.top_level_scope =
    if !matches!(was_top_level_scope, TopLevelScope::False) && fulfilled_callback.is_arrow() {
      TopLevelScope::ArrowFunction
    } else {
      TopLevelScope::False
    };

  parser.in_function_scope(
    fulfilled_callback.is_fn_expr(),
    scope_params.into_iter(),
    |parser| {
      if let Some(ns_obj) = namespace_obj_arg.as_ident() {
        tag_dynamic_import_referenced(parser, import_call, ns_obj.id.sym.clone());
      } else if let Some(ns_obj) = namespace_obj_arg.as_object() {
        if let Some(keys) =
          parser.collect_destructuring_assignment_properties_from_object_pattern(ns_obj)
        {
          parser
            .dynamic_import_references
            .add_import(import_call.span());
          let import_references = parser
            .dynamic_import_references
            .get_import_mut_expect(&import_call.span());
          let mut refs = Vec::new();
          keys.traverse_on_leaf(&mut |stack| {
            refs.push(stack.iter().map(|p| p.id.clone()).collect());
          });
          for ids in refs {
            import_references.add_reference(ids);
          }
        }
      } else {
        unreachable!()
      }
      if let Some(expr) = fulfilled_callback.as_fn_expr() {
        for param in &expr.function.params {
          parser.walk_pattern(&param.pat);
        }
        if let Some(stmt) = &expr.function.body {
          parser.detect_mode(&stmt.stmts);
          let prev = parser.prev_statement;
          parser.pre_walk_statement(Statement::Block(stmt));
          parser.prev_statement = prev;
          parser.walk_statement(Statement::Block(stmt));
        }
      } else if let Some(expr) = fulfilled_callback.as_arrow() {
        for pat in &expr.params {
          parser.walk_pattern(pat);
        }
        match &*expr.body {
          BlockStmtOrExpr::BlockStmt(stmt) => {
            parser.detect_mode(&stmt.stmts);
            let prev = parser.prev_statement;
            parser.pre_walk_statement(Statement::Block(stmt));
            parser.prev_statement = prev;
            parser.walk_statement(Statement::Block(stmt));
          }
          BlockStmtOrExpr::Expr(expr) => parser.walk_expression(expr),
        }
      } else {
        unreachable!()
      }
    },
  );
  parser.top_level_scope = was_top_level_scope;
}
