use rspack_util::swc::AstSubRangeExt;
use swc_next_ecma_ast::{
  Argument, ArgumentData, Expr, ExprData, NewExpression, PropertyKeyData, SubRange,
};

use crate::visitors::JavascriptParser;

pub(super) fn pure_global_call_arguments(
  parser: &JavascriptParser,
  expression: Expr,
) -> Option<SubRange<Argument>> {
  let ast = parser.ast.ast;
  let call = match ast.expr_data(expression) {
    ExprData::NewExpression(new_expression) => {
      return is_pure_global_new(parser, new_expression).then(|| new_expression.arguments(ast));
    }
    ExprData::CallExpression(call) => call,
    ExprData::ChainExpression(chain) => match ast.expr_data(chain.expression(ast)) {
      ExprData::CallExpression(call) => call,
      _ => return None,
    },
    _ => return None,
  };
  is_pure_global_callee(parser, call.callee(ast)).then(|| call.arguments(ast))
}

fn is_pure_global_callee(parser: &JavascriptParser, callee: Expr) -> bool {
  if let Some(name) = global_name(parser, callee) {
    return is_pure_global_function(name);
  }
  let ast = parser.ast.ast;
  let ExprData::MemberExpression(member) = ast.expr_data(callee) else {
    return false;
  };
  if member.computed(ast) {
    return false;
  }
  let PropertyKeyData::IdentifierName(property) = ast.property_key_data(member.property(ast))
  else {
    return false;
  };
  global_name(parser, member.object(ast))
    .is_some_and(|object| is_pure_global_method(object, ast.get_utf8(property.name(ast))))
}

fn is_pure_global_new(parser: &JavascriptParser, new_expression: NewExpression) -> bool {
  let ast = parser.ast.ast;
  let Some(name) = global_name(parser, new_expression.callee(ast)) else {
    return false;
  };
  if is_pure_global_constructor(name) {
    return true;
  }

  let first = match ast.nodes(new_expression.arguments(ast)).next() {
    None => None,
    Some(argument) => match ast.argument_data(argument) {
      ArgumentData::SpreadElement(_) => return false,
      ArgumentData::Expr(expression) => Some(ast.expr_data(expression)),
    },
  };
  match name {
    "Set" | "WeakSet" => first.is_none_or(|first| matches!(first, ExprData::ArrayExpression(_))),
    "Map" | "WeakMap" => first.is_none_or(|first| match first {
      ExprData::ArrayExpression(array) => ast.optional_nodes(array.elements(ast)).all(|element| {
        element.is_some_and(|element| match ast.argument_data(element) {
          ArgumentData::Expr(entry) => {
            matches!(ast.expr_data(entry), ExprData::ArrayExpression(_))
          }
          ArgumentData::SpreadElement(_) => false,
        })
      }),
      _ => false,
    }),
    "URLSearchParams" => first.is_none_or(|first| matches!(first, ExprData::StringLiteral(_))),
    _ if is_typed_array_constructor(name) => first.is_none_or(|first| {
      matches!(
        first,
        ExprData::NumericLiteral(_) | ExprData::ArrayExpression(_)
      )
    }),
    _ => false,
  }
}

fn global_name<'a>(parser: &'a JavascriptParser, expression: Expr) -> Option<&'a str> {
  if parser.active_synthetic_ast.is_some() {
    return None;
  }
  let ast = parser.ast.ast;
  let identifier = expression.as_identifier_reference(ast)?;
  let semantic = parser.ast.semantic;
  let reference = semantic.reference_of(identifier.node_id())?;
  semantic
    .reference(reference)
    .symbol
    .is_none()
    .then(|| ast.get_utf8(identifier.name(ast)))
}

// Calls and `new` expressions of these globals count as side-effect-free even
// when they can throw on invalid input, such as `new WeakSet([1])` or
// `decodeURIComponent("%")`
fn is_pure_global_function(name: &str) -> bool {
  matches!(
    name,
    "Boolean"
      | "Date"
      | "Number"
      | "Object"
      | "String"
      | "Symbol"
      | "decodeURI"
      | "decodeURIComponent"
      | "encodeURI"
      | "encodeURIComponent"
      | "isFinite"
      | "isNaN"
      | "parseFloat"
      | "parseInt"
  )
}

fn is_pure_global_method(object: &str, method: &str) -> bool {
  match object {
    "Array" => matches!(method, "isArray" | "of"),
    "Math" => true,
    "Number" => matches!(
      method,
      "isFinite" | "isInteger" | "isNaN" | "isSafeInteger" | "parseFloat" | "parseInt"
    ),
    "Object" => matches!(
      method,
      "entries"
        | "getOwnPropertyDescriptor"
        | "getOwnPropertyDescriptors"
        | "getOwnPropertyNames"
        | "getOwnPropertySymbols"
        | "getPrototypeOf"
        | "hasOwn"
        | "is"
        | "isExtensible"
        | "isFrozen"
        | "isSealed"
        | "keys"
        | "values"
    ),
    "String" => matches!(method, "fromCharCode" | "fromCodePoint"),
    "Symbol" => matches!(method, "for" | "keyFor"),
    _ => false,
  }
}

fn is_pure_global_constructor(name: &str) -> bool {
  matches!(
    name,
    "Array"
      | "ArrayBuffer"
      | "Boolean"
      | "DataView"
      | "Date"
      | "Error"
      | "EvalError"
      | "Number"
      | "Object"
      | "RangeError"
      | "ReferenceError"
      | "RegExp"
      | "String"
      | "SyntaxError"
      | "TypeError"
      | "URIError"
  )
}

fn is_typed_array_constructor(name: &str) -> bool {
  matches!(
    name,
    "BigInt64Array"
      | "BigUint64Array"
      | "Float32Array"
      | "Float64Array"
      | "Int8Array"
      | "Int16Array"
      | "Int32Array"
      | "Uint8Array"
      | "Uint8ClampedArray"
      | "Uint16Array"
      | "Uint32Array"
  )
}
