use rspack_util::swc::{AstSubRangeExt, PropertyKeyValue, classify_property_key};
use swc_next_ecma_ast::{
  AssignmentExpression, AssignmentOperator, AssignmentTargetData, Ast, Expr, ExprData,
  MemberExpression, ObjectPropertyKindData, PropertyKind, SimpleAssignmentTargetData,
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

pub(super) fn common_js_export_write(
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
