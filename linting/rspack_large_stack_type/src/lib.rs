#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_span;

use clippy_utils::diagnostics::span_lint_and_then;
use rustc_hir::{AmbigArg, HirId, ImplItemKind, ItemKind, QPath, TraitItemKind, Ty as HirTy};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::Span;

dylint_linting::declare_late_lint! {
  /// ### What it does
  ///
  /// Prevents known large Rspack artifact/result types from being used by value in type
  /// positions. These types should cross function and async boundaries through `Box<T>` or
  /// references, and should not be embedded in stack-sized wrappers such as `Result<T>` or
  /// `Option<T>`.
  ///
  /// ### Why is this bad?
  ///
  /// Returning or passing these structures by value creates large stack slots and extra moves in
  /// hot compilation paths. Keeping the owning allocation boxed makes the boundary explicit and
  /// avoids moving the full object through async state machines.
  ///
  /// ### Example
  ///
  /// ```rust
  /// # struct BuildResult;
  /// fn build() -> Result<BuildResult, ()> {
  ///   todo!()
  /// }
  /// ```
  ///
  /// Use instead:
  ///
  /// ```rust
  /// # struct BuildResult;
  /// fn build() -> Result<Box<BuildResult>, ()> {
  ///   todo!()
  /// }
  /// ```
  pub RSPACK_LARGE_STACK_TYPE,
  Deny,
  "large Rspack artifacts and results should be boxed or borrowed instead of passed by value"
}

const LARGE_RSPACK_TYPES: &[&str] = &[
  "BuildDepsValidationResult",
  "BuildModuleGraphArtifact",
  "CodeGenerationResult",
  "CodeGenerationResults",
  "ExportsInfoArtifact",
  "MinimizePersistentCacheArtifact",
  "ParseResult",
  "SourceMapDevToolPluginCacheArtifact",
  "BuildResult",
];

impl<'tcx> LateLintPass<'tcx> for RspackLargeStackType {
  fn check_ty(&mut self, cx: &LateContext<'tcx>, hir_ty: &'tcx HirTy<'tcx, AmbigArg>) {
    if hir_ty.span.from_expansion()
      || hir_ty_is_nested_in_another_ty(cx, hir_ty.hir_id)
      || hir_ty_is_type_alias_value(cx, hir_ty.hir_id)
      || hir_ty_is_impl_self_ty(cx, hir_ty.hir_id)
      || !ty_snippet_mentions_large_type(cx, hir_ty.span)
    {
      return;
    }

    let hir_ty = hir_ty.as_unambig_ty();
    if let Some(large_type) = find_unboxed_large_type(cx, hir_ty) {
      emit_large_type_lint(cx, large_type.span, large_type.name);
    }
  }
}

struct LargeTypeUse {
  name: &'static str,
  span: Span,
}

fn emit_large_type_lint(cx: &LateContext<'_>, span: Span, type_name: &str) {
  span_lint_and_then(
    cx,
    RSPACK_LARGE_STACK_TYPE,
    span,
    format!("`{type_name}` is a large Rspack type used by value"),
    |diag| {
      diag.help(format!(
        "use `Box<{type_name}>` for ownership across function/async boundaries, or pass `&{type_name}` / `&mut {type_name}` when ownership is not needed"
      ));
    },
  );
}

fn hir_ty_is_nested_in_another_ty(cx: &LateContext<'_>, hir_id: HirId) -> bool {
  let parent_id = cx.tcx.parent_hir_id(hir_id);
  matches!(cx.tcx.hir_node(parent_id), rustc_hir::Node::Ty(_))
}

fn hir_ty_is_type_alias_value(cx: &LateContext<'_>, hir_id: HirId) -> bool {
  match cx.tcx.parent_hir_node(hir_id) {
    rustc_hir::Node::Item(item) => {
      matches!(item.kind, ItemKind::TyAlias(_, _, ty) if ty.hir_id == hir_id)
    }
    rustc_hir::Node::TraitItem(item) => {
      matches!(item.kind, TraitItemKind::Type(_, Some(ty)) if ty.hir_id == hir_id)
    }
    rustc_hir::Node::ImplItem(item) => {
      matches!(item.kind, ImplItemKind::Type(ty) if ty.hir_id == hir_id)
    }
    _ => false,
  }
}

fn hir_ty_is_impl_self_ty(cx: &LateContext<'_>, hir_id: HirId) -> bool {
  matches!(
    cx.tcx.parent_hir_node(hir_id),
    rustc_hir::Node::Item(item)
      if matches!(item.kind, ItemKind::Impl(impl_block) if impl_block.self_ty.hir_id == hir_id)
  )
}

fn find_unboxed_large_type(
  cx: &LateContext<'_>,
  hir_ty: &rustc_hir::Ty<'_>,
) -> Option<LargeTypeUse> {
  match hir_ty.kind {
    rustc_hir::TyKind::Ref(..) | rustc_hir::TyKind::Ptr(_) => None,
    rustc_hir::TyKind::Slice(inner)
    | rustc_hir::TyKind::Array(inner, _)
    | rustc_hir::TyKind::Pat(inner, _) => find_unboxed_large_type(cx, inner),
    rustc_hir::TyKind::UnsafeBinder(binder) => find_unboxed_large_type(cx, binder.inner_ty),
    rustc_hir::TyKind::Tup(tys) => tys.iter().find_map(|ty| find_unboxed_large_type(cx, ty)),
    rustc_hir::TyKind::Path(qpath) => find_unboxed_large_type_in_qpath(cx, hir_ty.hir_id, qpath),
    _ => None,
  }
}

fn find_unboxed_large_type_in_qpath(
  cx: &LateContext<'_>,
  hir_id: HirId,
  qpath: QPath<'_>,
) -> Option<LargeTypeUse> {
  match qpath {
    QPath::Resolved(_, path) => {
      let resolved_path = qpath_def_path(cx, hir_id, qpath);
      if let Some(name) = resolved_path.as_deref().and_then(large_rspack_type_name) {
        return Some(LargeTypeUse {
          name,
          span: path.span,
        });
      }

      if resolved_path
        .as_deref()
        .is_some_and(is_large_type_allowed_container)
      {
        return None;
      }

      path
        .segments
        .iter()
        .filter_map(|segment| segment.args)
        .flat_map(|args| args.args.iter())
        .find_map(|arg| find_unboxed_large_type_in_generic_arg(cx, arg))
    }
    QPath::TypeRelative(ty, segment) => find_unboxed_large_type(cx, ty).or_else(|| {
      segment
        .args
        .into_iter()
        .flat_map(|args| args.args.iter())
        .find_map(|arg| find_unboxed_large_type_in_generic_arg(cx, arg))
    }),
  }
}

fn find_unboxed_large_type_in_generic_arg(
  cx: &LateContext<'_>,
  arg: &rustc_hir::GenericArg<'_>,
) -> Option<LargeTypeUse> {
  match arg {
    rustc_hir::GenericArg::Type(ty) => find_unboxed_large_type(cx, ty.as_unambig_ty()),
    rustc_hir::GenericArg::Lifetime(_)
    | rustc_hir::GenericArg::Const(_)
    | rustc_hir::GenericArg::Infer(_) => None,
  }
}

fn qpath_def_path(
  cx: &LateContext<'_>,
  hir_id: rustc_hir::HirId,
  qpath: QPath<'_>,
) -> Option<String> {
  cx.qpath_res(&qpath, hir_id)
    .opt_def_id()
    .map(|def_id| cx.tcx.def_path_str(def_id))
}

fn large_rspack_type_name(path: &str) -> Option<&'static str> {
  LARGE_RSPACK_TYPES
    .iter()
    .copied()
    .find(|name| path.ends_with(&format!("::{name}")))
}

fn is_large_type_allowed_container(path: &str) -> bool {
  [
    "::Box",
    "::Arc",
    "::Rc",
    "::Vec",
    "::VecDeque",
    "::HashMap",
    "::HashSet",
    "::BTreeMap",
    "::BTreeSet",
    "::IndexMap",
    "::IndexSet",
    "::DashMap",
    "::DashSet",
    "::LinkedHashMap",
    "::LinkedHashSet",
    "::BindingCell",
    "::WeakBindingCell",
    "::StealCell",
    "::MemoryGCStorage",
  ]
  .into_iter()
  .any(|suffix| path.ends_with(suffix))
}

fn ty_snippet_mentions_large_type(cx: &LateContext<'_>, span: Span) -> bool {
  cx.tcx
    .sess
    .source_map()
    .span_to_snippet(span)
    .ok()
    .is_some_and(|snippet| LARGE_RSPACK_TYPES.iter().any(|name| snippet.contains(name)))
}

#[test]
fn ui() {
  dylint_testing::ui::Test::example(env!("CARGO_PKG_NAME"), "ui").run();
}
