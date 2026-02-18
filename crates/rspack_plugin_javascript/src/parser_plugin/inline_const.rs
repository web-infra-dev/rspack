use rspack_core::EvaluatedInlinableValue;
use rspack_util::ryu_js;
use swc_experimental_ecma_ast::{ModuleDecl, ModuleItem, Program, VarDeclarator};

use super::JavascriptParserPlugin;
use crate::{
  utils::eval::{
    BasicEvaluatedExpression, evaluate_to_boolean, evaluate_to_null, evaluate_to_number,
    evaluate_to_string, evaluate_to_undefined,
  },
  visitors::{
    JavascriptParser, TagInfoData, VariableDeclaration, VariableDeclarationKind,
    scope_info::VariableInfoFlags,
  },
};

pub const INLINABLE_CONST_TAG: &str = "inlinable const";

#[derive(Debug, Clone)]
pub struct InlinableConstData {
  pub value: EvaluatedInlinableValue,
}

#[derive(Default)]
pub struct InlineConstPlugin;

impl JavascriptParserPlugin for InlineConstPlugin {
  fn program(&self, parser: &mut JavascriptParser, program: Program) -> Option<bool> {
    if let Some(module) = program.as_module() {
      for item in module.body(&parser.ast).iter() {
        let item = parser.ast.get_node_in_sub_range(item);
        match item {
          ModuleItem::ModuleDecl(m) => {
            if m.is_import()
              || m.is_export_all()
              || matches!(m, ModuleDecl::ExportNamed(m) if m.src(&parser.ast).is_some())
            {
              // For now we only handle cross-module const inlining, we might don't need to
              // inline const inside the module if we leave it to minimizer?
              parser.has_inlinable_const_decls = false;
              break;
            }
          }
          ModuleItem::Stmt(_) => {}
        }
      }
    }

    None
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser,
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression> {
    if !parser.has_inlinable_const_decls || for_name != INLINABLE_CONST_TAG {
      return None;
    }
    // Propagate inlinable constants. Help the rest const variable declarations that referencing the
    // inlinable constants to evaluate to an inlinable constants.
    let tag_info = parser
      .definitions_db
      .expect_get_tag_info(parser.current_tag_info?);
    let data = InlinableConstData::downcast(tag_info.data.clone()?);
    Some(match data.value {
      EvaluatedInlinableValue::Null => evaluate_to_null(start, end),
      EvaluatedInlinableValue::Undefined => evaluate_to_undefined(start, end),
      EvaluatedInlinableValue::Boolean(v) => evaluate_to_boolean(v, start, end),
      EvaluatedInlinableValue::Number(v) => evaluate_to_number(v, start, end),
      EvaluatedInlinableValue::String(v) => evaluate_to_string(v.to_string(), start, end),
    })
  }

  fn pre_declarator(
    &self,
    parser: &mut JavascriptParser,
    declarator: VarDeclarator,
    declaration: VariableDeclaration,
  ) -> Option<bool> {
    if !parser.has_inlinable_const_decls || !parser.is_top_level_scope() {
      return None;
    }
    if matches!(
      declaration.kind(&parser.ast),
      VariableDeclarationKind::Const
    ) && let Some(name) = declarator.name(&parser.ast).as_ident()
      && let Some(init) = declarator.init(&parser.ast)
    {
      let evaluated = parser.evaluate_expression(init);
      if let Some(inlinable) = to_evaluated_inlinable_value(&evaluated) {
        parser.tag_variable_with_flags(
          parser.ast.get_atom(name.id(&parser.ast).sym(&parser.ast)),
          INLINABLE_CONST_TAG,
          Some(InlinableConstData { value: inlinable }),
          VariableInfoFlags::NORMAL,
        );
      }
    }
    None
  }
}

fn to_evaluated_inlinable_value(
  evaluated: &BasicEvaluatedExpression,
) -> Option<EvaluatedInlinableValue> {
  if evaluated.is_bool() {
    Some(EvaluatedInlinableValue::new_boolean(evaluated.bool()))
  } else if evaluated.is_number()
    && let num = evaluated.number()
    && ryu_js::Buffer::new().format(num).len() <= EvaluatedInlinableValue::SHORT_SIZE
  {
    Some(EvaluatedInlinableValue::new_number(num))
  } else if evaluated.is_string()
    && let str = evaluated.string()
    && str.len() <= EvaluatedInlinableValue::SHORT_SIZE
  {
    Some(EvaluatedInlinableValue::new_string(str.as_str().into()))
  } else if evaluated.is_null() {
    Some(EvaluatedInlinableValue::new_null())
  } else if evaluated.is_undefined() {
    Some(EvaluatedInlinableValue::new_undefined())
  } else {
    None
  }
}
