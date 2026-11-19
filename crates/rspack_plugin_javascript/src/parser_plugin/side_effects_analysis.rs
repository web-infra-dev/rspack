/**
 * Some code is modified based on
 * https://github.com/CPunisher/swc-experimental/blob/57cf78c4bc1b963d4e4946492b88164958e6f764/crates/swc_ecma_utils/src/lib.rs
 * Apache-2.0 licensed
 */
// Conservative, syntax-level side-effects analysis.
//
// Rspack-specific policies such as `/*#__PURE__*/`, `pureFunctions`, parser hooks, and deferred
// import checks stay in `side_effects_parser_plugin`.
use rspack_util::swc::{AstSubRangeExt, PropertyKeyValue, classify_property_key};
use swc_next_ecma_ast::{
  ArgumentData, ArrowFunctionBodyData, Ast, BindingPattern, BindingPatternData, Class,
  ClassElementData, DeclData, Expr, ExprData, FormalParameterItemData, FormalParameters, Function,
  MethodDefinitionKind, ObjectPropertyKindData, PropertyKey, PropertyKeyData, PropertyKind, Stmt,
  StmtData, UnaryOperator, VariableKind,
};
use swc_next_ecma_semantic::Semantic;

use crate::visitors::formal_parameters_are_simple_identifiers;

#[derive(Clone, Copy)]
pub(super) struct SideEffectsContext<'a, 'ast> {
  ast: &'a Ast<'ast>,
  semantic: &'a Semantic<'ast>,
  is_unresolved_ref_safe: bool,
  is_synthetic_ast: bool,
  in_strict: bool,
  remaining_depth: u8,
}

impl<'a, 'ast> SideEffectsContext<'a, 'ast> {
  pub(super) fn new(
    ast: &'a Ast<'ast>,
    semantic: &'a Semantic<'ast>,
    is_unresolved_ref_safe: bool,
    is_synthetic_ast: bool,
  ) -> Self {
    Self {
      ast,
      semantic,
      is_unresolved_ref_safe,
      is_synthetic_ast,
      in_strict: false,
      remaining_depth: 4,
    }
  }

  fn consume_depth(self) -> Option<Self> {
    if self.remaining_depth == 0 {
      return None;
    }

    Some(Self {
      remaining_depth: self.remaining_depth - 1,
      ..self
    })
  }
}

pub(super) fn may_have_side_effects(expression: Expr, ctx: SideEffectsContext<'_, '_>) -> bool {
  let Some(ctx) = ctx.consume_depth() else {
    return true;
  };

  if is_pure_callee(expression, ctx)
    || global_member_name(expression, ctx).is_some_and(|(object, _)| object == "Math")
  {
    return false;
  }

  let ast = ctx.ast;
  match ast.expr_data(expression) {
    ExprData::IdentifierReference(identifier) => {
      if ctx.is_unresolved_ref_safe {
        return false;
      }

      if is_unresolved_reference(identifier, ctx) {
        !matches!(
          ast.get_utf8(identifier.name(ast)),
          "Infinity"
            | "NaN"
            | "Date"
            | "Math"
            | "undefined"
            | "Object"
            | "Array"
            | "Promise"
            | "Boolean"
            | "Number"
            | "String"
            | "BigInt"
            | "Error"
            | "RegExp"
            | "Function"
            | "document"
        )
      } else {
        false
      }
    }
    ExprData::StringLiteral(_)
    | ExprData::NumericLiteral(_)
    | ExprData::BigIntLiteral(_)
    | ExprData::BooleanLiteral(_)
    | ExprData::NullLiteral(_)
    | ExprData::RegExpLiteral(_)
    | ExprData::ThisExpression(_)
    | ExprData::PrivateIdentifier(_) => false,
    ExprData::ParenthesizedExpression(parenthesized) => {
      may_have_side_effects(parenthesized.expression(ast), ctx)
    }
    ExprData::TsAsExpression(expression) => may_have_side_effects(expression.expression(ast), ctx),
    ExprData::TsSatisfiesExpression(expression) => {
      may_have_side_effects(expression.expression(ast), ctx)
    }
    ExprData::TsTypeAssertion(expression) => may_have_side_effects(expression.expression(ast), ctx),
    ExprData::TsNonNullExpression(expression) => {
      may_have_side_effects(expression.expression(ast), ctx)
    }
    ExprData::TsInstantiationExpression(expression) => {
      may_have_side_effects(expression.expression(ast), ctx)
    }
    ExprData::Function(_) | ExprData::ArrowFunctionExpression(_) => false,
    ExprData::Class(class) => class_has_side_effects(ctx, class),
    ExprData::ArrayExpression(array) => {
      ast
        .optional_nodes(array.elements(ast))
        .flatten()
        .any(|argument| match ast.argument_data(argument) {
          ArgumentData::Expr(expression) => may_have_side_effects(expression, ctx),
          ArgumentData::SpreadElement(_) => true,
        })
    }
    ExprData::UnaryExpression(unary) => {
      unary.operator(ast) == UnaryOperator::Delete
        || may_have_side_effects(unary.argument(ast), ctx)
    }
    ExprData::BinaryExpression(binary) => {
      may_have_side_effects(binary.left(ast), ctx) || may_have_side_effects(binary.right(ast), ctx)
    }
    ExprData::LogicalExpression(logical) => {
      may_have_side_effects(logical.left(ast), ctx)
        || may_have_side_effects(logical.right(ast), ctx)
    }
    ExprData::MemberExpression(member)
      if matches!(
        ast.expr_data(member.object(ast)),
        ExprData::ObjectExpression(_)
          | ExprData::Function(_)
          | ExprData::ArrowFunctionExpression(_)
          | ExprData::Class(_)
      ) =>
    {
      let object = member.object(ast);
      if may_have_side_effects(object, ctx) {
        return true;
      }

      match ast.expr_data(object) {
        ExprData::Class(class) if class_member_access_may_have_side_effects(ctx, class) => {
          return true;
        }
        ExprData::ObjectExpression(object)
          if object_member_access_may_have_side_effects(ctx, object) =>
        {
          return true;
        }
        _ => {}
      }

      member.computed(ast) && property_key_may_have_side_effects(member.property(ast), ctx)
    }
    ExprData::TemplateLiteral(_)
    | ExprData::TaggedTemplateExpression(_)
    | ExprData::MetaProperty(_) => true,
    ExprData::AwaitExpression(_)
    | ExprData::YieldExpression(_)
    | ExprData::MemberExpression(_)
    | ExprData::Super(_)
    | ExprData::UpdateExpression(_)
    | ExprData::AssignmentExpression(_)
    | ExprData::ImportExpression(_) => true,
    ExprData::ChainExpression(chain) => match ast.expr_data(chain.expression(ast)) {
      ExprData::CallExpression(call) if is_pure_callee(call.callee(ast), ctx) => {
        pure_call_may_have_side_effects(call, ctx)
      }
      _ => true,
    },
    ExprData::NewExpression(new_expression)
      if is_pure_new_callee(new_expression.callee(ast), ctx)
        || is_pure_global_new(new_expression, ctx) =>
    {
      arguments_may_have_side_effects(new_expression.arguments(ast), ctx)
    }
    ExprData::NewExpression(_) => true,
    ExprData::CallExpression(call) if is_pure_callee(call.callee(ast), ctx) => {
      pure_call_may_have_side_effects(call, ctx)
    }
    ExprData::CallExpression(_) => true,
    ExprData::SequenceExpression(sequence) => ast
      .nodes(sequence.expressions(ast))
      .any(|node| may_have_side_effects(node, ctx)),
    ExprData::ConditionalExpression(conditional) => {
      may_have_side_effects(conditional.test(ast), ctx)
        || may_have_side_effects(conditional.consequent(ast), ctx)
        || may_have_side_effects(conditional.alternate(ast), ctx)
    }
    ExprData::ObjectExpression(object) => {
      ast.nodes(object.properties(ast)).any(|property| {
        match ast.object_property_kind_data(property) {
          ObjectPropertyKindData::SpreadElement(_) => true,
          ObjectPropertyKindData::ObjectProperty(property) => {
            if property.shorthand(ast) {
              return false;
            }

            let key_has_side_effects =
              property.computed(ast) && property_key_may_have_side_effects(property.key(ast), ctx);
            if property.kind(ast) == PropertyKind::Init && !property.method(ast) {
              key_has_side_effects || may_have_side_effects(property.value(ast), ctx)
            } else {
              key_has_side_effects
            }
          }
        }
      })
    }
    ExprData::JsxElement(_) | ExprData::JsxFragment(_) => true,
  }
}

fn is_pure_callee(expression: Expr, ctx: SideEffectsContext<'_, '_>) -> bool {
  if is_pure_global_callee(expression, ctx) {
    return true;
  }

  let ast = ctx.ast;
  match ast.expr_data(expression) {
    ExprData::MemberExpression(member) if !member.computed(ast) => {
      let PropertyKeyData::IdentifierName(property) = ast.property_key_data(member.property(ast))
      else {
        return false;
      };
      let property = ast.get_utf8(property.name(ast));

      match ast.expr_data(member.object(ast)) {
        ExprData::StringLiteral(_) => is_pure_string_method(property),
        ExprData::TemplateLiteral(template) if template.expressions(ast).is_empty() => {
          is_pure_string_method(property)
        }
        _ => false,
      }
    }
    ExprData::Function(function) => is_empty_function(ast, function),
    _ => false,
  }
}

fn pure_call_may_have_side_effects(
  call: swc_next_ecma_ast::CallExpression,
  ctx: SideEffectsContext<'_, '_>,
) -> bool {
  let ast = ctx.ast;
  arguments_may_have_side_effects(call.arguments(ast), ctx)
    || invoked_arguments_may_have_side_effects(call.arguments(ast), ctx)
}

pub(super) fn is_pure_global_callee(expression: Expr, ctx: SideEffectsContext<'_, '_>) -> bool {
  if let Some(name) = global_reference_name(expression, ctx) {
    return is_pure_global_function(name);
  }

  global_member_name(expression, ctx)
    .is_some_and(|(object, method)| is_pure_global_method(object, method))
}

pub(super) fn is_pure_global_new(
  new_expression: swc_next_ecma_ast::NewExpression,
  ctx: SideEffectsContext<'_, '_>,
) -> bool {
  let ast = ctx.ast;
  let Some(name) = global_reference_name(new_expression.callee(ast), ctx) else {
    return false;
  };
  let arguments = new_expression.arguments(ast);
  if invoked_arguments_may_have_side_effects(arguments, ctx) {
    return false;
  }
  if is_pure_global_constructor(name) {
    return true;
  }

  let first = match ast.nodes(arguments).next() {
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

pub(super) fn invoked_arguments_may_have_side_effects(
  arguments: swc_next_ecma_ast::SubRange<swc_next_ecma_ast::Argument>,
  ctx: SideEffectsContext<'_, '_>,
) -> bool {
  ctx
    .ast
    .nodes(arguments)
    .any(|argument| match ctx.ast.argument_data(argument) {
      ArgumentData::Expr(expression) => invoked_value_may_have_side_effects(expression, ctx),
      ArgumentData::SpreadElement(_) => true,
    })
}

fn invoked_value_may_have_side_effects(expression: Expr, ctx: SideEffectsContext<'_, '_>) -> bool {
  let ast = ctx.ast;
  match ast.expr_data(expression) {
    ExprData::Function(function) => {
      parameters_may_have_side_effects(function.params(ast), ctx)
        || ast
          .nodes(function.body(ast).body(ast))
          .any(|statement| stmt_may_have_side_effects(statement, ctx))
    }
    ExprData::ArrowFunctionExpression(arrow) => {
      parameters_may_have_side_effects(arrow.params(ast), ctx)
        || match ast.arrow_function_body_data(arrow.body(ast)) {
          ArrowFunctionBodyData::FunctionBody(body) => ast
            .nodes(body.body(ast))
            .any(|statement| stmt_may_have_side_effects(statement, ctx)),
          ArrowFunctionBodyData::Expr(expression) => may_have_side_effects(expression, ctx),
        }
    }
    ExprData::ParenthesizedExpression(parenthesized) => {
      invoked_value_may_have_side_effects(parenthesized.expression(ast), ctx)
    }
    ExprData::TsAsExpression(expression) => {
      invoked_value_may_have_side_effects(expression.expression(ast), ctx)
    }
    ExprData::TsSatisfiesExpression(expression) => {
      invoked_value_may_have_side_effects(expression.expression(ast), ctx)
    }
    ExprData::TsTypeAssertion(expression) => {
      invoked_value_may_have_side_effects(expression.expression(ast), ctx)
    }
    ExprData::TsNonNullExpression(expression) => {
      invoked_value_may_have_side_effects(expression.expression(ast), ctx)
    }
    ExprData::TsInstantiationExpression(expression) => {
      invoked_value_may_have_side_effects(expression.expression(ast), ctx)
    }
    ExprData::ConditionalExpression(conditional) => {
      invoked_value_may_have_side_effects(conditional.consequent(ast), ctx)
        || invoked_value_may_have_side_effects(conditional.alternate(ast), ctx)
    }
    ExprData::LogicalExpression(logical) => {
      invoked_value_may_have_side_effects(logical.left(ast), ctx)
        || invoked_value_may_have_side_effects(logical.right(ast), ctx)
    }
    ExprData::SequenceExpression(sequence) => ast
      .nodes(sequence.expressions(ast))
      .any(|expression| invoked_value_may_have_side_effects(expression, ctx)),
    ExprData::ArrayExpression(array) => {
      ast
        .optional_nodes(array.elements(ast))
        .flatten()
        .any(|element| match ast.argument_data(element) {
          ArgumentData::Expr(expression) => invoked_value_may_have_side_effects(expression, ctx),
          ArgumentData::SpreadElement(_) => true,
        })
    }
    ExprData::ObjectExpression(object) => {
      ast.nodes(object.properties(ast)).any(|property| {
        match ast.object_property_kind_data(property) {
          ObjectPropertyKindData::SpreadElement(_) => true,
          ObjectPropertyKindData::ObjectProperty(property) => {
            property.kind(ast) == PropertyKind::Init
              && !property.method(ast)
              && invoked_value_may_have_side_effects(property.value(ast), ctx)
          }
        }
      })
    }
    _ => false,
  }
}

fn parameters_may_have_side_effects(
  parameters: FormalParameters,
  ctx: SideEffectsContext<'_, '_>,
) -> bool {
  let ast = ctx.ast;
  ast
    .nodes(parameters.items(ast))
    .any(|item| match ast.formal_parameter_item_data(item) {
      FormalParameterItemData::FormalParameter(parameter) => {
        parameter
          .initializer(ast)
          .is_some_and(|initializer| may_have_side_effects(initializer, ctx))
          || binding_pattern_may_have_side_effects(parameter.pattern(ast).into(), ctx)
      }
      FormalParameterItemData::TsParameterProperty(_) => true,
    })
    || parameters
      .rest(ast)
      .is_some_and(|rest| binding_pattern_may_have_side_effects(rest.argument(ast).into(), ctx))
}

fn binding_pattern_may_have_side_effects(
  pattern: BindingPattern,
  ctx: SideEffectsContext<'_, '_>,
) -> bool {
  let ast = ctx.ast;
  match ast.binding_pattern_data(pattern) {
    BindingPatternData::BindingIdentifier(_) => false,
    BindingPatternData::AssignmentPattern(assignment) => {
      may_have_side_effects(assignment.right(ast), ctx)
        || binding_pattern_may_have_side_effects(assignment.left(ast), ctx)
    }
    BindingPatternData::BindingRestElement(rest) => {
      binding_pattern_may_have_side_effects(rest.argument(ast), ctx)
    }
    BindingPatternData::ArrayPattern(array) => {
      ast
        .optional_nodes(array.elements(ast))
        .flatten()
        .any(|element| binding_pattern_may_have_side_effects(element, ctx))
        || array
          .rest(ast)
          .is_some_and(|rest| binding_pattern_may_have_side_effects(rest.argument(ast), ctx))
    }
    BindingPatternData::ObjectPattern(object) => {
      ast.nodes(object.properties(ast)).any(|property| {
        (property.computed(ast) && property_key_may_have_side_effects(property.key(ast), ctx))
          || binding_pattern_may_have_side_effects(property.value(ast), ctx)
      }) || object
        .rest(ast)
        .is_some_and(|rest| binding_pattern_may_have_side_effects(rest.argument(ast), ctx))
    }
    BindingPatternData::SimpleAssignmentTarget(_) => true,
  }
}

pub(super) fn stmt_may_have_side_effects(statement: Stmt, ctx: SideEffectsContext<'_, '_>) -> bool {
  let ast = ctx.ast;
  match ast.stmt_data(statement) {
    StmtData::EmptyStatement(_) => false,
    StmtData::ExpressionStatement(expression) => {
      may_have_side_effects(expression.expression(ast), ctx)
    }
    StmtData::ReturnStatement(return_statement) => return_statement
      .argument(ast)
      .is_some_and(|argument| may_have_side_effects(argument, ctx)),
    StmtData::Declaration(declaration) => match ast.decl_data(declaration) {
      DeclData::VariableDeclaration(variable) => {
        if matches!(
          variable.kind(ast),
          VariableKind::Using | VariableKind::AwaitUsing
        ) {
          return true;
        }
        ast.nodes(variable.declarators(ast)).any(|declarator| {
          !matches!(
            ast.binding_pattern_data(declarator.id(ast)),
            BindingPatternData::BindingIdentifier(_)
          ) || declarator
            .init(ast)
            .is_some_and(|initializer| may_have_side_effects(initializer, ctx))
        })
      }
      _ => true,
    },
    _ => true,
  }
}

// Calls and `new` expressions of these globals count as side-effect-free even
// when they can throw on invalid input, such as `new WeakSet([1])` or
// `decodeURIComponent("%")`
fn is_pure_global_function(name: &str) -> bool {
  matches!(
    name,
    "Boolean"
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
    "Array" => matches!(method, "from" | "isArray" | "of"),
    "Math" => matches!(
      method,
      "abs"
        | "acos"
        | "acosh"
        | "asin"
        | "asinh"
        | "atan"
        | "atan2"
        | "atanh"
        | "cbrt"
        | "ceil"
        | "clz32"
        | "cos"
        | "cosh"
        | "exp"
        | "expm1"
        | "floor"
        | "fround"
        | "hypot"
        | "imul"
        | "log"
        | "log10"
        | "log1p"
        | "log2"
        | "max"
        | "min"
        | "pow"
        | "round"
        | "sign"
        | "sin"
        | "sinh"
        | "sqrt"
        | "tan"
        | "tanh"
        | "trunc"
    ),
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
      | "Promise"
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

fn global_reference_name<'a>(expression: Expr, ctx: SideEffectsContext<'a, '_>) -> Option<&'a str> {
  if ctx.is_synthetic_ast {
    return None;
  }
  let ast = ctx.ast;
  let ExprData::IdentifierReference(identifier) = ast.expr_data(expression) else {
    return None;
  };
  is_unresolved_reference(identifier, ctx).then(|| ast.get_utf8(identifier.name(ast)))
}

fn global_member_name<'a>(
  expression: Expr,
  ctx: SideEffectsContext<'a, '_>,
) -> Option<(&'a str, &'a str)> {
  let ast = ctx.ast;
  let ExprData::MemberExpression(member) = ast.expr_data(expression) else {
    return None;
  };
  if member.computed(ast) {
    return None;
  }
  let PropertyKeyData::IdentifierName(property) = ast.property_key_data(member.property(ast))
  else {
    return None;
  };
  let object = global_reference_name(member.object(ast), ctx)?;
  Some((object, ast.get_utf8(property.name(ast))))
}

fn is_unresolved_reference(
  identifier: swc_next_ecma_ast::IdentifierReference,
  ctx: SideEffectsContext<'_, '_>,
) -> bool {
  ctx
    .semantic
    .reference_of(identifier.node_id())
    .map(|reference| ctx.semantic.reference(reference))
    .is_some_and(|reference| reference.symbol.is_none())
}

fn arguments_may_have_side_effects(
  arguments: swc_next_ecma_ast::SubRange<swc_next_ecma_ast::Argument>,
  ctx: SideEffectsContext<'_, '_>,
) -> bool {
  ctx
    .ast
    .nodes(arguments)
    .any(|argument| match ctx.ast.argument_data(argument) {
      ArgumentData::Expr(expression) => may_have_side_effects(expression, ctx),
      ArgumentData::SpreadElement(_) => true,
    })
}

fn statement_may_have_side_effects(statement: Stmt, ctx: SideEffectsContext<'_, '_>) -> bool {
  let ast = ctx.ast;
  match ast.stmt_data(statement) {
    StmtData::BlockStatement(block) => ast
      .nodes(block.body(ast))
      .any(|node| statement_may_have_side_effects(node, ctx)),
    StmtData::EmptyStatement(_) => false,
    StmtData::LabeledStatement(labeled) => statement_may_have_side_effects(labeled.body(ast), ctx),
    StmtData::IfStatement(if_statement) => {
      may_have_side_effects(if_statement.test(ast), ctx)
        || statement_may_have_side_effects(if_statement.consequent(ast), ctx)
        || if_statement
          .alternate(ast)
          .is_some_and(|statement| statement_may_have_side_effects(statement, ctx))
    }
    StmtData::SwitchStatement(switch) => {
      may_have_side_effects(switch.discriminant(ast), ctx)
        || ast.nodes(switch.cases(ast)).any(|case| {
          case
            .test(ast)
            .is_some_and(|expression| may_have_side_effects(expression, ctx))
            || ast
              .nodes(case.consequent(ast))
              .any(|node| statement_may_have_side_effects(node, ctx))
        })
    }
    StmtData::TryStatement(try_statement) => {
      ast
        .nodes(try_statement.block(ast).body(ast))
        .any(|node| statement_may_have_side_effects(node, ctx))
        || try_statement.handler(ast).is_some_and(|handler| {
          ast
            .nodes(handler.body(ast).body(ast))
            .any(|node| statement_may_have_side_effects(node, ctx))
        })
        || try_statement.finalizer(ast).is_some_and(|finalizer| {
          ast
            .nodes(finalizer.body(ast))
            .any(|node| statement_may_have_side_effects(node, ctx))
        })
    }
    StmtData::Declaration(declaration) => match ast.decl_data(declaration) {
      DeclData::Class(class) => class_has_side_effects(ctx, class),
      DeclData::Function(_) => !ctx.in_strict,
      DeclData::VariableDeclaration(variable) => variable.kind(ast) == VariableKind::Var,
      _ => false,
    },
    StmtData::ExpressionStatement(expression) => {
      may_have_side_effects(expression.expression(ast), ctx)
    }
    _ => true,
  }
}

fn class_has_side_effects(ctx: SideEffectsContext<'_, '_>, class: Class) -> bool {
  let ast = ctx.ast;
  if class
    .super_class(ast)
    .is_some_and(|super_class| may_have_side_effects(super_class, ctx))
  {
    return true;
  }

  for member in ast.nodes(class.body(ast).body(ast)) {
    match ast.class_element_data(member) {
      ClassElementData::MethodDefinition(method) => {
        if method.computed(ast) && property_key_may_have_side_effects(method.key(ast), ctx) {
          return true;
        }
      }
      ClassElementData::TsMethodDefinition(method) => {
        if method.computed(ast) && property_key_may_have_side_effects(method.key(ast), ctx) {
          return true;
        }
      }
      ClassElementData::PropertyDefinition(property) => {
        if property.computed(ast) && property_key_may_have_side_effects(property.key(ast), ctx) {
          return true;
        }
        if property
          .value(ast)
          .is_some_and(|value| may_have_side_effects(value, ctx))
        {
          return true;
        }
      }
      ClassElementData::StaticBlock(block)
        if ast
          .nodes(block.body(ast))
          .any(|node| statement_may_have_side_effects(node, ctx)) =>
      {
        return true;
      }
      ClassElementData::StaticBlock(_) | ClassElementData::TsIndexSignature(_) => {}
    }
  }

  false
}

fn class_member_access_may_have_side_effects(
  ctx: SideEffectsContext<'_, '_>,
  class: Class,
) -> bool {
  let ast = ctx.ast;
  ast
    .nodes(class.body(ast).body(ast))
    .any(|member| match ast.class_element_data(member) {
      ClassElementData::MethodDefinition(method) => {
        method.r#static(ast)
          && matches!(
            method.kind(ast),
            MethodDefinitionKind::Get | MethodDefinitionKind::Set
          )
      }
      ClassElementData::TsMethodDefinition(method) => {
        method.r#static(ast)
          && matches!(
            method.kind(ast),
            MethodDefinitionKind::Get | MethodDefinitionKind::Set
          )
      }
      _ => false,
    })
}

fn object_member_access_may_have_side_effects(
  ctx: SideEffectsContext<'_, '_>,
  object: swc_next_ecma_ast::ObjectExpression,
) -> bool {
  let ast = ctx.ast;
  ast.nodes(object.properties(ast)).any(|property| {
    let ObjectPropertyKindData::ObjectProperty(property) = ast.object_property_kind_data(property)
    else {
      return true;
    };
    if property.computed(ast) || property.method(ast) || property.kind(ast) != PropertyKind::Init {
      return true;
    }
    property_key_is(property.key(ast), ctx, "__proto__")
  })
}

fn property_key_may_have_side_effects(key: PropertyKey, ctx: SideEffectsContext<'_, '_>) -> bool {
  match classify_property_key(ctx.ast, key) {
    PropertyKeyValue::OtherExpr(expression) => may_have_side_effects(expression, ctx),
    _ => false,
  }
}

fn property_key_is(key: PropertyKey, ctx: SideEffectsContext<'_, '_>, expected: &str) -> bool {
  let ast = ctx.ast;
  match classify_property_key(ast, key) {
    PropertyKeyValue::IdentifierName(identifier) => ast.get_utf8(identifier.name(ast)) == expected,
    PropertyKeyValue::StringLiteral(string) => {
      ast.get_wtf8(string.value(ast)).as_str() == Some(expected)
    }
    _ => false,
  }
}

fn is_pure_new_callee(expression: Expr, ctx: SideEffectsContext<'_, '_>) -> bool {
  let ast = ctx.ast;
  match ast.expr_data(expression) {
    ExprData::Function(function) => is_empty_function(ast, function),
    ExprData::Class(class) => {
      if class.super_class(ast).is_some() || class_has_side_effects(ctx, class) {
        return false;
      }

      for member in ast.nodes(class.body(ast).body(ast)) {
        match ast.class_element_data(member) {
          ClassElementData::PropertyDefinition(property) if !property.r#static(ast) => {
            return false;
          }
          ClassElementData::MethodDefinition(method)
            if method.kind(ast) == MethodDefinitionKind::Constructor
              && !method.value(ast).body(ast).body(ast).is_empty() =>
          {
            return false;
          }
          _ => {}
        }
      }

      true
    }
    _ => false,
  }
}

fn is_empty_function(ast: &Ast<'_>, function: Function) -> bool {
  formal_parameters_are_simple_identifiers(ast, function.params(ast))
    && function.body(ast).body(ast).is_empty()
}

fn is_pure_string_method(method: &str) -> bool {
  matches!(
    method,
    "charAt"
      | "charCodeAt"
      | "concat"
      | "endsWith"
      | "includes"
      | "indexOf"
      | "lastIndexOf"
      | "localeCompare"
      | "slice"
      | "split"
      | "startsWith"
      | "substr"
      | "substring"
      | "toLocaleLowerCase"
      | "toLocaleUpperCase"
      | "toLowerCase"
      | "toString"
      | "toUpperCase"
      | "trim"
      | "trimEnd"
      | "trimStart"
  )
}
