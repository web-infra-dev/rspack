//! Terser-compatible known-pure globals for side-effects analysis.
//!
//! This models Terser's `compress.unsafe` native-object tables for unresolved
//! global symbols. The option is intentionally opt-in because these tables are
//! a compatibility heuristic, not a proof that the runtime operation cannot
//! throw.
//!
//! ## Safety invariants
//!
//! * **Shadowing**: the callee identifier must not resolve to a module-local
//!   binding, so a local `const Boolean = …` is never mistaken for the built-in.
//! * **Arguments**: argument expressions are still checked separately for side
//!   effects by the caller.

use swc_experimental_ecma_ast::{Expr, Ident, Lit, MemberExpr, MemberProp};

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

/// Classify `callee` as a known-pure global.
///
/// Returns true when:
/// 1. The callee resolves to an unresolved global (not a local binding).
/// 2. The name + `position` combination is in the allowlist.
pub fn classify_pure_global(
  callee: &Expr<'_>,
  is_unresolved_ident: &mut impl FnMut(&Ident<'_>) -> bool,
  position: CalleePosition,
) -> bool {
  match callee {
    Expr::Ident(ident) if is_unresolved_ident(ident) => {
      classify_ident(ident.sym.as_str(), position)
    }
    Expr::Member(member) => {
      let Some(callee) = parse_member_callee(member, is_unresolved_ident) else {
        return false;
      };
      match callee {
        PureGlobalCallee::Static { obj, prop } => classify_static_fn(obj, prop),
      }
    }
    _ => false,
  }
}

/// Classify direct global symbol/property reads that Terser treats as safe
/// access under `unsafe`.
pub fn is_pure_global_access(
  expr: &Expr<'_>,
  is_unresolved_ident: &mut impl FnMut(&Ident<'_>) -> bool,
) -> bool {
  match expr {
    Expr::Ident(ident) => {
      is_unresolved_ident(ident) && is_pure_bare_access_global(ident.sym.as_str())
    }
    Expr::Member(member) => {
      is_static_property(&member.prop)
        && member.obj.as_ident().is_some_and(|obj| {
          is_unresolved_ident(obj) && is_pure_member_access_global(obj.sym.as_str())
        })
    }
    _ => false,
  }
}

// ---------------------------------------------------------------------------
// Internal classification tables
// ---------------------------------------------------------------------------

fn classify_ident(name: &str, position: CalleePosition) -> bool {
  match position {
    CalleePosition::New => false,
    CalleePosition::Call => classify_call_ident(name),
  }
}

/// `Name(…)`
fn classify_call_ident(name: &str) -> bool {
  matches!(
    name,
    "Boolean"
      | "Date"
      | "Error"
      | "EvalError"
      | "Number"
      | "Object"
      | "RangeError"
      | "ReferenceError"
      | "String"
      | "SyntaxError"
      | "TypeError"
      | "URIError"
      | "decodeURI"
      | "decodeURIComponent"
      | "encodeURI"
      | "encodeURIComponent"
      | "escape"
      | "isFinite"
      | "isNaN"
      | "parseFloat"
      | "parseInt"
      | "unescape"
  )
}

/// `Obj.method(…)`.
fn classify_static_fn(obj: &str, prop: &str) -> bool {
  match obj {
    "Array" => prop == "isArray",
    "Math" => matches!(
      prop,
      "abs"
        | "acos"
        | "asin"
        | "atan"
        | "atan2"
        | "ceil"
        | "cos"
        | "exp"
        | "floor"
        | "log"
        | "max"
        | "min"
        | "pow"
        | "round"
        | "sin"
        | "sqrt"
        | "tan"
    ),
    "Number" => matches!(prop, "isFinite" | "isNaN"),
    "Object" => matches!(
      prop,
      "create"
        | "getOwnPropertyDescriptor"
        | "getOwnPropertyNames"
        | "getPrototypeOf"
        | "hasOwn"
        | "isExtensible"
        | "isFrozen"
        | "isSealed"
        | "keys"
    ),
    "String" => prop == "fromCharCode",
    _ => false,
  }
}

enum PureGlobalCallee<'a> {
  Static { obj: &'a str, prop: &'a str },
}

fn parse_member_callee<'a>(
  member: &'a MemberExpr<'a>,
  is_unresolved_ident: &mut impl FnMut(&Ident<'_>) -> bool,
) -> Option<PureGlobalCallee<'a>> {
  let prop = static_member_name(&member.prop)?;
  match &member.obj {
    Expr::Ident(obj) if is_unresolved_ident(obj) => Some(PureGlobalCallee::Static {
      obj: obj.sym.as_str(),
      prop,
    }),
    _ => None,
  }
}

fn static_member_name<'a>(prop: &'a MemberProp<'a>) -> Option<&'a str> {
  match prop {
    MemberProp::Ident(ident) => Some(ident.sym.as_str()),
    MemberProp::Computed(computed) => match computed.expr.as_lit()? {
      Lit::Str(s) => s.value.as_str(),
      _ => None,
    },
    _ => None,
  }
}

fn is_static_property(prop: &MemberProp<'_>) -> bool {
  match prop {
    MemberProp::Ident(_) => true,
    MemberProp::Computed(computed) => computed
      .expr
      .as_lit()
      .is_some_and(|lit| matches!(lit, Lit::Str(_) | Lit::Num(_) | Lit::Bool(_) | Lit::Null(_))),
    MemberProp::PrivateName(_) => false,
  }
}

fn is_pure_bare_access_global(name: &str) -> bool {
  name == "Promise" || is_pure_member_access_global(name)
}

fn is_pure_member_access_global(name: &str) -> bool {
  matches!(
    name,
    "Array"
      | "Boolean"
      | "Date"
      | "Error"
      | "EvalError"
      | "Function"
      | "JSON"
      | "Math"
      | "Number"
      | "Object"
      | "RangeError"
      | "ReferenceError"
      | "RegExp"
      | "String"
      | "SyntaxError"
      | "TypeError"
      | "URIError"
      | "clearInterval"
      | "clearTimeout"
      | "console"
      | "decodeURI"
      | "decodeURIComponent"
      | "encodeURI"
      | "encodeURIComponent"
      | "escape"
      | "eval"
      | "isFinite"
      | "isNaN"
      | "parseFloat"
      | "parseInt"
      | "setInterval"
      | "setTimeout"
      | "unescape"
  )
}
