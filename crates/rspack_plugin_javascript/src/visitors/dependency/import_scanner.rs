use std::sync::Arc;

use rspack_core::{
  clean_regexp_in_context_module, context_reg_exp, AsyncDependenciesBlock, DependencyLocation,
  DynamicImportMode, ErrorSpan, GroupOptions, JavascriptParserOptions, ModuleIdentifier,
};
use rspack_core::{BoxDependency, BuildMeta, ChunkGroupOptions, ContextMode};
use rspack_core::{ContextNameSpaceObject, ContextOptions, DependencyCategory, SpanExt};
use rspack_error::miette::Diagnostic;
use rspack_regex::{regexp_as_str, RspackRegex};
use rustc_hash::FxHashSet;
use swc_core::common::comments::Comments;
use swc_core::common::{SourceFile, Spanned};
use swc_core::ecma::ast::{CallExpr, Callee, Expr, Lit};
use swc_core::ecma::atoms::JsWord;
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};

use super::context_helper::scanner_context_module;
use super::{is_import_meta_context_call, parse_order_string};
use crate::dependency::{ImportContextDependency, ImportDependency};
use crate::dependency::{ImportEagerDependency, ImportMetaContextDependency};
use crate::no_visit_ignored_stmt;
use crate::utils::{get_bool_by_obj_prop, get_literal_str_by_obj_prop, get_regex_by_obj_prop};
use crate::webpack_comment::try_extract_webpack_magic_comment;

pub struct ImportScanner<'a> {
  pub source_file: Arc<SourceFile>,
  pub module_identifier: ModuleIdentifier,
  pub dependencies: &'a mut Vec<BoxDependency>,
  pub blocks: &'a mut Vec<AsyncDependenciesBlock>,
  pub comments: Option<&'a dyn Comments>,
  pub build_meta: &'a BuildMeta,
  pub options: Option<&'a JavascriptParserOptions>,
  pub warning_diagnostics: &'a mut Vec<Box<dyn Diagnostic + Send + Sync>>,
  pub ignored: &'a mut FxHashSet<DependencyLocation>,
}

fn create_import_meta_context_dependency(node: &CallExpr) -> Option<ImportMetaContextDependency> {
  assert!(node.callee.is_expr());
  let dyn_imported = node.args.first()?;
  if dyn_imported.spread.is_some() {
    return None;
  }
  let context = dyn_imported
    .expr
    .as_lit()
    .and_then(|lit| {
      if let Lit::Str(str) = lit {
        return Some(str.value.to_string());
      }
      None
    })
    // TODO: should've used expression evaluation to handle cases like `abc${"efg"}`, etc.
    .or_else(|| {
      if let Some(tpl) = dyn_imported.expr.as_tpl()
        && tpl.exprs.is_empty()
        && tpl.quasis.len() == 1
        && let Some(el) = tpl.quasis.first()
      {
        return Some(el.raw.to_string());
      }
      None
    })?;
  let reg = r"^\.\/.*$";
  let reg_str = reg.to_string();
  let context_options = if let Some(obj) = node.args.get(1).and_then(|arg| arg.expr.as_object()) {
    let regexp = get_regex_by_obj_prop(obj, "regExp")
      .map(|regexp| RspackRegex::try_from(regexp).expect("reg failed"))
      .unwrap_or(RspackRegex::new(reg).expect("reg failed"));
    // let include = get_regex_by_obj_prop(obj, "include")
    //   .map(|regexp| RspackRegex::try_from(regexp).expect("reg failed"));
    // let exclude = get_regex_by_obj_prop(obj, "include")
    //   .map(|regexp| RspackRegex::try_from(regexp).expect("reg failed"));
    let mode = get_literal_str_by_obj_prop(obj, "mode")
      .map(|s| s.value.to_string().as_str().into())
      .unwrap_or(ContextMode::Sync);
    let recursive = get_bool_by_obj_prop(obj, "recursive")
      .map(|bool| bool.value)
      .unwrap_or(true);
    let reg_str = regexp_as_str(&regexp).to_string();
    ContextOptions {
      chunk_name: None,
      reg_exp: clean_regexp_in_context_module(regexp),
      reg_str,
      include: None,
      exclude: None,
      recursive,
      category: DependencyCategory::Esm,
      request: context,
      namespace_object: ContextNameSpaceObject::Unset,
      mode,
    }
  } else {
    ContextOptions {
      chunk_name: None,
      recursive: true,
      mode: ContextMode::Sync,
      include: None,
      exclude: None,
      reg_exp: context_reg_exp(reg, ""),
      reg_str,
      category: DependencyCategory::Esm,
      request: context,
      namespace_object: ContextNameSpaceObject::Unset,
    }
  };
  Some(ImportMetaContextDependency::new(
    node.span.real_lo(),
    node.span.real_hi(),
    context_options,
    Some(node.span.into()),
  ))
}

impl<'a> ImportScanner<'a> {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    source_file: Arc<SourceFile>,
    module_identifier: ModuleIdentifier,
    dependencies: &'a mut Vec<BoxDependency>,
    blocks: &'a mut Vec<AsyncDependenciesBlock>,
    comments: Option<&'a dyn Comments>,
    build_meta: &'a BuildMeta,
    options: Option<&'a JavascriptParserOptions>,
    warning_diagnostics: &'a mut Vec<Box<dyn Diagnostic + Send + Sync>>,
    ignored: &'a mut FxHashSet<DependencyLocation>,
  ) -> Self {
    Self {
      source_file,
      module_identifier,
      dependencies,
      blocks,
      comments,
      build_meta,
      options,
      warning_diagnostics,
      ignored,
    }
  }
}

impl Visit for ImportScanner<'_> {
  noop_visit_type!();
  no_visit_ignored_stmt!();

  fn visit_call_expr(&mut self, node: &CallExpr) {
    if !node.args.is_empty()
      && is_import_meta_context_call(node)
      && let Some(dep) = create_import_meta_context_dependency(node)
    {
      self.dependencies.push(Box::new(dep));
      return;
    }
    let Callee::Import(import_call) = &node.callee else {
      node.visit_children_with(self);
      return;
    };
    let Some(dyn_imported) = node.args.first() else {
      node.visit_children_with(self);
      return;
    };
    if dyn_imported.spread.is_some() {
      node.visit_children_with(self);
      return;
    }

    let mode = self
      .options
      .map(|o| o.dynamic_import_mode)
      .unwrap_or_default();

    let dynamic_import_preload = self
      .options
      .map(|o| o.dynamic_import_preload)
      .and_then(|o| o.get_order());

    let dynamic_import_prefetch = self
      .options
      .map(|o| o.dynamic_import_prefetch)
      .and_then(|o| o.get_order());

    match dyn_imported.expr.as_ref() {
      Expr::Lit(Lit::Str(imported)) => {
        if matches!(mode, DynamicImportMode::Eager) {
          let dep = ImportEagerDependency::new(
            node.span.real_lo(),
            node.span.real_hi(),
            imported.value.clone(),
            Some(node.span.into()),
            // TODO scan dynamic import referenced exports
            None,
          );
          self.dependencies.push(Box::new(dep));
          return;
        }
        let magic_comment_options = try_extract_webpack_magic_comment(
          &self.source_file,
          &self.comments,
          node.span,
          imported.span,
          self.warning_diagnostics,
        );
        if magic_comment_options
          .get_webpack_ignore()
          .unwrap_or_default()
        {
          return;
        }
        let chunk_name = magic_comment_options
          .get_webpack_chunk_name()
          .map(|x| x.to_owned());
        let chunk_prefetch = magic_comment_options
          .get_webpack_prefetch()
          .and_then(|x| parse_order_string(x.as_str()));
        let chunk_preload = magic_comment_options
          .get_webpack_preload()
          .and_then(|x| parse_order_string(x.as_str()));
        let span = ErrorSpan::from(node.span);
        let dep = Box::new(ImportDependency::new(
          node.span.real_lo(),
          node.span.real_hi(),
          imported.value.clone(),
          Some(span),
          // TODO scan dynamic import referenced exports
          None,
        ));
        let mut block = AsyncDependenciesBlock::new(
          self.module_identifier,
          Some(DependencyLocation::new(span.start, span.end)),
        );
        block.set_group_options(GroupOptions::ChunkGroup(ChunkGroupOptions::new(
          chunk_name,
          chunk_preload.or(dynamic_import_preload),
          chunk_prefetch.or(dynamic_import_prefetch),
        )));
        block.add_dependency(dep);
        self.blocks.push(block);
      }
      Expr::Tpl(tpl) if tpl.quasis.len() == 1 => {
        let magic_comment_options = try_extract_webpack_magic_comment(
          &self.source_file,
          &self.comments,
          node.span,
          tpl.span,
          self.warning_diagnostics,
        );
        let chunk_name = magic_comment_options
          .get_webpack_chunk_name()
          .map(|x| x.to_owned());
        let chunk_prefetch = magic_comment_options
          .get_webpack_prefetch()
          .and_then(|x| parse_order_string(x.as_str()));
        let chunk_preload = magic_comment_options
          .get_webpack_preload()
          .and_then(|x| parse_order_string(x.as_str()));
        let request = JsWord::from(
          tpl
            .quasis
            .first()
            .expect("should have one quasis")
            .raw
            .to_string(),
        );
        let span = ErrorSpan::from(node.span);
        let dep = Box::new(ImportDependency::new(
          node.span.real_lo(),
          node.span.real_hi(),
          request,
          Some(span),
          None,
        ));
        let mut block = AsyncDependenciesBlock::new(
          self.module_identifier,
          Some(DependencyLocation::new(span.start, span.end)),
        );
        block.set_group_options(GroupOptions::ChunkGroup(ChunkGroupOptions::new(
          chunk_name,
          chunk_preload.or(dynamic_import_preload),
          chunk_prefetch.or(dynamic_import_prefetch),
        )));
        block.add_dependency(dep);
        self.blocks.push(block);
      }
      _ => {
        let Some((context, reg)) = scanner_context_module(dyn_imported.expr.as_ref()) else {
          return;
        };
        let magic_comment_options = try_extract_webpack_magic_comment(
          &self.source_file,
          &self.comments,
          node.span,
          dyn_imported.span(),
          self.warning_diagnostics,
        );
        let chunk_name = magic_comment_options
          .get_webpack_chunk_name()
          .map(|x| x.to_owned());
        self
          .dependencies
          .push(Box::new(ImportContextDependency::new(
            import_call.span.real_lo(),
            import_call.span.real_hi(),
            node.span.real_hi(),
            ContextOptions {
              chunk_name,
              mode: ContextMode::Lazy,
              recursive: true,
              reg_exp: context_reg_exp(&reg, ""),
              reg_str: reg,
              include: None,
              exclude: None,
              category: DependencyCategory::Esm,
              request: context,
              namespace_object: if self.build_meta.strict_harmony_module {
                ContextNameSpaceObject::Strict
              } else {
                ContextNameSpaceObject::Bool(true)
              },
            },
            Some(node.span.into()),
          )));
      }
    }
  }
}
