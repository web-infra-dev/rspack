//! Known-pure global constructors and functions for tree-shaking.
//!
//! When tree-shaking analyses side effects, a `new Set()` or `Boolean(x)` with
//! the real (unresolved) global callee can be treated as side-effect-free if the
//! arguments satisfy the gate returned by [`classify_pure_global`].
//!
//! ## Safety invariants
//!
//! * **Shadowing**: the callee identifier must have the unresolved syntax context
//!   (`ctxt == unresolved_ctxt`), so a module-local `const Set = …` is never
//!   mistaken for the built-in.
//! * **Arguments**: the returned [`ArgGate`] describes the worst-case arg
//!   interaction. Each variant has a different arg-shape requirement; see
//!   [`ArgGate`] docs.
//! * **Throwing built-ins**: classification rejects callees whose throw
//!   behaviour depends on argument *values* (not just types). For example,
//!   `new Set(1)` throws `TypeError` and `new Uint8Array(-1)` throws
//!   `RangeError`, so they live behind stricter gates than `String("x")`.

use swc_core::{
  common::SyntaxContext,
  ecma::ast::{Expr, ExprOrSpread, Lit, MemberProp, UnaryOp},
};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Where the callee appears syntactically.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalleePosition {
  /// `new Callee(…)`
  New,
  /// `Callee(…)` or `Callee.method(…)`
  Call,
}

/// What level of argument checking a known-pure callee requires.
///
/// Each variant has its own arg-shape predicate; see [`check_arg_gate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgGate {
  /// The callee never coerces, iterates, or throws based on argument
  /// *values* — only nested expression side effects matter. Caller must
  /// still recursively check that arg expressions themselves are pure
  /// (e.g. `Boolean(sideEffect())` is impure because of `sideEffect()`).
  ///
  /// Examples: `Boolean(x)`, `Array.isArray(x)`, `Object.is(a, b)`.
  AnyPureArgs,

  /// All args must be *trivially safe*: a literal (number/string/bool/null/
  /// regex/bigint) or `undefined`/`NaN`/`Infinity`. Used for callees that
  /// don't throw and don't trigger user code on any literal argument.
  ///
  /// Examples: `String("x")`, `Object("y")`, `Symbol("desc")`.
  TriviallySafeArgs,

  /// All args must be `null`, `undefined`, `void <safe>`, or absent. Used
  /// for callees that throw on non-iterable / non-numeric values, where
  /// any literal other than nullish would be a runtime throw.
  ///
  /// Examples: `new Set()`, `new Map()`, `new Date()`, `new Number()`,
  /// `new Array()`. (`new Set(1)` would throw TypeError; `new Array(-1)`
  /// would throw RangeError.)
  NullishOrNoArgs,

  /// All args must be nullish or a *non-negative integer literal* fitting
  /// in `[0, 2^32)`. Used for `Length` constructors that throw `RangeError`
  /// on negative, fractional, or NaN/Infinity arguments.
  ///
  /// Examples: `new Uint8Array(16)`, `new ArrayBuffer(0)`. (`new
  /// Uint8Array(-1)` and `new Uint8Array(1.5)` both throw RangeError.)
  NullishOrNonNegativeIntLiteral,
}

/// Classify `callee` as a known-pure global.
///
/// Returns `Some(gate)` when:
/// 1. The callee resolves to an unresolved global (not a local binding).
/// 2. The name + `position` combination is in the allowlist.
///
/// The caller is responsible for checking the arguments against the
/// returned gate via [`check_arg_gate`] (or its components).
pub fn classify_pure_global(
  callee: &Expr,
  unresolved_ctxt: SyntaxContext,
  position: CalleePosition,
) -> Option<ArgGate> {
  match callee {
    Expr::Ident(ident) if ident.ctxt == unresolved_ctxt => {
      classify_ident(ident.sym.as_str(), position)
    }
    // `Array.isArray(…)` — only valid as a Call.
    Expr::Member(member) if position == CalleePosition::Call => {
      let MemberProp::Ident(prop) = &member.prop else {
        return None;
      };
      let Expr::Ident(obj) = member.obj.as_ref() else {
        return None;
      };
      if obj.ctxt != unresolved_ctxt {
        return None;
      }
      classify_member(obj.sym.as_str(), prop.sym.as_str())
    }
    _ => None,
  }
}

/// Validate `args` against `gate`. For [`ArgGate::AnyPureArgs`] returns
/// `true` unconditionally — the caller is expected to do its own recursive
/// arg-purity check (because args can be arbitrary expressions, not just
/// literals).
pub fn check_arg_gate(
  gate: ArgGate,
  args: &[ExprOrSpread],
  unresolved_ctxt: SyntaxContext,
) -> bool {
  match gate {
    ArgGate::AnyPureArgs => true,
    ArgGate::TriviallySafeArgs => are_args_trivially_safe(args, unresolved_ctxt),
    ArgGate::NullishOrNoArgs => are_args_nullish_only(args, unresolved_ctxt),
    ArgGate::NullishOrNonNegativeIntLiteral => {
      are_args_nullish_or_int_literal(args, unresolved_ctxt)
    }
  }
}

// ---------------------------------------------------------------------------
// Argument predicates
// ---------------------------------------------------------------------------

/// Trivially safe expression: a value that cannot trigger user code via any
/// coercion, getter, or iterator.
pub fn is_trivially_safe_expr(expr: &Expr, unresolved_ctxt: SyntaxContext) -> bool {
  match expr {
    Expr::Lit(_) => true,
    Expr::Tpl(t) => t.exprs.is_empty(),
    Expr::Unary(u) => {
      matches!(
        u.op,
        UnaryOp::Minus | UnaryOp::Plus | UnaryOp::Bang | UnaryOp::Tilde | UnaryOp::Void
      ) && is_trivially_safe_expr(&u.arg, unresolved_ctxt)
    }
    Expr::Ident(i) => {
      i.ctxt == unresolved_ctxt && matches!(i.sym.as_str(), "undefined" | "NaN" | "Infinity")
    }
    _ => false,
  }
}

/// Nullish: `null`, `undefined`, or `void <trivially-safe>`.
fn is_nullish_expr(expr: &Expr, unresolved_ctxt: SyntaxContext) -> bool {
  match expr {
    Expr::Lit(Lit::Null(_)) => true,
    Expr::Ident(i) => i.ctxt == unresolved_ctxt && i.sym.as_str() == "undefined",
    Expr::Unary(u) if u.op == UnaryOp::Void => is_trivially_safe_expr(&u.arg, unresolved_ctxt),
    _ => false,
  }
}

/// Non-negative integer literal that fits in `[0, 2^32)` — safe to pass as a
/// length to Array/TypedArray/ArrayBuffer constructors without `RangeError`.
///
/// Excludes BigInt literals (which throw `TypeError` for these constructors),
/// negative literals (`UnaryExpr Minus`), fractional values, NaN/Infinity.
fn is_non_negative_int_literal(expr: &Expr) -> bool {
  match expr {
    Expr::Lit(Lit::Num(n)) => {
      n.value >= 0.0
        && n.value.is_finite()
        && n.value.fract() == 0.0
        // `new Uint8Array(2^32)` throws RangeError; cap at u32 max.
        && n.value < 4_294_967_296.0
    }
    // `+5` is fine; `-5` is rejected because Minus is excluded.
    Expr::Unary(u) if u.op == UnaryOp::Plus => is_non_negative_int_literal(&u.arg),
    _ => false,
  }
}

/// All args trivially safe (and no spread).
pub fn are_args_trivially_safe(args: &[ExprOrSpread], unresolved_ctxt: SyntaxContext) -> bool {
  args
    .iter()
    .all(|a| a.spread.is_none() && is_trivially_safe_expr(&a.expr, unresolved_ctxt))
}

/// All args are nullish (or empty).
fn are_args_nullish_only(args: &[ExprOrSpread], unresolved_ctxt: SyntaxContext) -> bool {
  args
    .iter()
    .all(|a| a.spread.is_none() && is_nullish_expr(&a.expr, unresolved_ctxt))
}

/// All args are nullish or non-negative integer literals (or empty).
fn are_args_nullish_or_int_literal(args: &[ExprOrSpread], unresolved_ctxt: SyntaxContext) -> bool {
  args.iter().all(|a| {
    a.spread.is_none()
      && (is_nullish_expr(&a.expr, unresolved_ctxt) || is_non_negative_int_literal(&a.expr))
  })
}

// ---------------------------------------------------------------------------
// Internal classification tables
// ---------------------------------------------------------------------------

fn classify_ident(name: &str, position: CalleePosition) -> Option<ArgGate> {
  match position {
    CalleePosition::New => classify_new_ident(name),
    CalleePosition::Call => classify_call_ident(name),
  }
}

/// `new Name(…)`
fn classify_new_ident(name: &str) -> Option<ArgGate> {
  Some(match name {
    // `Boolean(x)` / `new Boolean(x)`: `ToBoolean` never throws and never
    // calls user code (no @@toPrimitive / valueOf / toString consultation).
    "Boolean" => ArgGate::AnyPureArgs,

    // `String(x)` / `new String(x)` / `Object(x)` / `new Object(x)`: safe
    // with literal arguments (incl BigInt — `String(1n)` returns "1",
    // `Object(1n)` wraps in a BigInt object).
    "String" | "Object" => ArgGate::TriviallySafeArgs,

    // `new Set(x)` and friends throw `TypeError` on non-iterable values
    // (e.g. `new Set(1)`). Only nullish or empty args are safe.
    "Set" | "Map" | "WeakSet" | "WeakMap" => ArgGate::NullishOrNoArgs,

    // `new Date(1n)` throws `TypeError` (BigInt → Number coercion fails).
    // `new Number(1n)` likewise. Be conservative: only nullish/no-arg.
    "Date" | "Number" => ArgGate::NullishOrNoArgs,

    // `new Array(-1)` / `new Array(1.5)` throw `RangeError`. With multiple
    // args the constructor stores them as elements without checking shape,
    // but the single-numeric-arg case is dangerous; `NullishOrNoArgs` is
    // the simplest sound rule.
    "Array" => ArgGate::NullishOrNoArgs,

    // TypedArrays / buffers: bad length (`-1`, `1.5`, `2^53`) throws
    // `RangeError`. Allow only nullish or non-negative integer literals.
    "ArrayBuffer" | "SharedArrayBuffer" | "Uint8Array" | "Int8Array" | "Uint8ClampedArray"
    | "Uint16Array" | "Int16Array" | "Uint32Array" | "Int32Array" | "Float32Array"
    | "Float64Array" | "BigInt64Array" | "BigUint64Array" => {
      ArgGate::NullishOrNonNegativeIntLiteral
    }

    _ => return None,
  })
}

/// `Name(…)`
fn classify_call_ident(name: &str) -> Option<ArgGate> {
  Some(match name {
    // See note in `classify_new_ident`: ToBoolean is fully safe.
    "Boolean" => ArgGate::AnyPureArgs,

    // ToString / Object wrap accept any literal incl BigInt.
    "String" | "Object" | "Symbol" => ArgGate::TriviallySafeArgs,

    // Same throwing behaviour as their `new` form.
    "Date" | "Number" => ArgGate::NullishOrNoArgs,
    "Array" => ArgGate::NullishOrNoArgs,

    _ => return None,
  })
}

/// `Obj.method(…)`
///
/// Only methods that are pure type/identity checks and never coerce, iterate,
/// or read properties.
///
/// Notably excluded:
/// * `Object.assign`/`freeze`/`create`/`fromEntries` — mutate or invoke user
///   code via getters/iterators.
/// * `Object.keys`/`values`/`entries` — Proxy `ownKeys`/`get` traps.
/// * `Array.from` — invokes iterator protocol and optional mapper fn.
fn classify_member(obj: &str, prop: &str) -> Option<ArgGate> {
  match obj {
    "Array" if matches!(prop, "isArray" | "of") => Some(ArgGate::AnyPureArgs),
    "Object" if prop == "is" => Some(ArgGate::AnyPureArgs),
    "Number" if matches!(prop, "isInteger" | "isFinite" | "isNaN" | "isSafeInteger") => {
      Some(ArgGate::AnyPureArgs)
    }
    _ => None,
  }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
  use super::*;

  fn ident_gate(name: &str, pos: CalleePosition) -> Option<ArgGate> {
    classify_ident(name, pos)
  }

  // -- collections (NullishOrNoArgs) --

  #[test]
  fn new_collections_require_nullish_args() {
    for name in ["Set", "Map", "WeakSet", "WeakMap"] {
      assert_eq!(
        ident_gate(name, CalleePosition::New),
        Some(ArgGate::NullishOrNoArgs),
        "{name}"
      );
    }
  }

  #[test]
  fn collections_as_function_call_are_not_pure() {
    // `Set()` (without `new`) throws TypeError. Not in the call list.
    for name in ["Set", "Map", "WeakSet", "WeakMap"] {
      assert_eq!(ident_gate(name, CalleePosition::Call), None, "{name}");
    }
  }

  // -- typed arrays / buffers --

  #[test]
  fn new_typed_arrays_require_nullish_or_int_literal() {
    for name in [
      "Uint8Array",
      "Int8Array",
      "Uint8ClampedArray",
      "Uint16Array",
      "Int16Array",
      "Uint32Array",
      "Int32Array",
      "Float32Array",
      "Float64Array",
      "BigInt64Array",
      "BigUint64Array",
      "ArrayBuffer",
      "SharedArrayBuffer",
    ] {
      assert_eq!(
        ident_gate(name, CalleePosition::New),
        Some(ArgGate::NullishOrNonNegativeIntLiteral),
        "{name}"
      );
    }
  }

  // -- primitive wrappers --

  #[test]
  fn new_string_object_are_trivially_safe() {
    for name in ["String", "Object"] {
      assert_eq!(
        ident_gate(name, CalleePosition::New),
        Some(ArgGate::TriviallySafeArgs),
        "{name}"
      );
    }
  }

  #[test]
  fn boolean_is_any_pure_args_in_both_positions() {
    assert_eq!(
      ident_gate("Boolean", CalleePosition::New),
      Some(ArgGate::AnyPureArgs)
    );
    assert_eq!(
      ident_gate("Boolean", CalleePosition::Call),
      Some(ArgGate::AnyPureArgs)
    );
  }

  #[test]
  fn number_and_date_require_nullish_only() {
    // Number(1n) and Date(1n) throw TypeError.
    for name in ["Number", "Date"] {
      assert_eq!(
        ident_gate(name, CalleePosition::New),
        Some(ArgGate::NullishOrNoArgs),
        "new {name}"
      );
      assert_eq!(
        ident_gate(name, CalleePosition::Call),
        Some(ArgGate::NullishOrNoArgs),
        "{name}()"
      );
    }
  }

  #[test]
  fn array_requires_nullish_only() {
    // `new Array(-1)` / `Array(0.5)` throw RangeError.
    assert_eq!(
      ident_gate("Array", CalleePosition::New),
      Some(ArgGate::NullishOrNoArgs)
    );
    assert_eq!(
      ident_gate("Array", CalleePosition::Call),
      Some(ArgGate::NullishOrNoArgs)
    );
  }

  #[test]
  fn new_symbol_is_not_pure() {
    // `new Symbol()` throws TypeError.
    assert_eq!(ident_gate("Symbol", CalleePosition::New), None);
  }

  #[test]
  fn call_symbol_is_trivially_safe() {
    assert_eq!(
      ident_gate("Symbol", CalleePosition::Call),
      Some(ArgGate::TriviallySafeArgs)
    );
  }

  #[test]
  fn unknown_globals_are_not_pure() {
    for name in ["Foo", "RegExp", "Promise", "Error", "BigInt"] {
      assert_eq!(ident_gate(name, CalleePosition::New), None, "new {name}");
      assert_eq!(ident_gate(name, CalleePosition::Call), None, "{name}()");
    }
  }

  // -- member calls --

  #[test]
  fn pure_member_calls() {
    assert_eq!(
      classify_member("Array", "isArray"),
      Some(ArgGate::AnyPureArgs)
    );
    assert_eq!(classify_member("Array", "of"), Some(ArgGate::AnyPureArgs));
    assert_eq!(classify_member("Object", "is"), Some(ArgGate::AnyPureArgs));
    for prop in ["isInteger", "isFinite", "isNaN", "isSafeInteger"] {
      assert_eq!(
        classify_member("Number", prop),
        Some(ArgGate::AnyPureArgs),
        "Number.{prop}"
      );
    }
  }

  #[test]
  fn array_from_is_not_pure() {
    assert_eq!(classify_member("Array", "from"), None);
  }

  #[test]
  fn object_mutating_or_iterating_methods_are_not_pure() {
    for method in [
      "assign",
      "freeze",
      "create",
      "fromEntries",
      "keys",
      "values",
      "entries",
      "getPrototypeOf",
    ] {
      assert_eq!(classify_member("Object", method), None, "Object.{method}");
    }
  }

  #[test]
  fn unknown_member_calls_are_not_pure() {
    assert_eq!(classify_member("Math", "random"), None);
    assert_eq!(classify_member("JSON", "parse"), None);
    assert_eq!(classify_member("Reflect", "apply"), None);
    assert_eq!(classify_member("Promise", "resolve"), None);
  }

  // -- argument predicates --

  // These tests construct synthetic Lit/Num exprs to exercise the
  // arg-shape predicates directly (no parser needed).

  use swc_core::{
    common::{DUMMY_SP, SyntaxContext as Ctxt},
    ecma::ast::{Bool, Null, Number, Str},
  };

  fn n(value: f64) -> Expr {
    Expr::Lit(Lit::Num(Number {
      span: DUMMY_SP,
      value,
      raw: None,
    }))
  }
  fn s(value: &str) -> Expr {
    Expr::Lit(Lit::Str(Str {
      span: DUMMY_SP,
      value: value.into(),
      raw: None,
    }))
  }
  fn null() -> Expr {
    Expr::Lit(Lit::Null(Null { span: DUMMY_SP }))
  }
  fn bool_lit(value: bool) -> Expr {
    Expr::Lit(Lit::Bool(Bool {
      span: DUMMY_SP,
      value,
    }))
  }

  fn args_of(es: Vec<Expr>) -> Vec<ExprOrSpread> {
    es.into_iter()
      .map(|expr| ExprOrSpread {
        spread: None,
        expr: Box::new(expr),
      })
      .collect()
  }

  #[test]
  fn empty_args_pass_every_gate() {
    let empty: Vec<ExprOrSpread> = vec![];
    let ctxt = Ctxt::empty();
    assert!(check_arg_gate(ArgGate::TriviallySafeArgs, &empty, ctxt));
    assert!(check_arg_gate(ArgGate::NullishOrNoArgs, &empty, ctxt));
    assert!(check_arg_gate(
      ArgGate::NullishOrNonNegativeIntLiteral,
      &empty,
      ctxt,
    ));
  }

  #[test]
  fn nullish_only_rejects_number_literal() {
    // `new Set(1)` throws TypeError — must NOT pass NullishOrNoArgs.
    let args = args_of(vec![n(1.0)]);
    assert!(!check_arg_gate(
      ArgGate::NullishOrNoArgs,
      &args,
      Ctxt::empty()
    ));
  }

  #[test]
  fn nullish_only_rejects_string_literal() {
    let args = args_of(vec![s("x")]);
    assert!(!check_arg_gate(
      ArgGate::NullishOrNoArgs,
      &args,
      Ctxt::empty()
    ));
  }

  #[test]
  fn nullish_only_accepts_null() {
    let args = args_of(vec![null()]);
    assert!(check_arg_gate(
      ArgGate::NullishOrNoArgs,
      &args,
      Ctxt::empty()
    ));
  }

  #[test]
  fn int_literal_accepts_zero_and_positive() {
    for v in [0.0, 1.0, 16.0, 1024.0, 4_294_967_295.0] {
      let args = args_of(vec![n(v)]);
      assert!(
        check_arg_gate(
          ArgGate::NullishOrNonNegativeIntLiteral,
          &args,
          Ctxt::empty()
        ),
        "value {v}"
      );
    }
  }

  #[test]
  fn int_literal_rejects_fractional() {
    // `new Uint8Array(1.5)` throws RangeError.
    let args = args_of(vec![n(1.5)]);
    assert!(!check_arg_gate(
      ArgGate::NullishOrNonNegativeIntLiteral,
      &args,
      Ctxt::empty()
    ));
  }

  #[test]
  fn int_literal_rejects_too_large() {
    // 2^32 is out of u32 range.
    let args = args_of(vec![n(4_294_967_296.0)]);
    assert!(!check_arg_gate(
      ArgGate::NullishOrNonNegativeIntLiteral,
      &args,
      Ctxt::empty()
    ));
  }

  #[test]
  fn int_literal_rejects_string() {
    let args = args_of(vec![s("16")]);
    assert!(!check_arg_gate(
      ArgGate::NullishOrNonNegativeIntLiteral,
      &args,
      Ctxt::empty()
    ));
  }

  #[test]
  fn trivially_safe_accepts_any_literal() {
    let ctxt = Ctxt::empty();
    let args = args_of(vec![n(1.5), s("x"), bool_lit(true), null()]);
    assert!(check_arg_gate(ArgGate::TriviallySafeArgs, &args, ctxt));
  }

  #[test]
  fn negative_unary_minus_is_not_int_literal() {
    use swc_core::ecma::ast::UnaryExpr;
    // `-1` is `UnaryExpr(Minus, Lit(1))` — must be rejected.
    let neg_one = Expr::Unary(UnaryExpr {
      span: DUMMY_SP,
      op: UnaryOp::Minus,
      arg: Box::new(n(1.0)),
    });
    let args = args_of(vec![neg_one]);
    assert!(!check_arg_gate(
      ArgGate::NullishOrNonNegativeIntLiteral,
      &args,
      Ctxt::empty()
    ));
  }

  #[test]
  fn positive_unary_plus_passes_through() {
    use swc_core::ecma::ast::UnaryExpr;
    let pos_five = Expr::Unary(UnaryExpr {
      span: DUMMY_SP,
      op: UnaryOp::Plus,
      arg: Box::new(n(5.0)),
    });
    let args = args_of(vec![pos_five]);
    assert!(check_arg_gate(
      ArgGate::NullishOrNonNegativeIntLiteral,
      &args,
      Ctxt::empty()
    ));
  }
}
