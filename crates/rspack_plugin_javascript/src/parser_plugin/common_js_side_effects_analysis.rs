use rspack_util::swc::{AstSubRangeExt, PropertyKeyValue, classify_property_key};
use swc_next_ecma_ast::{
  AssignmentExpression, AssignmentOperator, AssignmentTargetData, Ast, CallExpression, Expr,
  ExprData, MemberExpression, ObjectExpression, ObjectPropertyKindData, PropertyKey, PropertyKind,
  SimpleAssignmentTargetData,
};

use super::side_effects_analysis::property_key_is;

pub(super) enum CommonJsExports {
  ExportsMember,
  ModuleExports,
  ModuleExportsMember,
}

impl CommonJsExports {
  pub(super) fn root_name(&self) -> &'static str {
    match self {
      Self::ExportsMember => "exports",
      Self::ModuleExports | Self::ModuleExportsMember => "module",
    }
  }
}

pub(super) enum ExportsEffect {
  Assign {
    target: CommonJsExports,
    value: Expr,
  },
  DefineProperty {
    target: CommonJsExports,
    descriptor: ObjectExpression,
  },
}

impl ExportsEffect {
  pub(super) fn target(&self) -> &CommonJsExports {
    match self {
      Self::Assign { target, .. } | Self::DefineProperty { target, .. } => target,
    }
  }

  pub(super) fn all_operands(&self, ast: &Ast<'_>, mut is_pure: impl FnMut(Expr) -> bool) -> bool {
    match self {
      Self::Assign { value, .. } => is_pure(*value),
      Self::DefineProperty { descriptor, .. } => {
        ast.nodes(descriptor.properties(ast)).all(|property| {
          property
            .as_object_property(ast)
            .is_some_and(|property| is_pure(property.value(ast)))
        })
      }
    }
  }
}

pub(super) fn exports_effect(ast: &Ast<'_>, expression: Expr) -> Option<ExportsEffect> {
  match ast.expr_data(expression) {
    ExprData::AssignmentExpression(assignment) => {
      common_js_export_write(ast, assignment).map(|target| ExportsEffect::Assign {
        target,
        value: assignment.right(ast),
      })
    }
    ExprData::CallExpression(call) => common_js_define_property(ast, call)
      .map(|(target, descriptor)| ExportsEffect::DefineProperty { target, descriptor }),
    _ => None,
  }
}

fn common_js_export_write(
  ast: &Ast<'_>,
  assignment: AssignmentExpression,
) -> Option<CommonJsExports> {
  if assignment.operator(ast) != AssignmentOperator::Assign {
    return None;
  }
  let member = assignment_member_target(ast, assignment)?;
  let property = static_member_property(ast, member)?;
  let object = member.object(ast);

  if let Some(identifier) = object.as_identifier_reference(ast) {
    return match (ast.get_utf8(identifier.name(ast)), property) {
      ("exports", property) if property != "__proto__" => Some(CommonJsExports::ExportsMember),
      ("module", "exports") => Some(CommonJsExports::ModuleExports),
      _ => None,
    };
  }

  (property != "__proto__"
    && object
      .as_member_expression(ast)
      .is_some_and(|object| is_module_exports(ast, object)))
  .then_some(CommonJsExports::ModuleExportsMember)
}

fn common_js_define_property(
  ast: &Ast<'_>,
  call: CallExpression,
) -> Option<(CommonJsExports, ObjectExpression)> {
  let callee = call.callee(ast).as_member_expression(ast)?;
  let object = callee.object(ast).as_identifier_reference(ast)?;
  if ast.get_utf8(object.name(ast)) != "Object"
    || static_member_property(ast, callee) != Some("defineProperty")
  {
    return None;
  }

  let arguments = call.arguments(ast);
  if arguments.len() != 3 {
    return None;
  }
  let exports = define_property_target(ast, ast.first(arguments)?.as_expr(ast)?)?;
  let ExprData::StringLiteral(name) = ast.expr_data(ast.second(arguments)?.as_expr(ast)?) else {
    return None;
  };
  if ast.get_wtf8(name.value(ast)).as_str()? == "__proto__" {
    return None;
  }
  let descriptor = arguments
    .get_node(ast, 2)?
    .as_expr(ast)?
    .as_object_expression(ast)?;
  is_data_descriptor(ast, descriptor).then_some((exports, descriptor))
}

fn define_property_target(ast: &Ast<'_>, target: Expr) -> Option<CommonJsExports> {
  if let Some(identifier) = target.as_identifier_reference(ast) {
    return (ast.get_utf8(identifier.name(ast)) == "exports")
      .then_some(CommonJsExports::ExportsMember);
  }
  target
    .as_member_expression(ast)
    .is_some_and(|member| is_module_exports(ast, member))
    .then_some(CommonJsExports::ModuleExportsMember)
}

fn is_data_descriptor(ast: &Ast<'_>, descriptor: ObjectExpression) -> bool {
  ast.nodes(descriptor.properties(ast)).all(|property| {
    property.as_object_property(ast).is_some_and(|property| {
      property.kind(ast) == PropertyKind::Init
        && !property.computed(ast)
        && !property.method(ast)
        && is_data_descriptor_key(ast, property.key(ast))
    })
  })
}

fn is_data_descriptor_key(ast: &Ast<'_>, key: PropertyKey) -> bool {
  let name = match classify_property_key(ast, key) {
    PropertyKeyValue::IdentifierName(identifier) => Some(ast.get_utf8(identifier.name(ast))),
    PropertyKeyValue::StringLiteral(string) => ast.get_wtf8(string.value(ast)).as_str(),
    _ => None,
  };
  matches!(
    name,
    Some("value" | "writable" | "enumerable" | "configurable")
  )
}

fn assignment_member_target(
  ast: &Ast<'_>,
  assignment: AssignmentExpression,
) -> Option<MemberExpression> {
  let AssignmentTargetData::SimpleAssignmentTarget(target) =
    ast.assignment_target_data(assignment.left(ast))
  else {
    return None;
  };
  let SimpleAssignmentTargetData::MemberExpression(member) =
    ast.simple_assignment_target_data(target)
  else {
    return None;
  };
  Some(member)
}

fn is_module_exports(ast: &Ast<'_>, member: MemberExpression) -> bool {
  static_member_property(ast, member) == Some("exports")
    && member
      .object(ast)
      .as_identifier_reference(ast)
      .is_some_and(|identifier| ast.get_utf8(identifier.name(ast)) == "module")
}

fn static_member_property<'a>(ast: &Ast<'a>, member: MemberExpression) -> Option<&'a str> {
  match classify_property_key(ast, member.property(ast)) {
    PropertyKeyValue::IdentifierName(property) if !member.computed(ast) => {
      Some(ast.get_utf8(property.name(ast)))
    }
    PropertyKeyValue::StringLiteral(property) if member.computed(ast) => {
      ast.get_wtf8(property.value(ast)).as_str()
    }
    _ => None,
  }
}

pub(super) fn is_plain_object_literal(ast: &Ast<'_>, expression: Expr) -> bool {
  let ExprData::ObjectExpression(object) = ast.expr_data(expression) else {
    return false;
  };
  ast.nodes(object.properties(ast)).all(|property| {
    let ObjectPropertyKindData::ObjectProperty(property) = ast.object_property_kind_data(property)
    else {
      return true;
    };
    if property.kind(ast) != PropertyKind::Init {
      return false;
    }
    if property.computed(ast)
      || property.shorthand(ast)
      || property.method(ast)
      || !property_key_is(ast, property.key(ast), "__proto__")
    {
      return true;
    }
    let prototype = property.value(ast);
    matches!(ast.expr_data(prototype), ExprData::NullLiteral(_))
      || is_plain_object_literal(ast, prototype)
  })
}
