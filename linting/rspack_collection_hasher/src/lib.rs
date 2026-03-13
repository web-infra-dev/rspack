#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_hir_analysis;
extern crate rustc_middle;
extern crate rustc_span;

use clippy_utils::{diagnostics::span_lint_and_then, res::MaybeDef, sym as clippy_sym};
use rustc_hir::{AmbigArg, Expr, ExprKind, QPath, Ty as HirTy};
use rustc_hir_analysis::lower_ty;
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::ty::{self, GenericArg, Ty};
use rustc_span::Span;

dylint_linting::declare_late_lint! {
  /// ### What it does
  ///
  /// Enforces Rspack's collection hasher policy:
  /// - generic map/set collections must not use Rust's default hasher;
  /// - `Ustr` and `Identifier` keys must use identity-hasher-based collections.
  ///
  /// ### Why is this bad?
  ///
  /// Rspack is a build tool, so Rust's default `RandomState` hasher is unnecessary in internal
  /// hot paths. `Ustr` and `Identifier` already carry precomputed hashes and should avoid
  /// redundant hashing entirely.
  ///
  /// ### Example
  ///
  /// ```rust
  /// use std::collections::HashMap;
  ///
  /// let map = HashMap::<String, usize>::default();
  /// ```
  ///
  /// Use instead:
  ///
  /// ```rust
  /// use rspack_util::fx_hash::FxHashMap;
  ///
  /// let map = FxHashMap::<String, usize>::default();
  /// ```
  pub RSPACK_COLLECTION_HASHER,
  Warn,
  "Rspack collections should use FxHasher or identity-hasher-based aliases instead of Rust's default hasher"
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CollectionKind {
  StdHashMap,
  StdHashSet,
  IndexMap,
  IndexSet,
  DashMap,
  DashSet,
  LinkedHashMap,
  LinkedHashSet,
}

impl CollectionKind {
  fn from_def_path(path: &str) -> Option<Self> {
    if path.starts_with("indexmap::") && path.ends_with("::IndexMap") {
      Some(Self::IndexMap)
    } else if path.starts_with("indexmap::") && path.ends_with("::IndexSet") {
      Some(Self::IndexSet)
    } else if path.starts_with("dashmap::") && path.ends_with("::DashMap") {
      Some(Self::DashMap)
    } else if path.starts_with("dashmap::") && path.ends_with("::DashSet") {
      Some(Self::DashSet)
    } else if path.starts_with("hashlink::") && path.ends_with("::LinkedHashMap") {
      Some(Self::LinkedHashMap)
    } else if path.starts_with("hashlink::") && path.ends_with("::LinkedHashSet") {
      Some(Self::LinkedHashSet)
    } else {
      None
    }
  }

  fn name(self) -> &'static str {
    match self {
      Self::StdHashMap => "std::collections::HashMap",
      Self::StdHashSet => "std::collections::HashSet",
      Self::IndexMap => "indexmap::IndexMap",
      Self::IndexSet => "indexmap::IndexSet",
      Self::DashMap => "dashmap::DashMap",
      Self::DashSet => "dashmap::DashSet",
      Self::LinkedHashMap => "hashlink::LinkedHashMap",
      Self::LinkedHashSet => "hashlink::LinkedHashSet",
    }
  }

  fn hasher_arg_index(self) -> usize {
    match self {
      Self::StdHashMap | Self::IndexMap | Self::DashMap | Self::LinkedHashMap => 2,
      Self::StdHashSet | Self::IndexSet | Self::DashSet | Self::LinkedHashSet => 1,
    }
  }

  fn ordinary_recommendation(self) -> &'static str {
    match self {
      Self::StdHashMap => "rspack_util::fx_hash::FxHashMap<_, _>",
      Self::StdHashSet => "rspack_util::fx_hash::FxHashSet<_>",
      Self::IndexMap => "rspack_util::fx_hash::FxIndexMap<_, _>",
      Self::IndexSet => "rspack_util::fx_hash::FxIndexSet<_>",
      Self::DashMap => "rspack_util::fx_hash::FxDashMap<_, _>",
      Self::DashSet => "rspack_util::fx_hash::FxDashSet<_>",
      Self::LinkedHashMap => "rspack_util::fx_hash::FxLinkedHashMap<_, _>",
      Self::LinkedHashSet => "rspack_util::fx_hash::FxLinkedHashSet<_>",
    }
  }

  fn identifier_recommendation(self) -> &'static str {
    match self {
      Self::StdHashMap => "rspack_collections::IdentifierMap<_>",
      Self::StdHashSet => "rspack_collections::IdentifierSet",
      Self::IndexMap => "rspack_collections::IdentifierIndexMap<_>",
      Self::IndexSet => "rspack_collections::IdentifierIndexSet",
      Self::DashMap => "rspack_collections::IdentifierDashMap<_>",
      Self::DashSet => "rspack_collections::IdentifierDashSet",
      Self::LinkedHashMap => "rspack_collections::IdentifierLinkedMap<_>",
      Self::LinkedHashSet => "rspack_collections::IdentifierLinkedSet",
    }
  }

  fn ustr_recommendation(self) -> &'static str {
    match self {
      Self::StdHashMap => "ustr::UstrMap<_>",
      Self::StdHashSet => "ustr::UstrSet",
      Self::IndexMap => {
        "indexmap::IndexMap<ustr::Ustr, _, std::hash::BuildHasherDefault<ustr::IdentityHasher>>"
      }
      Self::IndexSet => {
        "indexmap::IndexSet<ustr::Ustr, std::hash::BuildHasherDefault<ustr::IdentityHasher>>"
      }
      Self::DashMap => {
        "dashmap::DashMap<ustr::Ustr, _, std::hash::BuildHasherDefault<ustr::IdentityHasher>>"
      }
      Self::DashSet => {
        "dashmap::DashSet<ustr::Ustr, std::hash::BuildHasherDefault<ustr::IdentityHasher>>"
      }
      Self::LinkedHashMap => {
        "hashlink::LinkedHashMap<ustr::Ustr, _, std::hash::BuildHasherDefault<ustr::IdentityHasher>>"
      }
      Self::LinkedHashSet => {
        "hashlink::LinkedHashSet<ustr::Ustr, std::hash::BuildHasherDefault<ustr::IdentityHasher>>"
      }
    }
  }
}

#[derive(Clone, Copy)]
enum KeyFlavor {
  Ordinary,
  Ustr,
  Identifier,
}

struct CollectionUse<'tcx> {
  kind: CollectionKind,
  key_ty: Ty<'tcx>,
  hasher_ty: Ty<'tcx>,
  direct_kind: Option<CollectionKind>,
  explicit_hasher: bool,
}

struct Violation {
  message: String,
  help: String,
}

impl<'tcx> LateLintPass<'tcx> for RspackCollectionHasher {
  fn check_ty(&mut self, cx: &LateContext<'tcx>, hir_ty: &'tcx HirTy<'tcx, AmbigArg>) {
    // `check_ty` sees every type-position HIR node, not only explicit annotations like
    // `let x: HashMap<..>`.
    //
    // For example, in `IdentifierMap::with_capacity_and_hasher(..)`, the left-hand side
    // `IdentifierMap` also shows up as a type node with HIR roughly shaped like:
    // `TyKind::Path(QPath::TypeRelative(<lhs-ty>, <assoc-item-segment>))`.
    //
    // The guard sequence below is intentionally conservative: we avoid lowering HIR nodes that
    // are not real collection type declarations, because dylint/rustc can hit delayed bugs or
    // ICEs when omitted lifetimes or placeholder generics are involved.
    if hir_ty.span.from_expansion() {
      return;
    }

    let hir_ty = hir_ty.as_unambig_ty();
    if hir_ty_has_elided_ref_lifetime(cx, hir_ty) || hir_ty_has_implicit_object_lifetime(hir_ty) {
      return;
    }

    if direct_collection_kind(cx, hir_ty).is_none() && !hir_ty_maybe_collection_alias(cx, hir_ty) {
      return;
    }

    if !hir_ty_has_explicit_type_args(hir_ty) {
      return;
    }

    if direct_collection_kind(cx, hir_ty).is_none() && hir_ty_has_placeholder(hir_ty) {
      return;
    }

    if let Some(collection) = collection_use_from_hir_ty(cx, hir_ty) {
      if let Some(violation) = find_violation(cx, &collection) {
        emit_lint(cx, hir_ty.span, collection.kind.name(), &violation);
      }
    }
  }

  fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
    if expr.span.from_expansion() {
      return;
    }

    let Some(kind) = direct_collection_ctor_kind(cx, expr) else {
      return;
    };

    emit_lint(
      cx,
      expr.span,
      kind.name(),
      &Violation {
        message: "uses Rust's default hasher".to_string(),
        help: format!(
          "Rspack should prefer {} for ordinary keys",
          kind.ordinary_recommendation()
        ),
      },
    );
  }
}

fn emit_lint(cx: &LateContext<'_>, span: Span, collection_name: &str, violation: &Violation) {
  span_lint_and_then(
    cx,
    RSPACK_COLLECTION_HASHER,
    span,
    format!("{collection_name} {}", violation.message),
    |diag| {
      diag.help(violation.help.clone());
    },
  );
}

fn collection_use_from_ty<'tcx>(
  cx: &LateContext<'tcx>,
  ty: Ty<'tcx>,
) -> Option<CollectionUse<'tcx>> {
  let ty = ty.peel_refs();
  let ty::Adt(def, args) = ty.kind() else {
    return None;
  };

  let kind = if ty.is_diag_item(cx, clippy_sym::HashMap) {
    CollectionKind::StdHashMap
  } else if ty.is_diag_item(cx, clippy_sym::HashSet) {
    CollectionKind::StdHashSet
  } else {
    CollectionKind::from_def_path(cx.tcx.def_path_str(def.did()).as_str())?
  };
  let key_ty = type_arg(args, 0)?;
  let hasher_ty = type_arg(args, kind.hasher_arg_index())?;

  Some(CollectionUse {
    kind,
    key_ty,
    hasher_ty,
    direct_kind: None,
    explicit_hasher: true,
  })
}

fn collection_use_from_hir_ty<'tcx>(
  cx: &LateContext<'tcx>,
  hir_ty: &rustc_hir::Ty<'tcx>,
) -> Option<CollectionUse<'tcx>> {
  let mut collection = collection_use_from_ty(cx, lower_ty(cx.tcx, hir_ty))?;
  collection.direct_kind = direct_collection_kind(cx, hir_ty);
  collection.explicit_hasher =
    explicit_hasher_arg_count(hir_ty) > collection.kind.hasher_arg_index();
  Some(collection)
}

fn type_arg<'tcx>(args: ty::GenericArgsRef<'tcx>, index: usize) -> Option<Ty<'tcx>> {
  args.get(index).and_then(|arg| GenericArg::as_type(*arg))
}

fn find_violation(cx: &LateContext<'_>, collection: &CollectionUse<'_>) -> Option<Violation> {
  match key_flavor(cx, collection.key_ty) {
    KeyFlavor::Identifier if !is_identity_hasher(cx, collection.hasher_ty) => Some(Violation {
      message: "uses a non-identity hasher for `Identifier` keys".to_string(),
      help: format!(
        "`Identifier` stores a precomputed `Ustr` hash in Rspack; use {} instead",
        collection.kind.identifier_recommendation()
      ),
    }),
    KeyFlavor::Ustr if !is_identity_hasher(cx, collection.hasher_ty) => Some(Violation {
      message: "uses a non-identity hasher for `Ustr` keys".to_string(),
      help: format!(
        "`Ustr` already stores a precomputed hash; use {} instead",
        collection.kind.ustr_recommendation()
      ),
    }),
    KeyFlavor::Ordinary
      if ordinary_keys_use_implicit_default_hasher(collection)
        || is_rust_default_hasher(cx, collection.hasher_ty) =>
    {
      Some(Violation {
        message: "uses Rust's default hasher".to_string(),
        help: format!(
          "Rspack should prefer {} for ordinary keys",
          collection.kind.ordinary_recommendation()
        ),
      })
    }
    _ => None,
  }
}

fn ordinary_keys_use_implicit_default_hasher(collection: &CollectionUse<'_>) -> bool {
  collection.direct_kind.is_some() && !collection.explicit_hasher
}

fn key_flavor(cx: &LateContext<'_>, key_ty: Ty<'_>) -> KeyFlavor {
  let key_ty = key_ty.peel_refs();
  let ty::Adt(def, _) = key_ty.kind() else {
    return KeyFlavor::Ordinary;
  };

  let path = cx.tcx.def_path_str(def.did());
  if path.ends_with("::Ustr") {
    KeyFlavor::Ustr
  } else if path.contains("rspack_collections") && path.ends_with("::Identifier") {
    KeyFlavor::Identifier
  } else {
    KeyFlavor::Ordinary
  }
}

fn is_identity_hasher(cx: &LateContext<'_>, hasher_ty: Ty<'_>) -> bool {
  let hasher_ty = hasher_ty.peel_refs();
  let ty::Adt(def, args) = hasher_ty.kind() else {
    return false;
  };

  let path = cx.tcx.def_path_str(def.did());
  if !path.ends_with("BuildHasherDefault") {
    return false;
  }

  let Some(inner_hasher) = type_arg(args, 0) else {
    return false;
  };

  matches_identity_hasher(cx, inner_hasher)
}

fn matches_identity_hasher(cx: &LateContext<'_>, hasher_ty: Ty<'_>) -> bool {
  let hasher_ty = hasher_ty.peel_refs();
  let ty::Adt(def, _) = hasher_ty.kind() else {
    return false;
  };

  let path = cx.tcx.def_path_str(def.did());
  path.ends_with("::IdentityHasher") || path.ends_with("::IdentifierHasher")
}

fn is_rust_default_hasher(cx: &LateContext<'_>, hasher_ty: Ty<'_>) -> bool {
  let hasher_ty = hasher_ty.peel_refs();
  let ty::Adt(def, args) = hasher_ty.kind() else {
    return false;
  };

  let path = cx.tcx.def_path_str(def.did());
  if path.ends_with("RandomState") {
    return true;
  }

  if path.ends_with("BuildHasherDefault") {
    if let Some(inner_hasher) = type_arg(args, 0) {
      return matches_default_hasher_impl(cx, inner_hasher);
    }
  }

  false
}

fn matches_default_hasher_impl(cx: &LateContext<'_>, hasher_ty: Ty<'_>) -> bool {
  let hasher_ty = hasher_ty.peel_refs();
  let ty::Adt(def, _) = hasher_ty.kind() else {
    return false;
  };

  cx.tcx.def_path_str(def.did()).ends_with("DefaultHasher")
}

fn direct_collection_ctor_kind(cx: &LateContext<'_>, expr: &Expr<'_>) -> Option<CollectionKind> {
  // Only match constructor-style calls such as `Foo::new(..)` and `Foo::default(..)`,
  // whose HIR is roughly `ExprKind::Call(<callee>, <args>)` with `<callee>` being a path.
  //
  // We intentionally do not call `expr_ty(expr)` here. Whole-expression type inference can
  // walk into unrelated omitted-lifetime nodes elsewhere in the function body and trigger
  // delayed bugs in the dylint/rustc driver.
  let ExprKind::Call(callee, _) = expr.kind else {
    return None;
  };

  if callee_uses_approved_alias(cx, callee) {
    return None;
  }

  let path = callee_def_path(cx, callee)?;
  if !(path.ends_with("::new")
    || path.ends_with("::default")
    || path.ends_with("::with_capacity")
    || path.ends_with("::with_capacity_and_hasher")
    || path.ends_with("::with_hasher")
    || path.ends_with("::from_iter"))
  {
    return None;
  }

  collection_kind_from_ctor_path(path.as_str())
}

fn callee_uses_approved_alias(cx: &LateContext<'_>, expr: &Expr<'_>) -> bool {
  let ExprKind::Path(qpath) = expr.kind else {
    return false;
  };

  match qpath {
    QPath::Resolved(_, path) => {
      // Paths such as `FxHashMap::default()` usually lower to
      // `ExprKind::Path(QPath::Resolved(..))`.
      path
        .segments
        .iter()
        .any(|segment| is_collection_alias_name(segment.ident.as_str()))
        || qpath_def_path(cx, expr.hir_id, qpath)
          .is_some_and(|path| is_approved_collection_alias_path(path.as_str()))
    }
    QPath::TypeRelative(ty, segment) => {
      // Associated calls like `IdentifierMap::with_capacity_and_hasher(..)` or
      // `HashMap::with_capacity_and_hasher(..)` usually lower to:
      // `ExprKind::Path(QPath::TypeRelative(<lhs-ty>, <method-segment>))`.
      //
      // We have to inspect both the left-hand type node `ty` and the resolved def path to tell
      // apart:
      // - the real `std::collections::HashMap`
      // - a local rename such as `use rustc_hash::FxHashMap as HashMap`
      hir_ty_maybe_collection_alias(cx, ty)
        || is_collection_alias_name(segment.ident.as_str())
        || qpath_def_path(cx, expr.hir_id, qpath)
          .is_some_and(|path| is_approved_collection_alias_path(path.as_str()))
    }
  }
}

fn collection_kind_from_ctor_path(path: &str) -> Option<CollectionKind> {
  if path.contains("::LinkedHashMap") {
    Some(CollectionKind::LinkedHashMap)
  } else if path.contains("::LinkedHashSet") {
    Some(CollectionKind::LinkedHashSet)
  } else if path.contains("::IndexMap") {
    Some(CollectionKind::IndexMap)
  } else if path.contains("::IndexSet") {
    Some(CollectionKind::IndexSet)
  } else if path.contains("::DashMap") {
    Some(CollectionKind::DashMap)
  } else if path.contains("::DashSet") {
    Some(CollectionKind::DashSet)
  } else if path.contains("::HashMap") {
    Some(CollectionKind::StdHashMap)
  } else if path.contains("::HashSet") {
    Some(CollectionKind::StdHashSet)
  } else {
    None
  }
}

fn callee_def_path(cx: &LateContext<'_>, expr: &Expr<'_>) -> Option<String> {
  let ExprKind::Path(qpath) = expr.kind else {
    return None;
  };

  cx.qpath_res(&qpath, expr.hir_id)
    .opt_def_id()
    .map(|def_id| cx.tcx.def_path_str(def_id))
}

fn direct_collection_kind(
  cx: &LateContext<'_>,
  hir_ty: &rustc_hir::Ty<'_>,
) -> Option<CollectionKind> {
  // Direct type references like `HashMap<K, V>`, `IndexMap<K, V>`, or `DashMap<K, V>` usually
  // lower to `TyKind::Path(QPath::Resolved(..))`.
  //
  // We prefer resolving the `def_id` directly from HIR here so we can recognize the collection
  // without eagerly lowering the full type.
  let rustc_hir::TyKind::Path(qpath) = hir_ty.kind else {
    return None;
  };

  let def_id = cx.qpath_res(&qpath, hir_ty.hir_id).opt_def_id()?;

  if cx.tcx.get_diagnostic_name(def_id) == Some(clippy_sym::HashMap) {
    Some(CollectionKind::StdHashMap)
  } else if cx.tcx.get_diagnostic_name(def_id) == Some(clippy_sym::HashSet) {
    Some(CollectionKind::StdHashSet)
  } else {
    CollectionKind::from_def_path(cx.tcx.def_path_str(def_id).as_str())
  }
}

fn hir_ty_has_placeholder<'tcx, A>(hir_ty: &rustc_hir::Ty<'tcx, A>) -> bool {
  match hir_ty.kind {
    rustc_hir::TyKind::Infer(_) => true,
    rustc_hir::TyKind::Slice(inner)
    | rustc_hir::TyKind::Array(inner, _)
    | rustc_hir::TyKind::Ptr(rustc_hir::MutTy { ty: inner, .. })
    | rustc_hir::TyKind::Ref(_, rustc_hir::MutTy { ty: inner, .. }) => {
      hir_ty_has_placeholder(inner)
    }
    rustc_hir::TyKind::Tup(tys) => tys.iter().any(hir_ty_has_placeholder),
    rustc_hir::TyKind::Path(qpath) => qpath_has_placeholder(qpath),
    _ => false,
  }
}

fn hir_ty_has_explicit_type_args(hir_ty: &rustc_hir::Ty<'_>) -> bool {
  match hir_ty.kind {
    rustc_hir::TyKind::Path(QPath::Resolved(_, path)) => path
      .segments
      .iter()
      .any(|segment| segment.args.is_some_and(|args| !args.args.is_empty())),
    rustc_hir::TyKind::Path(QPath::TypeRelative(ty, segment)) => {
      hir_ty_has_explicit_type_args(ty) || segment.args.is_some_and(|args| !args.args.is_empty())
    }
    _ => false,
  }
}

fn hir_ty_maybe_collection_alias(cx: &LateContext<'_>, hir_ty: &rustc_hir::Ty<'_>) -> bool {
  let rustc_hir::TyKind::Path(qpath) = hir_ty.kind else {
    return false;
  };

  let ident = match qpath {
    QPath::Resolved(_, path) => path.segments.last().map(|segment| segment.ident.as_str()),
    QPath::TypeRelative(_, segment) => Some(segment.ident.as_str()),
  };

  // Check both the final source-level segment name and the resolved def path.
  //
  // The def path lookup is what makes renamed imports work:
  // `use rustc_hash::FxHashMap as HashMap;`
  // still appears as `HashMap` in HIR surface syntax, but resolves back to
  // `rustc_hash::FxHashMap`.
  ident.is_some_and(is_collection_alias_name)
    || qpath_def_path(cx, hir_ty.hir_id, qpath)
      .is_some_and(|path| is_approved_collection_alias_path(path.as_str()))
}

fn is_collection_alias_name(name: &str) -> bool {
  matches!(
    name,
    "FxHashMap"
      | "FxHashSet"
      | "FxIndexMap"
      | "FxIndexSet"
      | "FxDashMap"
      | "FxDashSet"
      | "FxLinkedHashMap"
      | "FxLinkedHashSet"
      | "UstrMap"
      | "UstrSet"
      | "IdentifierMap"
      | "IdentifierSet"
      | "IdentifierIndexMap"
      | "IdentifierIndexSet"
      | "IdentifierDashMap"
      | "IdentifierDashSet"
      | "IdentifierLinkedMap"
      | "IdentifierLinkedSet"
  )
}

fn is_approved_collection_alias_path(path: &str) -> bool {
  path.ends_with("::FxHashMap")
    || path.ends_with("::FxHashSet")
    || path.ends_with("::FxIndexMap")
    || path.ends_with("::FxIndexSet")
    || path.ends_with("::FxDashMap")
    || path.ends_with("::FxDashSet")
    || path.ends_with("::FxLinkedHashMap")
    || path.ends_with("::FxLinkedHashSet")
    || path.ends_with("::UstrMap")
    || path.ends_with("::UstrSet")
    || path.ends_with("::IdentifierMap")
    || path.ends_with("::IdentifierSet")
    || path.ends_with("::IdentifierIndexMap")
    || path.ends_with("::IdentifierIndexSet")
    || path.ends_with("::IdentifierDashMap")
    || path.ends_with("::IdentifierDashSet")
    || path.ends_with("::IdentifierLinkedMap")
    || path.ends_with("::IdentifierLinkedSet")
}

fn qpath_def_path(
  cx: &LateContext<'_>,
  hir_id: rustc_hir::HirId,
  qpath: QPath<'_>,
) -> Option<String> {
  // `QPath` describes how the path looks in HIR syntax, while `qpath_res` answers what it
  // actually resolved to after name resolution.
  //
  // Normalize both into a def-path string so alias checks can reuse the same representation.
  cx.qpath_res(&qpath, hir_id)
    .opt_def_id()
    .map(|def_id| cx.tcx.def_path_str(def_id))
}

fn hir_ty_has_elided_ref_lifetime(cx: &LateContext<'_>, hir_ty: &rustc_hir::Ty<'_>) -> bool {
  // We intentionally inspect the source snippet instead of chasing the full lifetime HIR.
  //
  // The only question we need to answer is conservative: does this type contain an elided
  // reference lifetime like `&T` or `&mut T`? If yes, further lowering has been prone to
  // delayed bugs in the dylint/rustc driver, so skipping that node is safer than crashing the
  // whole lint run.
  let Ok(snippet) = cx.tcx.sess.source_map().span_to_snippet(hir_ty.span) else {
    return false;
  };

  let bytes = snippet.as_bytes();
  let mut idx = 0;
  while idx < bytes.len() {
    if bytes[idx] == b'&' {
      idx += 1;
      while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
        idx += 1;
      }
      if idx < bytes.len() && bytes[idx] != b'\'' {
        return true;
      }
    } else {
      idx += 1;
    }
  }
  false
}

fn hir_ty_has_implicit_object_lifetime<'tcx, A>(hir_ty: &rustc_hir::Ty<'tcx, A>) -> bool {
  match hir_ty.kind {
    // `dyn Trait` lowers to `TyKind::TraitObject(.., TaggedRef<Lifetime, TraitObjectSyntax>)`.
    // When the object lifetime is omitted, the tagged lifetime carries
    // `LifetimeKind::ImplicitObjectLifetimeDefault`.
    rustc_hir::TyKind::TraitObject(_, lifetime) => {
      matches!(
        lifetime.pointer().kind,
        rustc_hir::LifetimeKind::ImplicitObjectLifetimeDefault
      )
    }
    rustc_hir::TyKind::Slice(inner)
    | rustc_hir::TyKind::Array(inner, _)
    | rustc_hir::TyKind::Pat(inner, _)
    | rustc_hir::TyKind::Ptr(rustc_hir::MutTy { ty: inner, .. })
    | rustc_hir::TyKind::Ref(_, rustc_hir::MutTy { ty: inner, .. }) => {
      hir_ty_has_implicit_object_lifetime(inner)
    }
    rustc_hir::TyKind::UnsafeBinder(binder) => hir_ty_has_implicit_object_lifetime(binder.inner_ty),
    rustc_hir::TyKind::Tup(tys) => tys.iter().any(hir_ty_has_implicit_object_lifetime),
    rustc_hir::TyKind::Path(qpath) => qpath_has_implicit_object_lifetime(qpath),
    _ => false,
  }
}

fn qpath_has_placeholder(qpath: QPath<'_>) -> bool {
  match qpath {
    QPath::Resolved(_, path) => path.segments.iter().any(path_segment_has_placeholder),
    QPath::TypeRelative(ty, segment) => {
      hir_ty_has_placeholder(ty) || path_segment_has_placeholder(segment)
    }
  }
}

fn qpath_has_implicit_object_lifetime(qpath: QPath<'_>) -> bool {
  match qpath {
    QPath::Resolved(_, path) => path
      .segments
      .iter()
      .any(path_segment_has_implicit_object_lifetime),
    QPath::TypeRelative(ty, segment) => {
      hir_ty_has_implicit_object_lifetime(ty) || path_segment_has_implicit_object_lifetime(segment)
    }
  }
}

fn path_segment_has_placeholder(segment: &rustc_hir::PathSegment<'_>) -> bool {
  segment
    .args
    .is_some_and(|args| args.args.iter().any(generic_arg_has_placeholder))
}

fn path_segment_has_implicit_object_lifetime(segment: &rustc_hir::PathSegment<'_>) -> bool {
  segment.args.is_some_and(|args| {
    args
      .args
      .iter()
      .any(generic_arg_has_implicit_object_lifetime)
  })
}

fn generic_arg_has_placeholder(arg: &rustc_hir::GenericArg<'_>) -> bool {
  match arg {
    rustc_hir::GenericArg::Type(ty) => hir_ty_has_placeholder(ty),
    rustc_hir::GenericArg::Infer(_) => true,
    rustc_hir::GenericArg::Lifetime(_) | rustc_hir::GenericArg::Const(_) => false,
  }
}

fn generic_arg_has_implicit_object_lifetime(arg: &rustc_hir::GenericArg<'_>) -> bool {
  match arg {
    rustc_hir::GenericArg::Type(ty) => hir_ty_has_implicit_object_lifetime(ty),
    rustc_hir::GenericArg::Infer(_)
    | rustc_hir::GenericArg::Lifetime(_)
    | rustc_hir::GenericArg::Const(_) => false,
  }
}

fn explicit_hasher_arg_count(hir_ty: &rustc_hir::Ty<'_>) -> usize {
  let rustc_hir::TyKind::Path(QPath::Resolved(_, path)) = hir_ty.kind else {
    return 0;
  };

  path
    .segments
    .last()
    .and_then(|segment| segment.args.as_ref())
    .map(|args| {
      args
        .args
        .iter()
        .filter(|arg| matches!(arg, rustc_hir::GenericArg::Type(_)))
        .count()
    })
    .unwrap_or(0)
}

#[test]
fn ui() {
  dylint_testing::ui::Test::example(env!("CARGO_PKG_NAME"), "ui").run();
}
