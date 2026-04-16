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
//!   interaction. `AnyPureArgs` means the callee never coerces/iterates its args
//!   (only nested expression side effects matter). `TriviallySafeArgs` means the
//!   callee *may* coerce/iterate, so every arg must be a literal or equivalent.

use swc_core::{
  common::SyntaxContext,
  ecma::ast::{Expr, ExprOrSpread, MemberProp, UnaryOp},
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgGate {
  /// The callee has no hidden dependency on argument *values* — it never
  /// coerces via `valueOf`/`toString`/`@@toPrimitive` and never invokes an
  /// iterator. As long as the argument *expressions* themselves are
  /// side-effect-free (evaluated recursively), the overall call is pure.
  ///
  /// Example: `Boolean(x)` — only checks truthiness.
  AnyPureArgs,

  /// The callee *may* coerce or iterate arguments, so each argument must be
  /// a *trivially safe* value (literal, `undefined`, `NaN`, `Infinity`, or
  /// unary on those).
  ///
  /// Example: `new Set(x)` — iterates `x` if it's an iterable.
  TriviallySafeArgs,
}

/// Classify `callee` as a known-pure global.
///
/// Returns `Some(gate)` when:
/// 1. The callee resolves to an unresolved global (not a local binding).
/// 2. The name + `position` combination is in the allowlist.
///
/// The caller is responsible for checking the arguments according to the
/// returned [`ArgGate`].
pub fn classify_pure_global(
  callee: &Expr,
  unresolved_ctxt: SyntaxContext,
  position: CalleePosition,
) -> Option<ArgGate> {
  match callee {
    // Simple identifier: `Set(…)` or `new Set(…)`
    Expr::Ident(ident) if ident.ctxt == unresolved_ctxt => {
      classify_ident(ident.sym.as_str(), position)
    }
    // Member expression: `Array.isArray(…)` or `Object.is(…)`
    // Only valid in Call position — `new Array.isArray(…)` is nonsensical.
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

/// Check whether an expression is *trivially safe* — a value that cannot
/// trigger user code via any coercion, getter, or iterator.
///
/// Used by the caller when [`ArgGate::TriviallySafeArgs`] is returned:
/// every argument to the "pure global" call must pass this check.
pub fn is_trivially_safe_expr(expr: &Expr, unresolved_ctxt: SyntaxContext) -> bool {
  match expr {
    // Number, string, boolean, null, regex, bigint literals.
    Expr::Lit(_) => true,
    // Template literal with no interpolations: `hello`.
    Expr::Tpl(t) => t.exprs.is_empty(),
    // Unary operator on a safe value: `-1`, `+0`, `!true`, `~0xff`, `void 0`.
    Expr::Unary(u) => {
      matches!(
        u.op,
        UnaryOp::Minus | UnaryOp::Plus | UnaryOp::Bang | UnaryOp::Tilde | UnaryOp::Void
      ) && is_trivially_safe_expr(&u.arg, unresolved_ctxt)
    }
    // Well-known global constants: `undefined`, `NaN`, `Infinity`.
    Expr::Ident(i) => {
      i.ctxt == unresolved_ctxt && matches!(i.sym.as_str(), "undefined" | "NaN" | "Infinity")
    }
    _ => false,
  }
}

/// All arguments are trivially safe (and no spread).
pub fn are_args_trivially_safe(args: &[ExprOrSpread], unresolved_ctxt: SyntaxContext) -> bool {
  args
    .iter()
    .all(|a| a.spread.is_none() && is_trivially_safe_expr(&a.expr, unresolved_ctxt))
}

// ---------------------------------------------------------------------------
// Internal classification tables
// ---------------------------------------------------------------------------

/// Classify a simple global identifier by name + position.
fn classify_ident(name: &str, position: CalleePosition) -> Option<ArgGate> {
  match position {
    CalleePosition::New => classify_new_ident(name),
    CalleePosition::Call => classify_call_ident(name),
  }
}

/// `new Name(…)` — constructors that are pure with trivially safe args.
fn classify_new_ident(name: &str) -> Option<ArgGate> {
  let is_known = matches!(
    name,
    // Collections (iterate their arg if iterable)
    "Set" | "Map" | "WeakSet" | "WeakMap"
    // Primitive wrappers (coerce via valueOf/toString)
    | "Object" | "Array" | "Date"
    | "String" | "Number" | "Boolean"
    // TypedArrays / buffers (coerce length via ToIndex)
    | "ArrayBuffer" | "SharedArrayBuffer"
    | "Uint8Array" | "Int8Array" | "Uint8ClampedArray"
    | "Uint16Array" | "Int16Array"
    | "Uint32Array" | "Int32Array"
    | "Float32Array" | "Float64Array"
    | "BigInt64Array" | "BigUint64Array"
  );
  is_known.then_some(ArgGate::TriviallySafeArgs)
}

/// `Name(…)` — direct calls.
fn classify_call_ident(name: &str) -> Option<ArgGate> {
  // Boolean(x) never coerces — just checks truthiness.
  if name == "Boolean" {
    return Some(ArgGate::AnyPureArgs);
  }
  // These may coerce args via valueOf/toString/@@toPrimitive.
  let is_known = matches!(
    name,
    "Array" | "Object" | "String" | "Number" | "Symbol" | "Date"
  );
  is_known.then_some(ArgGate::TriviallySafeArgs)
}

/// `Obj.method(…)` — member-expression calls.
///
/// Only methods that are pure type/identity checks and never read properties
/// (no Proxy traps), iterate (no `Symbol.iterator`), coerce
/// (no `valueOf`/`toString`), or mutate.
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

  // -- classify_pure_global with ident --

  fn ident_gate(name: &str, pos: CalleePosition) -> Option<ArgGate> {
    classify_ident(name, pos)
  }

  #[test]
  fn new_collections_are_pure_with_safe_args() {
    for name in ["Set", "Map", "WeakSet", "WeakMap"] {
      assert_eq!(
        ident_gate(name, CalleePosition::New),
        Some(ArgGate::TriviallySafeArgs),
        "{name}"
      );
    }
  }

  #[test]
  fn new_typed_arrays_are_pure_with_safe_args() {
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
        Some(ArgGate::TriviallySafeArgs),
        "{name}"
      );
    }
  }

  #[test]
  fn new_primitives_wrappers_are_pure_with_safe_args() {
    for name in ["Object", "Array", "Date", "String", "Number", "Boolean"] {
      assert_eq!(
        ident_gate(name, CalleePosition::New),
        Some(ArgGate::TriviallySafeArgs),
        "{name}"
      );
    }
  }

  #[test]
  fn new_symbol_is_not_pure() {
    // `new Symbol()` throws TypeError.
    assert_eq!(ident_gate("Symbol", CalleePosition::New), None);
  }

  #[test]
  fn new_unknown_is_not_pure() {
    assert_eq!(ident_gate("Foo", CalleePosition::New), None);
    assert_eq!(ident_gate("RegExp", CalleePosition::New), None);
    assert_eq!(ident_gate("Promise", CalleePosition::New), None);
    assert_eq!(ident_gate("Error", CalleePosition::New), None);
  }

  #[test]
  fn call_boolean_is_pure_with_any_args() {
    assert_eq!(
      ident_gate("Boolean", CalleePosition::Call),
      Some(ArgGate::AnyPureArgs)
    );
  }

  #[test]
  fn call_coercing_globals_are_pure_with_safe_args() {
    for name in ["Array", "Object", "String", "Number", "Symbol", "Date"] {
      assert_eq!(
        ident_gate(name, CalleePosition::Call),
        Some(ArgGate::TriviallySafeArgs),
        "{name}"
      );
    }
  }

  #[test]
  fn call_unknown_is_not_pure() {
    assert_eq!(ident_gate("Foo", CalleePosition::Call), None);
    assert_eq!(ident_gate("RegExp", CalleePosition::Call), None);
    assert_eq!(ident_gate("BigInt", CalleePosition::Call), None);
  }

  // -- classify_member --

  #[test]
  fn array_is_array_is_pure() {
    assert_eq!(
      classify_member("Array", "isArray"),
      Some(ArgGate::AnyPureArgs)
    );
  }

  #[test]
  fn array_of_is_pure() {
    assert_eq!(classify_member("Array", "of"), Some(ArgGate::AnyPureArgs));
  }

  #[test]
  fn array_from_is_not_pure() {
    assert_eq!(classify_member("Array", "from"), None);
  }

  #[test]
  fn object_is_is_pure() {
    assert_eq!(classify_member("Object", "is"), Some(ArgGate::AnyPureArgs));
  }

  #[test]
  fn object_mutating_is_not_pure() {
    for method in [
      "assign",
      "freeze",
      "create",
      "fromEntries",
      "keys",
      "values",
      "entries",
    ] {
      assert_eq!(classify_member("Object", method), None, "Object.{method}");
    }
  }

  #[test]
  fn number_type_checks_are_pure() {
    for method in ["isInteger", "isFinite", "isNaN", "isSafeInteger"] {
      assert_eq!(
        classify_member("Number", method),
        Some(ArgGate::AnyPureArgs),
        "Number.{method}"
      );
    }
  }

  #[test]
  fn unknown_member_is_not_pure() {
    assert_eq!(classify_member("Math", "random"), None);
    assert_eq!(classify_member("JSON", "parse"), None);
    assert_eq!(classify_member("Reflect", "apply"), None);
  }

  // -- is_trivially_safe_expr (lightweight checks on synthetic exprs) --

  #[test]
  fn empty_args_are_trivially_safe() {
    let empty: &[ExprOrSpread] = &[];
    assert!(are_args_trivially_safe(empty, SyntaxContext::empty()));
  }
}
