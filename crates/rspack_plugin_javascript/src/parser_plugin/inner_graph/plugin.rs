use rspack_core::{
  Dependency, DependencyRange, InnerGraphMapSetValue, InnerGraphMapUsage, InnerGraphMapValue,
  InnerGraphState, InnerGraphUsageOperation, TopLevelSymbol, UsedByExports,
};
use rspack_util::SpanExt;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_core::{
  atoms::Atom,
  common::{Mark, Span, Spanned, SyntaxContext},
  ecma::ast::{
    AssignOp, ClassMember, DefaultDecl, ExportDefaultExpr, Expr, ModuleDecl, Pat, VarDeclarator,
  },
};

use crate::{
  ClassExt,
  dependency::PureExpressionDependency,
  parser_plugin::{DEFAULT_STAR_JS_WORD, JavascriptParserPlugin},
  side_effects_parser_plugin::{
    is_pure_class, is_pure_class_member, is_pure_expression, is_pure_function,
  },
  visitors::{
    ExportedVariableInfo, JavascriptParser, Statement, TagInfoData, VariableDeclaration,
    scope_info::VariableInfoFlags,
  },
};

#[derive(Debug)]
pub struct InnerGraphParserPlugin {
  unresolved_context: SyntaxContext,
  analyze_pure_annotation: bool,
}

pub static TOP_LEVEL_SYMBOL: &str = "inner graph top level symbol";

impl InnerGraphParserPlugin {
  pub fn new(unresolved_mark: Mark, analyze_pure_annotation: bool) -> Self {
    Self {
      unresolved_context: SyntaxContext::empty().apply_mark(unresolved_mark),
      analyze_pure_annotation,
    }
  }

  pub fn for_each_expression(parser: &mut JavascriptParser, for_name: &str) {
    if !parser.inner_graph.is_enabled() || for_name != TOP_LEVEL_SYMBOL {
      return;
    }

    if let Some(tag_info) = parser.current_tag_info {
      let tag_info = parser.definitions_db.expect_get_tag_info(tag_info);
      let symbol = TopLevelSymbol::downcast(tag_info.data.clone().expect("should have data"));
      let usage = parser.inner_graph.get_top_level_symbol();
      parser.inner_graph.add_usage(
        symbol,
        match usage {
          Some(atom) => InnerGraphMapUsage::TopLevel(atom),
          None => InnerGraphMapUsage::True,
        },
      )
    }
  }

  pub fn for_each_statement(parser: &mut JavascriptParser, stmt_span: &Span) {
    if let Some(v) = parser
      .inner_graph
      .statement_with_top_level_symbol
      .get(stmt_span)
    {
      parser.inner_graph.set_top_level_symbol(Some(*v));

      if let Some(pure_part) = parser.inner_graph.statement_pure_part.get(stmt_span) {
        let pure_part_start = pure_part.real_lo();
        let pure_part_end = pure_part.real_hi();
        let dep = PureExpressionDependency::new(
          DependencyRange::new(pure_part_start, pure_part_end),
          *parser.module_identifier,
        );
        let dep_id = *dep.id();
        parser.add_dependency(Box::new(dep));
        Self::on_usage(parser, InnerGraphUsageOperation::PureExpression(dep_id));
      }
    }
  }

  pub fn infer_dependency_usage(
    state: &mut InnerGraphState,
  ) -> Vec<(InnerGraphUsageOperation, UsedByExports)> {
    let mut non_terminal = HashSet::from_iter(state.inner_graph.keys().cloned());
    let mut processed: HashMap<TopLevelSymbol, HashSet<InnerGraphMapSetValue>> = HashMap::default();

    while !non_terminal.is_empty() {
      let mut keys_to_remove = vec![];
      for key in non_terminal.iter() {
        let mut new_set = HashSet::default();
        // Using enum to manipulate original is pretty hard, so I use an extra variable to
        // flagging the new set has changed to boolean `true`
        // you could refer https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/optimize/InnerGraph.js#L150
        let mut set_is_true = false;
        let mut is_terminal = true;
        let already_processed = processed.entry(*key).or_default();
        if let Some(InnerGraphMapValue::Set(names)) = state.inner_graph.get(key) {
          for name in names.iter() {
            already_processed.insert(name.clone());
          }
          for name in names {
            match name {
              InnerGraphMapSetValue::Str(v) => {
                new_set.insert(InnerGraphMapSetValue::Str(v.clone()));
              }
              InnerGraphMapSetValue::TopLevel(v) => {
                let item_value = state.inner_graph.get(v);
                match item_value {
                  Some(InnerGraphMapValue::True) => {
                    set_is_true = true;
                    break;
                  }
                  Some(InnerGraphMapValue::Set(item_value)) => {
                    for i in item_value {
                      if matches!(i, InnerGraphMapSetValue::TopLevel(value) if value == key) {
                        continue;
                      }
                      if already_processed.contains(i) {
                        continue;
                      }
                      new_set.insert(i.clone());
                      if matches!(i, InnerGraphMapSetValue::TopLevel(_)) {
                        is_terminal = false;
                      }
                    }
                  }
                  _ => {}
                }
              }
            }
          }
          if set_is_true {
            state.inner_graph.insert(*key, InnerGraphMapValue::True);
          } else if new_set.is_empty() {
            state.inner_graph.insert(*key, InnerGraphMapValue::Nil);
          } else {
            state
              .inner_graph
              .insert(*key, InnerGraphMapValue::Set(new_set));
          }
        }

        if is_terminal {
          keys_to_remove.push(*key);
          // We use `""` to represent global_key
          if key.is_global() {
            let global_value = state.inner_graph.get(&TopLevelSymbol::global()).cloned();
            if let Some(global_value) = global_value {
              for (key, value) in state.inner_graph.iter_mut() {
                if !key.is_global() && value != &InnerGraphMapValue::True {
                  if global_value == InnerGraphMapValue::True {
                    *value = InnerGraphMapValue::True;
                  } else {
                    let mut new_set = match value {
                      InnerGraphMapValue::Set(set) => std::mem::take(set),
                      InnerGraphMapValue::True => unreachable!(),
                      InnerGraphMapValue::Nil => HashSet::default(),
                    };
                    let extend_value = match global_value.clone() {
                      InnerGraphMapValue::Set(set) => set,
                      InnerGraphMapValue::True => unreachable!(),
                      InnerGraphMapValue::Nil => HashSet::default(),
                    };
                    new_set.extend(extend_value);
                    *value = InnerGraphMapValue::Set(new_set);
                  }
                }
              }
            }
          }
        }
      }
      // Work around for rustc borrow rules
      for k in keys_to_remove {
        non_terminal.remove(&k);
      }
    }

    let mut finalized = vec![];
    for (symbol, cbs) in state.usage_map.drain() {
      let usage = state.inner_graph.get(&symbol);
      let used_by_exports = if let Some(usage) = usage {
        match usage {
          InnerGraphMapValue::Set(set) => {
            let finalized_set = HashSet::from_iter(
              set
                .iter()
                .map(|item| item.to_atom(&state.symbol_map).clone()),
            );
            UsedByExports::Set(finalized_set)
          }
          InnerGraphMapValue::True => UsedByExports::Bool(true),
          InnerGraphMapValue::Nil => UsedByExports::Bool(false),
        }
      } else {
        UsedByExports::Bool(false)
      };
      for cb in cbs {
        finalized.push((cb, used_by_exports.clone()));
      }
    }

    finalized
  }

  pub fn add_variable_usage(parser: &mut JavascriptParser, name: &Atom, usage: InnerGraphMapUsage) {
    let symbol = parser
      .get_tag_data(name, TOP_LEVEL_SYMBOL)
      .map(TopLevelSymbol::downcast)
      .unwrap_or_else(|| Self::tag_top_level_symbol(parser, name));

    parser.inner_graph.add_usage(symbol, usage);
  }

  pub fn on_usage(parser: &mut JavascriptParser, operation: InnerGraphUsageOperation) {
    if parser.inner_graph.is_enabled()
      && let Some(symbol) = parser.inner_graph.get_top_level_symbol()
    {
      parser
        .inner_graph
        .usage_map
        .entry(symbol)
        .or_default()
        .push(operation);
      // When inner graph is enabled but no top-level symbol, the expression is always used,
      // so we skip adding PureExpressionDependency (same as UsedByExports::Bool(true))
    }
    // When inner graph is disabled, we skip adding PureExpressionDependency (same as None)
  }

  pub fn tag_top_level_symbol(
    parser: &mut crate::visitors::JavascriptParser,
    name: &Atom,
  ) -> TopLevelSymbol {
    parser.define_variable(name.clone());

    let existing = parser.get_variable_info(name);
    if let Some(existing) = existing
      && let Some(tag_info) = existing.tag_info
      && let tag_info = parser.definitions_db.expect_get_mut_tag_info(tag_info)
      && tag_info.tag == TOP_LEVEL_SYMBOL
      && let Some(tag_data) = tag_info.data.clone()
    {
      return TopLevelSymbol::downcast(tag_data);
    }

    let symbol = parser.inner_graph.new_top_level_symbol(name.clone());
    parser.tag_variable_with_flags(
      name.clone(),
      TOP_LEVEL_SYMBOL,
      Some(symbol),
      VariableInfoFlags::NORMAL,
    );
    symbol
  }
}

impl JavascriptParserPlugin for InnerGraphParserPlugin {
  fn program(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    _ast: &swc_core::ecma::ast::Program,
  ) -> Option<bool> {
    parser.inner_graph.enable();

    None
  }

  fn finish(&self, parser: &mut JavascriptParser) -> Option<bool> {
    if !parser.inner_graph.is_enabled() {
      return None;
    }

    None
  }

  fn pre_statement(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    stmt: Statement,
  ) -> Option<bool> {
    if !parser.inner_graph.is_enabled() {
      return None;
    }

    if parser.is_top_level_scope()
      && let Some(fn_decl) = stmt.as_function_decl()
    {
      let name = &fn_decl
        .ident()
        .map(|ident| ident.sym.clone())
        .unwrap_or_else(|| DEFAULT_STAR_JS_WORD.clone());
      let fn_variable = Self::tag_top_level_symbol(parser, name);

      parser
        .inner_graph
        .statement_with_top_level_symbol
        .insert(stmt.span(), fn_variable);

      return Some(true);
    }

    None
  }

  fn block_pre_statement(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    stmt: Statement,
  ) -> Option<bool> {
    if !parser.inner_graph.is_enabled() || !parser.is_top_level_scope() {
      return None;
    }

    if let Some(class_decl) = stmt.as_class_decl()
      && is_pure_class(
        parser,
        self.analyze_pure_annotation,
        class_decl.class(),
        self.unresolved_context,
        parser.comments,
        None,
      )
    {
      let name = &class_decl
        .ident()
        .map(|ident| ident.sym.clone())
        .unwrap_or_else(|| DEFAULT_STAR_JS_WORD.clone());
      let class_variable = Self::tag_top_level_symbol(parser, name);
      parser
        .inner_graph
        .class_with_top_level_symbol
        .insert(stmt.span(), class_variable);
      return Some(true);
    }

    None
  }

  fn block_pre_module_declaration(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    export_decl: &ModuleDecl,
  ) -> Option<bool> {
    if !parser.inner_graph.is_enabled() || !parser.is_top_level_scope() {
      return None;
    }

    if let ModuleDecl::ExportDefaultDecl(export_default_decl) = export_decl {
      let decl = &export_default_decl.decl;

      if let DefaultDecl::Class(class_expr) = decl
        && is_pure_class(
          parser,
          self.analyze_pure_annotation,
          &class_expr.class,
          self.unresolved_context,
          parser.comments,
          None,
        )
      {
        let variable = Self::tag_top_level_symbol(parser, &DEFAULT_STAR_JS_WORD);
        parser
          .inner_graph
          .class_with_top_level_symbol
          .insert(decl.span(), variable);
      } else if let DefaultDecl::Fn(fn_expr) = decl
        && is_pure_function(
          parser,
          self.analyze_pure_annotation,
          &fn_expr.function,
          self.unresolved_context,
          parser.comments,
          None,
        )
      {
        let variable = Self::tag_top_level_symbol(parser, &DEFAULT_STAR_JS_WORD);
        parser
          .inner_graph
          .statement_with_top_level_symbol
          .insert(export_decl.span(), variable);
      }
    }

    // Webpack using estree types, which treats all `export default ...` as ExportDefaultDeclaration type
    // https://github.com/estree/estree/blob/master/es2015.md#exportdefaultdeclaration
    // but SWC using ExportDefaultExpr to represent `export default 1`
    let mut callees = vec![];
    if let ModuleDecl::ExportDefaultExpr(ExportDefaultExpr { expr, .. }) = export_decl
      && is_pure_expression(
        parser,
        self.analyze_pure_annotation,
        expr,
        self.unresolved_context,
        parser.comments,
        Some(&mut callees),
      )
    {
      let export_part = &**expr;
      let variable = Self::tag_top_level_symbol(parser, &DEFAULT_STAR_JS_WORD);

      for (name, span) in callees {
        variable.add_depend_on(&mut parser.inner_graph, name, span);
      }

      let export_span = export_decl.span();
      parser
        .inner_graph
        .statement_with_top_level_symbol
        .insert(export_span, variable);

      if !export_part.is_fn_expr() && !export_part.is_arrow() && !export_part.is_lit() {
        parser
          .inner_graph
          .statement_pure_part
          .insert(export_span, expr.span());
      }
    }

    None
  }

  fn pre_declarator(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    decl: &VarDeclarator,
    _stmt: VariableDeclaration<'_>,
  ) -> Option<bool> {
    if !parser.inner_graph.is_enabled() || !parser.is_top_level_scope() {
      return None;
    }

    if let Pat::Ident(ident) = &decl.name
      && let Some(init) = &decl.init
    {
      let name = &ident.id.sym;
      let mut callees = vec![];

      if init.is_class()
        && is_pure_class(
          parser,
          self.analyze_pure_annotation,
          &init.as_class().expect("should be class").class,
          self.unresolved_context,
          parser.comments,
          None,
        )
      {
        let v = Self::tag_top_level_symbol(parser, name);

        parser
          .inner_graph
          .class_with_top_level_symbol
          .insert(init.span(), v);
      } else if is_pure_expression(
        parser,
        self.analyze_pure_annotation,
        init,
        self.unresolved_context,
        parser.comments,
        Some(&mut callees),
      ) {
        let v = Self::tag_top_level_symbol(parser, name);
        for (symbol, span) in callees {
          v.add_depend_on(&mut parser.inner_graph, symbol, span);
        }

        parser
          .inner_graph
          .decl_with_top_level_symbol
          .insert(decl.span(), v);

        if !init.is_fn_expr() && !init.is_arrow() && !init.is_lit() {
          parser.inner_graph.pure_declarators.insert(decl.span());
        }
      }
    }

    None
  }

  fn statement(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    stmt: Statement,
  ) -> Option<bool> {
    if !parser.inner_graph.is_enabled() || !parser.is_top_level_scope() {
      return None;
    }

    parser.inner_graph.set_top_level_symbol(None);

    Self::for_each_statement(parser, &stmt.span());

    None
  }

  fn module_declaration(&self, parser: &mut JavascriptParser, stmt: &ModuleDecl) -> Option<bool> {
    if !parser.inner_graph.is_enabled() || !parser.is_top_level_scope() {
      return None;
    }

    parser.inner_graph.set_top_level_symbol(None);
    let stmt_span = stmt.span();

    if let Some(v) = parser
      .inner_graph
      .statement_with_top_level_symbol
      .get(&stmt_span)
    {
      parser.inner_graph.set_top_level_symbol(Some(*v));

      if let Some(pure_part) = parser.inner_graph.statement_pure_part.get(&stmt_span) {
        let pure_part_start = pure_part.real_lo();
        let pure_part_end = pure_part.real_hi();
        let dep = PureExpressionDependency::new(
          DependencyRange::new(pure_part_start, pure_part_end),
          *parser.module_identifier,
        );
        let dep_id = *dep.id();
        parser.add_dependency(Box::new(dep));
        Self::on_usage(parser, InnerGraphUsageOperation::PureExpression(dep_id));
      }
    }

    if let ModuleDecl::ExportDefaultDecl(default_decl) = stmt {
      match &default_decl.decl {
        DefaultDecl::Class(class) => {
          Self::for_each_statement(parser, &class.span());
        }
        DefaultDecl::Fn(f) => {
          Self::for_each_statement(parser, &f.span());
        }
        DefaultDecl::TsInterfaceDecl(_) => unreachable!(),
      }
    }

    None
  }

  fn class_extends_expression(
    &self,
    parser: &mut JavascriptParser,
    super_class: &Expr,
    class_decl_or_expr: crate::visitors::ClassDeclOrExpr,
  ) -> Option<bool> {
    if !parser.inner_graph.is_enabled() || !parser.is_top_level_scope() {
      return None;
    }

    let is_pure_super_class = is_pure_expression(
      parser,
      self.analyze_pure_annotation,
      super_class,
      self.unresolved_context,
      parser.comments,
      None,
    );

    if let Some(v) = parser
      .inner_graph
      .class_with_top_level_symbol
      .get(&class_decl_or_expr.span())
      && is_pure_super_class
    {
      parser.inner_graph.set_top_level_symbol(Some(*v));

      let expr_span = super_class.span();

      let dep = PureExpressionDependency::new(
        DependencyRange::new(expr_span.real_lo(), expr_span.real_hi()),
        *parser.module_identifier,
      );
      let dep_id = *dep.id();
      parser.add_dependency(Box::new(dep));
      Self::on_usage(parser, InnerGraphUsageOperation::PureExpression(dep_id));
    }

    None
  }

  fn class_body_element(
    &self,
    parser: &mut JavascriptParser,
    element: &ClassMember,
    class_decl_or_expr: crate::visitors::ClassDeclOrExpr,
  ) -> Option<bool> {
    if !parser.inner_graph.is_enabled() || !parser.is_top_level_scope() {
      return None;
    }
    if let Some(top_level_symbol) = parser
      .inner_graph
      .class_with_top_level_symbol
      .get(&class_decl_or_expr.span())
    {
      let top_level_symbol_variable_name = parser
        .inner_graph
        .top_level_symbol(top_level_symbol)
        .name
        .clone();
      parser.inner_graph.set_top_level_symbol(None);
      /*
       * ```js
       * var A = class B {
       *   static {
       *     this;
       *     B;
       *   }
       * }
       * ```
       * Alias `this` and `B` (class ident) to top level symbol `A` here, so `A` is used if `this` or `B`
       * is used in static block (`add_usage` in identifier hook and this hook), even `A` is not used in
       * any other place.
       */
      if let ClassMember::StaticBlock(_) = element {
        let class_var = parser
          .get_variable_info(&top_level_symbol_variable_name)
          .map(|info| ExportedVariableInfo::VariableInfo(info.id()))
          .unwrap_or(ExportedVariableInfo::Name(top_level_symbol_variable_name));
        if let Some(class_ident) = class_decl_or_expr.ident() {
          parser.set_variable(class_ident.sym.clone(), class_var.clone());
        }
        parser.set_variable("this".into(), class_var);
      }
    }

    None
  }

  fn class_body_value(
    &self,
    parser: &mut JavascriptParser,
    element: &swc_core::ecma::ast::ClassMember,
    expr_span: Span,
    class_decl_or_expr: crate::visitors::ClassDeclOrExpr,
  ) -> Option<bool> {
    if !parser.inner_graph.is_enabled() || !parser.is_top_level_scope() {
      return None;
    }
    let pure_member = is_pure_class_member(
      parser,
      self.analyze_pure_annotation,
      element,
      self.unresolved_context,
      parser.comments,
      None,
    );
    if let Some(v) = parser
      .inner_graph
      .class_with_top_level_symbol
      .get(&class_decl_or_expr.span())
    {
      if !element.is_static() || pure_member {
        parser.inner_graph.set_top_level_symbol(Some(*v));
        if !matches!(element, ClassMember::Method(_)) && element.is_static() {
          let dep = PureExpressionDependency::new(
            DependencyRange::new(expr_span.real_lo(), expr_span.real_hi()),
            *parser.module_identifier,
          );
          let dep_id = *dep.id();
          parser.add_dependency(Box::new(dep));
          Self::on_usage(parser, InnerGraphUsageOperation::PureExpression(dep_id));
        }
      } else {
        parser.inner_graph.set_top_level_symbol(None);
      }
    }

    None
  }

  fn declarator(
    &self,
    parser: &mut JavascriptParser,
    decl: &swc_core::ecma::ast::VarDeclarator,
    _stmt: VariableDeclaration<'_>,
  ) -> Option<bool> {
    if !parser.inner_graph.is_enabled() {
      return None;
    }

    if let Some(v) = parser
      .inner_graph
      .decl_with_top_level_symbol
      .get(&decl.span())
    {
      parser.inner_graph.set_top_level_symbol(Some(*v));

      if parser.inner_graph.pure_declarators.contains(&decl.span) {
        // class Foo extends Bar {}
        // if Foo is not used, we can ignore extends Bar
        if let Some(init) = &decl.init
          && let Expr::Class(class_expr) = init.as_ref()
        {
          let super_span = class_expr.class.super_class.span();
          let dep = PureExpressionDependency::new(
            DependencyRange::new(super_span.real_lo(), super_span.real_hi()),
            *parser.module_identifier,
          );
          let dep_id = *dep.id();
          parser.add_dependency(Box::new(dep));
          Self::on_usage(parser, InnerGraphUsageOperation::PureExpression(dep_id));
        } else if decl.init.is_none() || !decl.init.as_ref().expect("unreachable").is_class() {
          let init = decl.init.as_ref().expect("should have initialization");
          let init_span = init.span();
          let dep = PureExpressionDependency::new(
            DependencyRange::new(init_span.real_lo(), init_span.real_hi()),
            *parser.module_identifier,
          );
          let dep_id = *dep.id();
          parser.add_dependency(Box::new(dep));
          InnerGraphParserPlugin::on_usage(
            parser,
            InnerGraphUsageOperation::PureExpression(dep_id),
          );
        }
      }

      parser.walk_expression(decl.init.as_ref().expect("should have initialization"));
      parser.inner_graph.set_top_level_symbol(None);
      return Some(true);
    } else if decl.name.is_ident()
      && let Some(init) = &decl.init
      && init.is_class()
      && parser
        .inner_graph
        .class_with_top_level_symbol
        .contains_key(&init.span())
    {
      parser.walk_expression(init);
      parser.inner_graph.set_top_level_symbol(None);
      return Some(true);
    }

    None
  }

  fn member(
    &self,
    parser: &mut JavascriptParser,
    _expr: &swc_core::ecma::ast::MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    Self::for_each_expression(parser, for_name);
    None
  }

  fn assign(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::AssignExpr,
    for_name: &str,
  ) -> Option<bool> {
    if !parser.inner_graph.is_enabled() || for_name != TOP_LEVEL_SYMBOL {
      return None;
    }
    if matches!(expr.op, AssignOp::Assign) {
      return Some(true);
    }
    None
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    _ident: &swc_core::ecma::ast::Ident,
    for_name: &str,
  ) -> Option<bool> {
    Self::for_each_expression(parser, for_name);
    None
  }

  fn this(
    &self,
    parser: &mut JavascriptParser,
    _expr: &swc_core::ecma::ast::ThisExpr,
    for_name: &str,
  ) -> Option<bool> {
    Self::for_each_expression(parser, for_name);
    None
  }
}
