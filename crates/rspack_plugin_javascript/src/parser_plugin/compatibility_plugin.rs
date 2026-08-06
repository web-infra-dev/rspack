use rspack_core::{BoxDependencyTemplate, ConstDependency, ContextDependency, DependencyRange};
use rspack_util::{SpanExt, itoa};
use swc_atoms::Atom;
use swc_experimental_ecma_ast::{CallExpr, GetSpan, Ident, Program, VarDeclarator};

use super::JavascriptParserPlugin;
use crate::{
  dependency::CommonJsRequireContextDependency,
  visitors::{JavascriptParser, Statement, TagInfoData, VariableDeclaration, expr_name},
};

pub const NESTED_IDENTIFIER_TAG: &str = "_identifier__nested_rspack_identifier__";

#[derive(Debug, Clone)]
pub struct NestedRequireData {
  pub name: String,
  update: bool,
  loc: DependencyRange,
  in_short_hand: bool,
}

pub struct CompatibilityPlugin;

impl CompatibilityPlugin {
  fn nested_require_name<'a>(&self, parser: &'a JavascriptParser) -> &'a str {
    parser
      .parser_runtime_requirements
      .compatibility_runtime_scope
      .as_str()
  }

  pub fn browserify_require_handler(
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

  /// Materialize the declaration replacement before an export records the nested name.
  /// A non-self-referential function would not otherwise visit the identifier hook.
  pub(crate) fn update_nested_binding_declaration(
    parser: &mut JavascriptParser,
    name: &Atom,
  ) -> Option<Atom> {
    let faster_module_concatenation = parser
      .compiler_options
      .experiments
      .faster_module_concatenation;
    let (nested_name, dep) = {
      let data = parser.get_tag_data_mut::<NestedRequireData>(name, NESTED_IDENTIFIER_TAG)?;
      let nested_name = Atom::from(data.name.as_str());
      let content = if data.in_short_hand {
        format!("{name}: {}", data.name).into()
      } else {
        data.name.clone().into()
      };
      let dep = if data.update {
        None
      } else if faster_module_concatenation {
        Some(ConstDependency::new_with_concatenation_scope_identifier(
          data.loc,
          content,
          data.name.clone().into(),
        ))
      } else {
        Some(ConstDependency::new(data.loc, content))
      };
      data.update = true;
      (nested_name, dep)
    };
    if let Some(dep) = dep {
      parser.add_presentational_dependency(Box::new(dep));
    }
    Some(nested_name)
  }
}

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for CompatibilityPlugin {
  fn program(&self, parser: &mut JavascriptParser<'p>, ast: &Program) -> Option<bool> {
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
    parser: &mut JavascriptParser<'p>,
    decl: &VarDeclarator,
    _statement: VariableDeclaration<'_>,
  ) -> Option<bool> {
    let ident = decl.name.as_ident()?;

    if ident.id.sym.as_str() == self.nested_require_name(parser) {
      let span = ident.span();
      let start = span.real_lo();
      let end = span.real_hi();
      self.tag_nested_require_data(
        parser,
        Atom::from(ident.id.sym.as_str()),
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
    } else if ident.id.sym.as_str() == parser.parser_runtime_requirements.exports {
      let span = ident.span();
      self.tag_nested_require_data(
        parser,
        Atom::from(ident.id.sym.as_str()),
        "__nested_rspack_exports__".to_string(),
        parser.in_short_hand,
        span.real_lo(),
        span.real_hi(),
      );
      return Some(true);
    }

    None
  }

  fn pattern(
    &self,
    parser: &mut JavascriptParser<'p>,
    ident: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == parser.parser_runtime_requirements.exports {
      self.tag_nested_require_data(
        parser,
        Atom::from(ident.sym.as_str()),
        "__nested_rspack_exports__".to_string(),
        parser.in_short_hand,
        ident.span().real_lo(),
        ident.span().real_hi(),
      );
      if !parser.is_top_level_scope() {
        return Some(true);
      }
    } else if for_name == self.nested_require_name(parser) {
      let span = ident.span();
      let start = span.real_lo();
      let end = span.real_hi();
      self.tag_nested_require_data(
        parser,
        Atom::from(ident.sym.as_str()),
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
      if !parser.is_top_level_scope() {
        return Some(true);
      }
    }
    None
  }

  fn pre_statement(&self, parser: &mut JavascriptParser<'p>, stmt: Statement) -> Option<bool> {
    let fn_decl = stmt.as_function_decl()?;
    let ident = fn_decl.ident()?;
    let name = &ident.sym;
    if name.as_str() != self.nested_require_name(parser) {
      None
    } else {
      self.tag_nested_require_data(
        parser,
        Atom::from(name.as_str()),
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

  fn declarator(
    &self,
    parser: &mut JavascriptParser,
    declarator: &VarDeclarator,
    _stmt: VariableDeclaration<'_>,
  ) -> Option<bool> {
    let faster_module_concatenation = parser
      .compiler_options
      .experiments
      .faster_module_concatenation;
    if let Some(ident) = declarator.name.as_ident()
      && (ident.id.sym.as_str() == parser.parser_runtime_requirements.exports
        || ident.id.sym.as_str() == self.nested_require_name(parser))
    {
      let data = parser.get_tag_data_mut::<NestedRequireData>(
        &Atom::from(ident.id.sym.as_str()),
        NESTED_IDENTIFIER_TAG,
      )?;
      if !data.update {
        let dep = if faster_module_concatenation {
          ConstDependency::new_with_concatenation_scope_identifier(
            data.loc,
            data.name.clone().into(),
            data.name.clone().into(),
          )
        } else {
          ConstDependency::new(data.loc, data.name.clone().into())
        };
        data.update = true;
        parser.add_presentational_dependency(Box::new(dep));
      }
    }
    None
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    ident: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    if for_name != NESTED_IDENTIFIER_TAG {
      return None;
    }
    let faster_module_concatenation = parser
      .compiler_options
      .experiments
      .faster_module_concatenation;
    let tag_info = parser
      .definitions_db
      .expect_get_mut_tag_info(parser.current_tag_info?)
      .data
      .as_deref_mut()?;

    let nested_require_data = NestedRequireData::downcast_mut(tag_info);
    let mut deps: Vec<BoxDependencyTemplate> = Vec::with_capacity(2);
    let name = nested_require_data.name.clone();
    if !nested_require_data.update {
      let shorthand = nested_require_data.in_short_hand;
      let content = if shorthand {
        format!("{}: {}", ident.sym, name).into()
      } else {
        name.clone().into()
      };
      let dep = if faster_module_concatenation {
        ConstDependency::new_with_concatenation_scope_identifier(
          nested_require_data.loc,
          content,
          name.clone().into(),
        )
      } else {
        ConstDependency::new(nested_require_data.loc, content)
      };
      deps.push(Box::new(dep));
      nested_require_data.update = true;
    }

    let content = if parser.in_short_hand {
      format!("{}: {}", ident.sym, name).into()
    } else {
      name.clone().into()
    };
    let dep = if faster_module_concatenation {
      ConstDependency::new_with_concatenation_scope_identifier(
        ident.span.into(),
        content,
        name.into(),
      )
    } else {
      ConstDependency::new(ident.span.into(), content)
    };
    deps.push(Box::new(dep));
    parser.add_presentational_dependencies(deps);
    Some(true)
  }

  fn call(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == expr_name::REQUIRE {
      return self.browserify_require_handler(parser, expr);
    }
    None
  }
}
