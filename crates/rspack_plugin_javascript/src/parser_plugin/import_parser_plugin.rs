use itertools::Itertools;
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
    ast::{CallExpr, Expr, Ident, MemberExpr, VarDeclarator},
    atoms::Atom,
  },
};

use super::JavascriptParserPlugin;
use crate::{
  dependency::{ImportContextDependency, ImportDependency, ImportEagerDependency},
  utils::object_properties::{get_attributes, get_value_by_obj_prop},
  visitors::{
    ContextModuleScanResult, JavascriptParser, TagInfoData, VariableDeclaration, context_reg_exp,
    create_context_dependency, create_traceable_error, get_non_optional_part, parse_order_string,
  },
  webpack_comment::try_extract_webpack_magic_comment,
};

const DYNAMIC_IMPORT_TAG: &str = "dynamic import";

#[derive(Debug, Default)]
pub struct ImportsReferencesState {
  inner: FxHashMap<Span, ImportReferences>,
}

impl ImportsReferencesState {
  pub fn add_import(&mut self, import: Span) {
    self.inner.insert(import, ImportReferences::default());
  }

  pub fn add_import_reference(&mut self, import: Span, reference: Vec<Atom>) {
    let references = self
      .inner
      .get_mut(&import)
      .expect("should add_import before");
    references.references.push(reference);
  }

  fn take_import_references(
    &mut self,
  ) -> impl Iterator<Item = (ImportDependencyLocator, Vec<Vec<Atom>>)> + use<> {
    let inner = std::mem::take(&mut self.inner);
    inner
      .into_values()
      .filter_map(|value| value.dep_locator.map(|locator| (locator, value.references)))
  }

  fn get_import(&self, import: &Span) -> Option<&ImportReferences> {
    self.inner.get(import)
  }

  fn get_import_mut(&mut self, import: &Span) -> Option<&mut ImportReferences> {
    self.inner.get_mut(import)
  }
}

#[derive(Debug, Default)]
struct ImportReferences {
  dep_locator: Option<ImportDependencyLocator>,
  references: Vec<Vec<Atom>>,
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
  fn collect_destructuring_assignment_properties(
    &self,
    _parser: &mut JavascriptParser,
    expr: &Expr,
  ) -> Option<bool> {
    if let Some(call) = expr.as_call()
      && call.callee.is_import()
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
      let import_span = call.span();
      parser.dynamic_import_references.add_import(import_span);
      parser.tag_variable(
        binding.id.sym.clone(),
        DYNAMIC_IMPORT_TAG,
        Some(ImportTagData { import_span }),
      );
    }
    None
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    _ident: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    if for_name != DYNAMIC_IMPORT_TAG {
      return None;
    }
    let tag_info = parser
      .definitions_db
      .expect_get_tag_info(parser.current_tag_info?);
    let data = ImportTagData::downcast(tag_info.data.clone()?);
    parser
      .dynamic_import_references
      .add_import_reference(data.import_span, vec![]);
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
      .add_import_reference(data.import_span, ids.to_vec());
    Some(true)
  }

  fn call_member_chain(
    &self,
    parser: &mut JavascriptParser,
    _expr: &CallExpr,
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
      .add_import_reference(data.import_span, ids.to_vec());
    Some(true)
  }

  fn import_call(&self, parser: &mut JavascriptParser, node: &CallExpr) -> Option<bool> {
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

    let magic_comment_options =
      try_extract_webpack_magic_comment(parser, node.span, dyn_imported.span());
    if magic_comment_options
      .get_webpack_ignore()
      .unwrap_or_default()
    {
      return None;
    }

    let mode = magic_comment_options
      .get_webpack_mode()
      .map(|x| DynamicImportMode::from(x.as_str()))
      .unwrap_or(dynamic_import_mode.expect("should have dynamic_import_mode"));
    let chunk_name = magic_comment_options
      .get_webpack_chunk_name()
      .map(|x| x.to_owned());
    let chunk_prefetch = magic_comment_options
      .get_webpack_prefetch()
      .and_then(|x| parse_order_string(x.as_str()))
      .or(dynamic_import_prefetch);
    let chunk_preload = magic_comment_options
      .get_webpack_preload()
      .and_then(|x| parse_order_string(x.as_str()))
      .or(dynamic_import_preload);
    let fetch_priority = magic_comment_options
      .get_fetch_priority()
      .map(|x| DynamicImportFetchPriority::from(x.as_str()))
      .or(dynamic_import_fetch_priority);
    let include = magic_comment_options.get_webpack_include();
    let exclude = magic_comment_options.get_webpack_exclude();
    let mut exports = magic_comment_options.get_webpack_exports().map(|x| {
      x.iter()
        .map(|name| vec![Atom::from(name.as_str())])
        .collect::<Vec<_>>()
    });

    let referenced_in_destructuring =
      parser.destructuring_assignment_properties_for(&import_call_span);
    let referenced_in_member = parser
      .dynamic_import_references
      .get_import(&import_call_span);
    let is_statical = referenced_in_destructuring.is_some() || referenced_in_member.is_some();
    if is_statical && exports.is_some() {
      let mut error: Error = create_traceable_error(
        "Useless magic comments".into(),
        "You don't need `webpackExports` if the usage of dynamic import is statically analyse-able. You can safely remove the `webpackExports` magic comment.".into(),
        parser.source_file,
        import_call_span.into(),
      );
      error.severity = Severity::Warning;
      error.hide_stack = Some(true);
      parser.add_warning(error.into());
    }
    if let Some(referenced_properties_in_destructuring) = referenced_in_destructuring {
      exports = Some(
        referenced_properties_in_destructuring
          .iter()
          .cloned()
          .map(|x| vec![x.id])
          .collect_vec(),
      );
    }

    let attributes = get_attributes_from_call_expr(node);
    let param = parser.evaluate_expression(dyn_imported.expr.as_ref());

    if param.is_string() {
      if matches!(mode, DynamicImportMode::Eager) {
        let dep = ImportEagerDependency::new(
          param.string().as_str().into(),
          import_call_span.into(),
          exports,
          attributes,
        );
        let dep_idx = parser.next_dependency_idx();
        if let Some(import_references) = parser
          .dynamic_import_references
          .get_import_mut(&import_call_span)
        {
          import_references.dep_locator = Some(ImportDependencyLocator {
            block_idx: None,
            dep_idx,
            dep_type: DependencyType::DynamicImportEager,
          });
        }
        parser.add_dependency(Box::new(dep));
        return Some(true);
      }

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
      let source_map: SharedSourceMap = parser.source_map.clone();
      let mut block = AsyncDependenciesBlock::new(
        *parser.module_identifier,
        Into::<DependencyRange>::into(import_call_span).to_loc(Some(&source_map)),
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
      if let Some(import_references) = parser
        .dynamic_import_references
        .get_import_mut(&import_call_span)
      {
        import_references.dep_locator = Some(ImportDependencyLocator {
          block_idx: Some(block_idx),
          dep_idx: 0,
          dep_type: DependencyType::DynamicImport,
        });
      }
      parser.add_block(Box::new(block));
      Some(true)
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
      if let Some(import_references) = parser
        .dynamic_import_references
        .get_import_mut(&import_call_span)
      {
        import_references.dep_locator = Some(ImportDependencyLocator {
          block_idx: None,
          dep_idx,
          dep_type: DependencyType::ImportContext,
        });
      }
      parser.add_dependency(Box::new(dep));
      Some(true)
    }
  }

  fn finish(&self, parser: &mut JavascriptParser) -> Option<bool> {
    for (locator, references) in parser.dynamic_import_references.take_import_references() {
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
          dep.referenced_exports = Some(references);
        }
        DependencyType::DynamicImportEager => {
          let dep = dep
            .downcast_mut::<ImportEagerDependency>()
            .expect("Failed to downcast to ImportEagerDependency");
          dep.referenced_exports = Some(references);
        }
        DependencyType::ImportContext => {
          let dep = dep
            .downcast_mut::<ImportContextDependency>()
            .expect("Failed to downcast to ImportContextDependency");
          dep.options.referenced_exports = Some(references);
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
