use rspack_core::{BoxDependencyTemplate, ConstDependency, ContextDependency, DependencyRange};
use rspack_util::{SpanExt, itoa};
use swc_core::{
  atoms::Atom,
  common::Spanned,
  ecma::ast::{CallExpr, VarDeclarator},
};

use super::JavascriptParserPlugin;
use crate::{
  dependency::CommonJsRequireContextDependency,
  visitors::{JavascriptParser, Statement, TagInfoData, VariableDeclaration, expr_name},
};

pub(super) const NESTED_IDENTIFIER_TAG: &str = "_identifier__nested_rspack_identifier__";

#[derive(Debug, Clone)]
pub(super) struct NestedRequireData {
  pub name: String,
  update: bool,
  loc: DependencyRange,
  in_short_hand: bool,
}

pub(crate) struct CompatibilityPlugin;

impl CompatibilityPlugin {
  pub(crate) fn browserify_require_handler(
    &self,
    parser: &mut JavascriptParser,
    expr: &CallExpr,
  ) -> Option<bool> {
    if expr.args.len() != 2 {
      return None;
    }
    let second = parser.evaluate_expression(&expr.args[1].expr);
    if !second.is_bool() || !matches!(second.as_bool(), Some(true)) {
      return None;
    }
    let dep = ConstDependency::new(expr.callee.span().into(), "require".into());
    if let Some(last) = parser.pop_dependency() {
      if let Some(last) = last.downcast_ref::<CommonJsRequireContextDependency>()
        && let options = last.options()
        && options.recursive
        && options.request == "."
      {
      } else {
        parser.add_dependency(last);
      }
    }
    parser.add_presentational_dependency(Box::new(dep));
    Some(true)
  }

  fn tag_nested_require_data(
    &self,
    parser: &mut JavascriptParser,
    name: Atom,
    rename: String,
    in_short_hand: bool,
    start: u32,
    end: u32,
  ) {
    parser.tag_variable(
      name,
      NESTED_IDENTIFIER_TAG,
      Some(NestedRequireData {
        name: rename,
        update: false,
        loc: DependencyRange::new(start, end),
        in_short_hand,
      }),
    );
  }
}

impl JavascriptParserPlugin for CompatibilityPlugin {
  fn program(
    &self,
    parser: &mut JavascriptParser,
    ast: &swc_core::ecma::ast::Program,
  ) -> Option<bool> {
    if ast
      .as_module()
      .and_then(|m| m.shebang.as_ref())
      .or_else(|| ast.as_script().and_then(|s| s.shebang.as_ref()))
      .is_some()
    {
      parser
        .add_presentational_dependency(Box::new(ConstDependency::new((0, 0).into(), "//".into())));
    }

    None
  }

  fn pre_declarator(
    &self,
    parser: &mut JavascriptParser,
    decl: &VarDeclarator,
    _statement: VariableDeclaration<'_>,
  ) -> Option<bool> {
    let ident = decl.name.as_ident()?;

    if ident.sym.as_str() == parser.parser_runtime_requirements.require {
      let start = ident.span().real_lo();
      let end = ident.span().real_hi();
      self.tag_nested_require_data(
        parser,
        ident.sym.clone(),
        {
          let mut start_buffer = itoa::Buffer::new();
          let start_str = start_buffer.format(start);
          let mut end_buffer = itoa::Buffer::new();
          let end_str = end_buffer.format(end);
          format!("__nested_rspack_require_{start_str}_{end_str}__")
        },
        parser.in_short_hand,
        start,
        end,
      );
      return Some(true);
    } else if ident.sym.as_str() == parser.parser_runtime_requirements.exports {
      self.tag_nested_require_data(
        parser,
        ident.sym.clone(),
        "__nested_rspack_exports__".to_string(),
        parser.in_short_hand,
        ident.span().real_lo(),
        ident.span().real_hi(),
      );
      return Some(true);
    }

    None
  }

  fn pattern(
    &self,
    parser: &mut JavascriptParser,
    ident: &swc_core::ecma::ast::Ident,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == parser.parser_runtime_requirements.exports {
      self.tag_nested_require_data(
        parser,
        ident.sym.clone(),
        "__nested_rspack_exports__".to_string(),
        parser.in_short_hand,
        ident.span().real_lo(),
        ident.span().real_hi(),
      );
      return Some(true);
    } else if for_name == parser.parser_runtime_requirements.require {
      let start = ident.span().real_lo();
      let end = ident.span().real_hi();
      self.tag_nested_require_data(
        parser,
        ident.sym.clone(),
        {
          let mut start_buffer = itoa::Buffer::new();
          let start_str = start_buffer.format(start);
          let mut end_buffer = itoa::Buffer::new();
          let end_str = end_buffer.format(end);
          format!("__nested_rspack_require_{start_str}_{end_str}__")
        },
        parser.in_short_hand,
        start,
        end,
      );
      return Some(true);
    }
    None
  }

  fn pre_statement(&self, parser: &mut JavascriptParser, stmt: Statement) -> Option<bool> {
    let fn_decl = stmt.as_function_decl()?;
    let ident = fn_decl.ident()?;
    let name = &ident.sym;
    if name.as_str() != parser.parser_runtime_requirements.require {
      None
    } else {
      self.tag_nested_require_data(
        parser,
        name.clone(),
        {
          let mut lo_buffer = itoa::Buffer::new();
          let lo_str = lo_buffer.format(fn_decl.span().real_lo());
          format!("__nested_rspack_require_{lo_str}__")
        },
        parser.in_short_hand,
        ident.span().real_lo(),
        ident.span().real_hi(),
      );
      Some(true)
    }
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &swc_core::ecma::ast::Ident,
    for_name: &str,
  ) -> Option<bool> {
    if for_name != NESTED_IDENTIFIER_TAG {
      return None;
    }
    let tag_info = parser
      .definitions_db
      .expect_get_mut_tag_info(parser.current_tag_info?);

    let mut nested_require_data = NestedRequireData::downcast(tag_info.data.take()?);
    let mut deps: Vec<BoxDependencyTemplate> = Vec::with_capacity(2);
    let name = nested_require_data.name.clone();
    if !nested_require_data.update {
      let shorthand = nested_require_data.in_short_hand;
      deps.push(Box::new(ConstDependency::new(
        nested_require_data.loc,
        if shorthand {
          format!("{}: {}", ident.sym, name).into()
        } else {
          name.clone().into()
        },
      )));
      nested_require_data.update = true;
    }
    tag_info.data = Some(NestedRequireData::into_any(nested_require_data));

    deps.push(Box::new(ConstDependency::new(
      ident.span.into(),
      if parser.in_short_hand {
        format!("{}: {}", ident.sym, name).into()
      } else {
        name.into()
      },
    )));
    parser.add_presentational_dependencies(deps);
    Some(true)
  }

  fn call(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == expr_name::REQUIRE {
      return self.browserify_require_handler(parser, expr);
    }
    None
  }
}
