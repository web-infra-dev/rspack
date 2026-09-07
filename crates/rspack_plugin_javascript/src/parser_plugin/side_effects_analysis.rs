/**
 * Some code is modified based on
 * https://github.com/CPunisher/swc-experimental/blob/57cf78c4bc1b963d4e4946492b88164958e6f764/crates/swc_ecma_utils/src/lib.rs
 * Apache-2.0 licensed
 */
// Conservative, syntax-level side-effects analysis.
//
// Rspack-specific policies such as `/*#__PURE__*/`, `pureFunctions`, parser hooks, and deferred
// import checks stay in `side_effects_parser_plugin`.
use swc_next_ecma_ast::{
  ArgumentData, Ast, Class, ClassElementData, DeclData, Expr, ExprData, Function,
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
  in_strict: bool,
  remaining_depth: u8,
}

impl<'a, 'ast> SideEffectsContext<'a, 'ast> {
  pub(super) fn new(
    ast: &'a Ast<'ast>,
    semantic: &'a Semantic<'ast>,
    is_unresolved_ref_safe: bool,
  ) -> Self {
    Self {
      ast,
      semantic,
      is_unresolved_ref_safe,
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

  if is_pure_callee(expression, ctx) {
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
    ExprData::ArrayExpression(array) => array
      .elements(ast)
      .iter()
      .filter_map(|slot| ast.get_node_in_sub_range(slot))
      .any(|argument| match ast.argument_data(argument) {
        ArgumentData::Expr(expression) => may_have_side_effects(expression, ctx),
        ArgumentData::SpreadElement(_) => true,
      }),
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
        arguments_may_have_side_effects(call.arguments(ast), ctx)
      }
      _ => true,
    },
    ExprData::NewExpression(new_expression)
      if is_pure_new_callee(new_expression.callee(ast), ctx) =>
    {
      arguments_may_have_side_effects(new_expression.arguments(ast), ctx)
    }
    ExprData::NewExpression(_) => true,
    ExprData::CallExpression(call) if is_pure_callee(call.callee(ast), ctx) => {
      arguments_may_have_side_effects(call.arguments(ast), ctx)
    }
    ExprData::CallExpression(_) => true,
    ExprData::SequenceExpression(sequence) => sequence
      .expressions(ast)
      .iter()
      .any(|slot| may_have_side_effects(ast.get_node_in_sub_range(slot), ctx)),
    ExprData::ConditionalExpression(conditional) => {
      may_have_side_effects(conditional.test(ast), ctx)
        || may_have_side_effects(conditional.consequent(ast), ctx)
        || may_have_side_effects(conditional.alternate(ast), ctx)
    }
    ExprData::ObjectExpression(object) => object.properties(ast).iter().any(|slot| {
      let property = ast.get_node_in_sub_range(slot);
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
    }),
    ExprData::JsxElement(_) | ExprData::JsxFragment(_) => true,
  }
}

fn is_pure_callee(expression: Expr, ctx: SideEffectsContext<'_, '_>) -> bool {
  if is_global_ref_to(expression, ctx, "Date") {
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
      let object = member.object(ast);

      if is_global_ref_to(object, ctx, "Math") {
        return true;
      }

      match ast.expr_data(object) {
        ExprData::IdentifierReference(identifier) => ast.get_utf8(identifier.name(ast)) == "Math",
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

fn is_unresolved_reference(
  identifier: swc_next_ecma_ast::IdentifierReference,
  ctx: SideEffectsContext<'_, '_>,
) -> bool {
  ctx
    .semantic
    .reference_of(identifier.node_id())
    .map(|reference| ctx.semantic.reference(reference))
    .is_some_and(|reference| reference.symbol.is_none() && !reference.flags.is_dynamic())
}

fn is_global_ref_to(expression: Expr, ctx: SideEffectsContext<'_, '_>, id: &str) -> bool {
  let ast = ctx.ast;
  let ExprData::IdentifierReference(identifier) = ast.expr_data(expression) else {
    return false;
  };
  ast.get_utf8(identifier.name(ast)) == id && is_unresolved_reference(identifier, ctx)
}

fn arguments_may_have_side_effects(
  arguments: swc_next_ecma_ast::TypedSubRange<swc_next_ecma_ast::Argument>,
  ctx: SideEffectsContext<'_, '_>,
) -> bool {
  arguments.iter().any(|slot| {
    let argument = ctx.ast.get_node_in_sub_range(slot);
    match ctx.ast.argument_data(argument) {
      ArgumentData::Expr(expression) => may_have_side_effects(expression, ctx),
      ArgumentData::SpreadElement(_) => true,
    }
  })
}

fn statement_may_have_side_effects(statement: Stmt, ctx: SideEffectsContext<'_, '_>) -> bool {
  let ast = ctx.ast;
  match ast.stmt_data(statement) {
    StmtData::BlockStatement(block) => block
      .body(ast)
      .iter()
      .any(|slot| statement_may_have_side_effects(ast.get_node_in_sub_range(slot), ctx)),
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
        || switch.cases(ast).iter().any(|slot| {
          let case = ast.get_node_in_sub_range(slot);
          case
            .test(ast)
            .is_some_and(|expression| may_have_side_effects(expression, ctx))
            || case
              .consequent(ast)
              .iter()
              .any(|slot| statement_may_have_side_effects(ast.get_node_in_sub_range(slot), ctx))
        })
    }
    StmtData::TryStatement(try_statement) => {
      try_statement
        .block(ast)
        .body(ast)
        .iter()
        .any(|slot| statement_may_have_side_effects(ast.get_node_in_sub_range(slot), ctx))
        || try_statement.handler(ast).is_some_and(|handler| {
          handler
            .body(ast)
            .body(ast)
            .iter()
            .any(|slot| statement_may_have_side_effects(ast.get_node_in_sub_range(slot), ctx))
        })
        || try_statement.finalizer(ast).is_some_and(|finalizer| {
          finalizer
            .body(ast)
            .iter()
            .any(|slot| statement_may_have_side_effects(ast.get_node_in_sub_range(slot), ctx))
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

  for slot in class.body(ast).body(ast).iter() {
    let member = ast.get_node_in_sub_range(slot);
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
        if block
          .body(ast)
          .iter()
          .any(|slot| statement_may_have_side_effects(ast.get_node_in_sub_range(slot), ctx)) =>
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
  class.body(ast).body(ast).iter().any(|slot| {
    let member = ast.get_node_in_sub_range(slot);
    match ast.class_element_data(member) {
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
    }
  })
}

fn object_member_access_may_have_side_effects(
  ctx: SideEffectsContext<'_, '_>,
  object: swc_next_ecma_ast::ObjectExpression,
) -> bool {
  let ast = ctx.ast;
  object.properties(ast).iter().any(|slot| {
    let property = ast.get_node_in_sub_range(slot);
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
  match ctx.ast.property_key_data(key) {
    PropertyKeyData::Expr(expression) => may_have_side_effects(expression, ctx),
    _ => false,
  }
}

fn property_key_is(key: PropertyKey, ctx: SideEffectsContext<'_, '_>, expected: &str) -> bool {
  let ast = ctx.ast;
  match ast.property_key_data(key) {
    PropertyKeyData::IdentifierName(identifier) => ast.get_utf8(identifier.name(ast)) == expected,
    PropertyKeyData::StringLiteral(string) => {
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

      for slot in class.body(ast).body(ast).iter() {
        let member = ast.get_node_in_sub_range(slot);
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
