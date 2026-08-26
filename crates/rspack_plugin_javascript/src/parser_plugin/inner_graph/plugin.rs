use rspack_core::{
  BoxDependency, Dependency, DependencyId, DependencyRange, UsedByExports,
  UsedByExportsDeferredPureCheck,
};
use rspack_util::SpanExt;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_next_ecma_ast::{
  AssignmentExpression, AssignmentOperator, ClassElement, ClassElementData,
  ExportDefaultDeclarationKindData, Expr, ExprData, GetSpan, Program, Span, Stmt, StmtData,
  ThisExpression, VariableDeclarator,
};

use super::state::{
  InnerGraphMapSetValue, InnerGraphMapUsage, InnerGraphMapValue, InnerGraphState,
  InnerGraphUsageOperation, TopLevelSymbol,
};
use crate::{
  Atom,
  dependency::{ESMImportSpecifierDependency, PureExpressionDependency, URLDependency},
  parser_plugin::{DEFAULT_STAR_JS_WORD, JavascriptParserPlugin},
  side_effects_parser_plugin::{
    is_pure_class, is_pure_class_member, is_pure_expression, is_pure_function,
  },
  visitors::{
    ExportedVariableInfo, HookMemberExpression, Identifier, JavascriptParser, Statement,
    TagInfoData, VariableDeclaration, scope_info::VariableInfoFlags,
  },
};

fn class_member_is_static(parser: &JavascriptParser, member: ClassElement) -> bool {
  let ast = parser.ast.ast;
  match ast.class_element_data(member) {
    ClassElementData::MethodDefinition(method) => method.r#static(ast),
    ClassElementData::PropertyDefinition(property) => property.r#static(ast),
    ClassElementData::StaticBlock(_) => true,
    ClassElementData::TsMethodDefinition(method) => method.r#static(ast),
    ClassElementData::TsIndexSignature(_) => false,
  }
}

#[derive(Debug)]
pub struct InnerGraphParserPlugin {
  analyze_pure_annotation: bool,
}

pub static TOP_LEVEL_SYMBOL: &str = "inner graph top level symbol";

impl InnerGraphParserPlugin {
  pub fn new(analyze_pure_annotation: bool) -> Self {
    Self {
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
        let pure_part: &Span = pure_part;
        let pure_part_start = pure_part.real_lo();
        let pure_part_end = pure_part.real_hi();
        let dep = PureExpressionDependency::new(
          DependencyRange::new(pure_part_start, pure_part_end),
          *parser.module_identifier,
        );
        let dep_idx = parser.next_dependency_idx();
        parser.add_dependency(BoxDependency::new(dep));
        Self::on_usage(parser, InnerGraphUsageOperation::PureExpression(dep_idx));
      }
    }
  }

  pub fn infer_dependency_usage(
    state: &mut InnerGraphState,
    deferred_pure_checks_by_symbol: &[Vec<UsedByExportsDeferredPureCheck>],
  ) -> Vec<(InnerGraphUsageOperation, UsedByExports)> {
    let symbol_count = state.symbol_count();
    let mut non_terminal = vec![false; symbol_count];
    let mut remaining = 0;
    for symbol in state.graph_symbols() {
      non_terminal[symbol.index()] = true;
      remaining += 1;
    }
    let mut processed = (0..symbol_count)
      .map(|_| HashSet::default())
      .collect::<Vec<HashSet<InnerGraphMapSetValue>>>();

    while remaining != 0 {
      let mut keys_to_remove = vec![];
      for (index, is_non_terminal) in non_terminal.iter().copied().enumerate() {
        if !is_non_terminal {
          continue;
        }
        let key = TopLevelSymbol::from_index(index);
        let mut new_set = HashSet::default();
        // Using enum to manipulate original is pretty hard, so I use an extra variable to
        // flagging the new set has changed to boolean `true`
        // you could refer https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/optimize/InnerGraph.js#L150
        let mut set_is_true = false;
        let mut is_terminal = true;
        let already_processed = &mut processed[index];
        if matches!(state.graph(key), Some(InnerGraphMapValue::Set(_))) {
          let Some(InnerGraphMapValue::Set(names)) = state.take_graph(key) else {
            unreachable!("checked Set value before removing inner graph entry")
          };
          already_processed.extend(names.iter().cloned());
          for name in names {
            match name {
              InnerGraphMapSetValue::Str(v) => {
                new_set.insert(InnerGraphMapSetValue::Str(v));
              }
              InnerGraphMapSetValue::TopLevel(dep_symbol) => {
                if dep_symbol == key {
                  continue;
                }
                if !deferred_pure_checks_by_symbol[dep_symbol.index()].is_empty() {
                  new_set.insert(InnerGraphMapSetValue::TopLevel(dep_symbol));
                }
                match state.graph(dep_symbol) {
                  Some(InnerGraphMapValue::True) => {
                    set_is_true = true;
                    break;
                  }
                  Some(InnerGraphMapValue::Set(item_value)) => {
                    for i in item_value {
                      if matches!(i, InnerGraphMapSetValue::TopLevel(value) if *value == key) {
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
            state.set_graph(key, InnerGraphMapValue::True);
          } else if new_set.is_empty() {
            state.set_graph(key, InnerGraphMapValue::Nil);
          } else {
            state.set_graph(key, InnerGraphMapValue::Set(new_set));
          }
        }

        if is_terminal {
          keys_to_remove.push(key);
          // We use `""` to represent global_key
          if key.is_global() {
            let global_value = state.graph(TopLevelSymbol::global()).cloned();
            if let Some(global_value) = global_value {
              for index in 1..symbol_count {
                let symbol = TopLevelSymbol::from_index(index);
                let Some(value) = state.graph_mut(symbol) else {
                  continue;
                };
                if value != &InnerGraphMapValue::True {
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
        if std::mem::replace(&mut non_terminal[k.index()], false) {
          remaining -= 1;
        }
      }
    }

    let mut finalized = vec![];
    for index in 0..symbol_count {
      let symbol = TopLevelSymbol::from_index(index);
      let cbs = state.take_usage_operations(symbol);
      if cbs.is_empty() {
        continue;
      }
      let mut deferred_pure_checks = deferred_pure_checks_by_symbol[index].clone();
      let usage = state.graph(symbol);
      let used_by_exports = if let Some(usage) = usage {
        match usage {
          InnerGraphMapValue::Set(set) => {
            let mut finalized_set = HashSet::default();
            for item in set {
              match item {
                InnerGraphMapSetValue::TopLevel(dep_symbol) => {
                  let checks = &deferred_pure_checks_by_symbol[dep_symbol.index()];
                  if !checks.is_empty() {
                    deferred_pure_checks.extend(checks.iter().cloned());
                  }
                }
                InnerGraphMapSetValue::Str(export_name) => {
                  finalized_set.insert(export_name.clone());
                }
              }
            }
            UsedByExports::set(finalized_set)
          }
          InnerGraphMapValue::True => UsedByExports::bool(true),
          InnerGraphMapValue::Nil => UsedByExports::bool(false),
        }
      } else {
        UsedByExports::bool(false)
      }
      .with_deferred_pure_checks(deferred_pure_checks);
      for cb in cbs {
        finalized.push((cb, used_by_exports.clone()));
      }
    }

    finalized
  }

  pub fn finalize_dependency_usage(
    state: &mut InnerGraphState,
    dependencies: &mut [BoxDependency],
  ) {
    if !state.is_enabled() || !state.has_usage_operations() {
      return;
    }

    let mut deferred_pure_checks_by_symbol = vec![Vec::new(); state.symbol_count()];
    if state
      .symbols()
      .map(|(_, symbol_data)| symbol_data)
      .any(|symbol_data| !symbol_data.depend_on_pure.is_empty())
    {
      let mut dep_by_span: HashMap<(u32, u32), (DependencyId, Atom)> = dependencies
        .iter()
        .filter_map(|dep| {
          let specifier_dep = dep.downcast_ref::<ESMImportSpecifierDependency>()?;
          let range = specifier_dep.range()?;
          Some((
            (range.start, range.end),
            (*dep.id(), specifier_dep.imported_name().clone()),
          ))
        })
        .collect();

      let mut always_used_symbols = Vec::new();

      for (symbol, symbol_data) in state.symbols() {
        // A single UsedByExports edge cannot safely describe a pure
        // expression whose purity depends on multiple imported callees. Keep
        // the expression conservative until the dependency model can encode
        // the combined condition.
        if symbol_data.depend_on_pure.len() > 1 {
          always_used_symbols.push(symbol);
          continue;
        }
        let mut deferred_pure_checks = Vec::new();
        for (_name, span) in &symbol_data.depend_on_pure {
          if let Some((dep_id, import_name)) = dep_by_span.remove(&(span.real_lo(), span.real_hi()))
          {
            deferred_pure_checks.push(UsedByExportsDeferredPureCheck {
              dep_id,
              atom: import_name,
            });
          } else {
            always_used_symbols.push(symbol);
            deferred_pure_checks.clear();
            break;
          }
        }

        if !deferred_pure_checks.is_empty() {
          deferred_pure_checks_by_symbol[symbol.index()] = deferred_pure_checks;
        }
      }

      for symbol in always_used_symbols {
        state.set_graph(symbol, InnerGraphMapValue::True);
        deferred_pure_checks_by_symbol[symbol.index()].clear();
      }
    }

    for (operation, used_by_exports) in
      Self::infer_dependency_usage(state, &deferred_pure_checks_by_symbol)
    {
      let dep_idx = match operation {
        InnerGraphUsageOperation::PureExpression(dep_idx)
        | InnerGraphUsageOperation::ESMImportSpecifier(dep_idx)
        | InnerGraphUsageOperation::URLDependency(dep_idx) => dep_idx,
      };
      let Some(dep) = dependencies.get_mut(dep_idx) else {
        continue;
      };
      match operation {
        InnerGraphUsageOperation::PureExpression(_) => {
          if let Some(dep) = dep.downcast_mut::<PureExpressionDependency>() {
            dep.set_used_by_exports(Some(used_by_exports));
          }
        }
        InnerGraphUsageOperation::ESMImportSpecifier(_) => {
          if let Some(dep) = dep.downcast_mut::<ESMImportSpecifierDependency>() {
            dep.set_used_by_exports(Some(used_by_exports));
          }
        }
        InnerGraphUsageOperation::URLDependency(_) => {
          if let Some(dep) = dep.downcast_mut::<URLDependency>() {
            dep.set_used_by_exports(Some(used_by_exports));
          }
        }
      }
    }
  }

  pub fn add_variable_usage(parser: &mut JavascriptParser, name: &Atom, usage: InnerGraphMapUsage) {
    let symbol = parser
      .get_tag_data::<TopLevelSymbol>(name, TOP_LEVEL_SYMBOL)
      .copied()
      .unwrap_or_else(|| Self::tag_top_level_symbol(parser, name));

    parser.inner_graph.add_usage(symbol, usage);
  }

  pub fn on_usage(parser: &mut JavascriptParser, operation: InnerGraphUsageOperation) {
    if parser.inner_graph.is_enabled()
      && let Some(symbol) = parser.inner_graph.get_top_level_symbol()
    {
      parser.inner_graph.add_usage_operation(symbol, operation);
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

    if let Some(existing) = parser
      .get_tag_data::<TopLevelSymbol>(name, TOP_LEVEL_SYMBOL)
      .copied()
    {
      return existing;
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

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for InnerGraphParserPlugin {
  fn program(&self, parser: &mut crate::visitors::JavascriptParser, _ast: Program) -> Option<bool> {
    parser.inner_graph.enable();

    None
  }

  fn finish(&self, parser: &mut JavascriptParser<'p>) -> Option<bool> {
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
      let ast = parser.ast.ast;
      let name = &fn_decl.ident(ast).map_or_else(
        || DEFAULT_STAR_JS_WORD.clone(),
        |ident| Atom::from(ast.get_utf8(ident.name(ast))),
      );
      let fn_variable = Self::tag_top_level_symbol(parser, name);

      parser
        .inner_graph
        .statement_with_top_level_symbol
        .insert(stmt.span(ast), fn_variable);

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
        parser.ast.comments,
        None,
      )
    {
      let ast = parser.ast.ast;
      let name = &class_decl.ident(ast).map_or_else(
        || DEFAULT_STAR_JS_WORD.clone(),
        |ident| Atom::from(ast.get_utf8(ident.name(ast))),
      );
      let class_variable = Self::tag_top_level_symbol(parser, name);
      parser
        .inner_graph
        .class_with_top_level_symbol
        .insert(stmt.span(ast), class_variable);
      return Some(true);
    }

    None
  }

  fn block_pre_module_declaration(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    export_decl: Stmt,
  ) -> Option<bool> {
    if !parser.inner_graph.is_enabled() || !parser.is_top_level_scope() {
      return None;
    }

    let ast = parser.ast.ast;
    if let StmtData::ExportDefaultDeclaration(export_default) = ast.stmt_data(export_decl) {
      let declaration = export_default.declaration(ast);

      if let ExportDefaultDeclarationKindData::Class(class) =
        ast.export_default_declaration_kind_data(declaration)
        && is_pure_class(
          parser,
          self.analyze_pure_annotation,
          class,
          parser.ast.comments,
          None,
        )
      {
        let variable = Self::tag_top_level_symbol(parser, &DEFAULT_STAR_JS_WORD);
        parser
          .inner_graph
          .class_with_top_level_symbol
          .insert(declaration.span(ast), variable);
      } else if let ExportDefaultDeclarationKindData::Function(function) =
        ast.export_default_declaration_kind_data(declaration)
        && is_pure_function(parser, function)
      {
        let variable = Self::tag_top_level_symbol(parser, &DEFAULT_STAR_JS_WORD);
        parser
          .inner_graph
          .statement_with_top_level_symbol
          .insert(export_decl.span(ast), variable);
      }
      if let ExportDefaultDeclarationKindData::Expr(export_part) =
        ast.export_default_declaration_kind_data(declaration)
      {
        let mut callees = vec![];
        if is_pure_expression(
          parser,
          self.analyze_pure_annotation,
          export_part,
          parser.ast.comments,
          Some(&mut callees),
        ) {
          let variable = Self::tag_top_level_symbol(parser, &DEFAULT_STAR_JS_WORD);
          for (name, span) in callees {
            variable.add_depend_on(&mut parser.inner_graph, name, span);
          }
          let export_span = export_decl.span(ast);
          parser
            .inner_graph
            .statement_with_top_level_symbol
            .insert(export_span, variable);
          if !matches!(
            ast.expr_data(export_part),
            ExprData::Function(_)
              | ExprData::ArrowFunctionExpression(_)
              | ExprData::StringLiteral(_)
              | ExprData::NumericLiteral(_)
              | ExprData::BigIntLiteral(_)
              | ExprData::BooleanLiteral(_)
              | ExprData::NullLiteral(_)
              | ExprData::RegExpLiteral(_)
          ) {
            parser
              .inner_graph
              .statement_pure_part
              .insert(export_span, export_part.span(ast));
          }
        }
      }
    }

    None
  }

  fn pre_declarator(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    decl: VariableDeclarator,
    _stmt: VariableDeclaration,
  ) -> Option<bool> {
    if !parser.inner_graph.is_enabled() || !parser.is_top_level_scope() {
      return None;
    }

    let ast = parser.ast.ast;
    if let Some(identifier) = decl.id(ast).as_binding_identifier(ast)
      && let Some(init) = decl.init(ast)
    {
      let name = Atom::from(ast.get_utf8(identifier.name(ast)));
      let mut callees = vec![];

      if let Some(class) = init.as_class(ast)
        && is_pure_class(
          parser,
          self.analyze_pure_annotation,
          class,
          parser.ast.comments,
          None,
        )
      {
        let v = Self::tag_top_level_symbol(parser, &name);

        parser
          .inner_graph
          .class_with_top_level_symbol
          .insert(init.span(ast), v);
      } else if !init.is_class(ast)
        && is_pure_expression(
          parser,
          self.analyze_pure_annotation,
          init,
          parser.ast.comments,
          Some(&mut callees),
        )
      {
        let v = Self::tag_top_level_symbol(parser, &name);
        for (symbol, span) in callees {
          v.add_depend_on(&mut parser.inner_graph, symbol, span);
        }

        parser
          .inner_graph
          .decl_with_top_level_symbol
          .insert(decl.span(ast), v);

        if !matches!(
          ast.expr_data(init),
          ExprData::Function(_)
            | ExprData::ArrowFunctionExpression(_)
            | ExprData::StringLiteral(_)
            | ExprData::NumericLiteral(_)
            | ExprData::BigIntLiteral(_)
            | ExprData::BooleanLiteral(_)
            | ExprData::NullLiteral(_)
            | ExprData::RegExpLiteral(_)
        ) {
          parser.inner_graph.pure_declarators.insert(decl.span(ast));
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

    Self::for_each_statement(parser, &stmt.span(parser.ast.ast));

    None
  }

  fn module_declaration(&self, parser: &mut JavascriptParser<'p>, stmt: Stmt) -> Option<bool> {
    if !parser.inner_graph.is_enabled() || !parser.is_top_level_scope() {
      return None;
    }

    parser.inner_graph.set_top_level_symbol(None);
    let ast = parser.ast.ast;
    let stmt_span = stmt.span(ast);

    if let Some(v) = parser
      .inner_graph
      .statement_with_top_level_symbol
      .get(&stmt_span)
    {
      parser.inner_graph.set_top_level_symbol(Some(*v));

      if let Some(pure_part) = parser.inner_graph.statement_pure_part.get(&stmt_span) {
        let pure_part: &Span = pure_part;
        let pure_part_start = pure_part.real_lo();
        let pure_part_end = pure_part.real_hi();
        let dep = PureExpressionDependency::new(
          DependencyRange::new(pure_part_start, pure_part_end),
          *parser.module_identifier,
        );
        let dep_idx = parser.next_dependency_idx();
        parser.add_dependency(BoxDependency::new(dep));
        Self::on_usage(parser, InnerGraphUsageOperation::PureExpression(dep_idx));
      }
    }

    if let StmtData::ExportDefaultDeclaration(default_decl) = ast.stmt_data(stmt) {
      match ast.export_default_declaration_kind_data(default_decl.declaration(ast)) {
        ExportDefaultDeclarationKindData::Class(class) => {
          Self::for_each_statement(parser, &class.span(ast));
        }
        ExportDefaultDeclarationKindData::Function(function) => {
          Self::for_each_statement(parser, &function.span(ast));
        }
        _ => {}
      }
    }

    None
  }

  fn class_extends_expression(
    &self,
    parser: &mut JavascriptParser<'p>,
    super_class: Expr,
    class_decl_or_expr: crate::visitors::ClassDeclOrExpr,
  ) -> Option<bool> {
    if !parser.inner_graph.is_enabled() || !parser.is_top_level_scope() {
      return None;
    }

    let is_pure_super_class = is_pure_expression(
      parser,
      self.analyze_pure_annotation,
      super_class,
      parser.ast.comments,
      None,
    );

    if let Some(v) = parser
      .inner_graph
      .class_with_top_level_symbol
      .get(&class_decl_or_expr.span(parser.ast.ast))
      && is_pure_super_class
    {
      parser.inner_graph.set_top_level_symbol(Some(*v));

      let expr_span = super_class.span(parser.ast.ast);

      let dep = PureExpressionDependency::new(
        DependencyRange::new(expr_span.real_lo(), expr_span.real_hi()),
        *parser.module_identifier,
      );
      let dep_idx = parser.next_dependency_idx();
      parser.add_dependency(BoxDependency::new(dep));
      Self::on_usage(parser, InnerGraphUsageOperation::PureExpression(dep_idx));
    }

    None
  }

  fn class_body_element(
    &self,
    parser: &mut JavascriptParser<'p>,
    element: ClassElement,
    class_decl_or_expr: crate::visitors::ClassDeclOrExpr,
  ) -> Option<bool> {
    if !parser.inner_graph.is_enabled() || !parser.is_top_level_scope() {
      return None;
    }
    if let Some(top_level_symbol) = parser
      .inner_graph
      .class_with_top_level_symbol
      .get(&class_decl_or_expr.span(parser.ast.ast))
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
      if matches!(
        parser.ast.ast.class_element_data(element),
        ClassElementData::StaticBlock(_)
      ) {
        let class_var = parser
          .get_variable_info(&top_level_symbol_variable_name)
          .map(|info| ExportedVariableInfo::VariableInfo(info.id()))
          .unwrap_or(ExportedVariableInfo::Name(top_level_symbol_variable_name));
        if let Some(class_ident) = class_decl_or_expr.ident(parser.ast.ast) {
          parser.set_variable(
            Atom::from(parser.ast.ast.get_utf8(class_ident.name(parser.ast.ast))),
            class_var.clone(),
          );
        }
        parser.set_variable("this".into(), class_var);
      }
    }

    None
  }

  fn class_body_value(
    &self,
    parser: &mut JavascriptParser<'p>,
    element: ClassElement,
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
      parser.ast.comments,
      None,
    );
    if let Some(v) = parser
      .inner_graph
      .class_with_top_level_symbol
      .get(&class_decl_or_expr.span(parser.ast.ast))
    {
      if !class_member_is_static(parser, element) || pure_member {
        parser.inner_graph.set_top_level_symbol(Some(*v));
        if !matches!(
          parser.ast.ast.class_element_data(element),
          ClassElementData::MethodDefinition(_) | ClassElementData::TsMethodDefinition(_)
        ) && class_member_is_static(parser, element)
        {
          let dep = PureExpressionDependency::new(
            DependencyRange::new(expr_span.real_lo(), expr_span.real_hi()),
            *parser.module_identifier,
          );
          let dep_idx = parser.next_dependency_idx();
          parser.add_dependency(BoxDependency::new(dep));
          Self::on_usage(parser, InnerGraphUsageOperation::PureExpression(dep_idx));
        }
      } else {
        parser.inner_graph.set_top_level_symbol(None);
      }
    }

    None
  }

  fn declarator(
    &self,
    parser: &mut JavascriptParser<'p>,
    decl: VariableDeclarator,
    _stmt: VariableDeclaration,
  ) -> Option<bool> {
    if !parser.inner_graph.is_enabled() {
      return None;
    }

    if let Some(v) = parser
      .inner_graph
      .decl_with_top_level_symbol
      .get(&decl.span(parser.ast.ast))
    {
      parser.inner_graph.set_top_level_symbol(Some(*v));

      let ast = parser.ast.ast;
      if parser
        .inner_graph
        .pure_declarators
        .contains(&decl.span(ast))
      {
        // class Foo extends Bar {}
        // if Foo is not used, we can ignore extends Bar
        if let Some(init) = decl.init(ast)
          && let Some(class) = init.as_class(ast)
          && let Some(super_class) = class.super_class(ast)
        {
          let super_span = super_class.span(ast);
          let dep = PureExpressionDependency::new(
            DependencyRange::new(super_span.real_lo(), super_span.real_hi()),
            *parser.module_identifier,
          );
          let dep_idx = parser.next_dependency_idx();
          parser.add_dependency(BoxDependency::new(dep));
          Self::on_usage(parser, InnerGraphUsageOperation::PureExpression(dep_idx));
        } else if let Some(init) = decl.init(ast)
          && !init.is_class(ast)
        {
          let init_span = init.span(ast);
          let dep = PureExpressionDependency::new(
            DependencyRange::new(init_span.real_lo(), init_span.real_hi()),
            *parser.module_identifier,
          );
          let dep_idx = parser.next_dependency_idx();
          parser.add_dependency(BoxDependency::new(dep));
          InnerGraphParserPlugin::on_usage(
            parser,
            InnerGraphUsageOperation::PureExpression(dep_idx),
          );
        }
      }

      parser.walk_expression(decl.init(ast).expect("should have initialization"));
      parser.inner_graph.set_top_level_symbol(None);
      return Some(true);
    } else if decl
      .id(parser.ast.ast)
      .is_binding_identifier(parser.ast.ast)
      && let Some(init) = decl.init(parser.ast.ast)
      && init.is_class(parser.ast.ast)
      && parser
        .inner_graph
        .class_with_top_level_symbol
        .contains_key(&init.span(parser.ast.ast))
    {
      parser.walk_expression(init);
      parser.inner_graph.set_top_level_symbol(None);
      return Some(true);
    }

    None
  }

  fn member(
    &self,
    parser: &mut JavascriptParser<'p>,
    _expr: HookMemberExpression,
    for_name: &str,
  ) -> Option<bool> {
    Self::for_each_expression(parser, for_name);
    None
  }

  fn assign(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: AssignmentExpression,
    _ident: &Identifier,
    for_name: &str,
  ) -> Option<bool> {
    if !parser.inner_graph.is_enabled() || for_name != TOP_LEVEL_SYMBOL {
      return None;
    }
    if expr.operator(parser.ast.ast) == AssignmentOperator::Assign {
      return Some(true);
    }
    None
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    _ident: &Identifier,
    for_name: &str,
  ) -> Option<bool> {
    Self::for_each_expression(parser, for_name);
    None
  }

  fn this(
    &self,
    parser: &mut JavascriptParser<'p>,
    _expr: ThisExpression,
    for_name: &str,
  ) -> Option<bool> {
    Self::for_each_expression(parser, for_name);
    None
  }
}
