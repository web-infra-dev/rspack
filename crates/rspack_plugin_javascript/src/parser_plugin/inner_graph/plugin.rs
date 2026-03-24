use std::sync::atomic::{AtomicU32, Ordering};

use rspack_core::UsedByExports;
use rspack_util::SpanExt;
use rustc_hash::FxHashMap as HashMap;
use smallvec::SmallVec;
use swc_core::{
  atoms::Atom,
  common::{Mark, Span, Spanned, SyntaxContext},
  ecma::ast::{
    AssignOp, ClassMember, DefaultDecl, ExportDefaultExpr, Expr, ModuleDecl, Pat, VarDeclarator,
  },
};

use super::state::InnerGraphUsageOperation;
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

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum InnerGraphMapSetValue {
  TopLevel(TopLevelSymbol),
  Str(Atom),
}

/// You need to make sure that InnerGraphMapUsage is not a  [InnerGraphMapUsage::True] variant
impl From<InnerGraphMapUsage> for InnerGraphMapSetValue {
  fn from(value: InnerGraphMapUsage) -> Self {
    match value {
      InnerGraphMapUsage::TopLevel(str) => Self::TopLevel(str),
      InnerGraphMapUsage::Value(str) => Self::Str(str),
      InnerGraphMapUsage::True => unreachable!(""),
    }
  }
}

impl InnerGraphMapSetValue {
  pub(crate) fn to_atom(&self) -> &Atom {
    match self {
      InnerGraphMapSetValue::TopLevel(v) => &v.name,
      InnerGraphMapSetValue::Str(v) => v,
    }
  }
}

#[derive(Default, Clone, Debug)]
pub struct InnerGraphSet(SmallVec<[InnerGraphMapSetValue; 4]>);

impl InnerGraphSet {
  pub fn with_capacity(capacity: usize) -> Self {
    Self(SmallVec::with_capacity(capacity))
  }

  pub fn with_item(value: InnerGraphMapSetValue) -> Self {
    let mut set = Self::with_capacity(1);
    set.insert(value);
    set
  }

  pub fn insert(&mut self, value: InnerGraphMapSetValue) -> bool {
    if self.0.contains(&value) {
      return false;
    }
    self.0.push(value);
    true
  }

  pub fn contains(&self, value: &InnerGraphMapSetValue) -> bool {
    self.0.contains(value)
  }

  pub fn extend<I>(&mut self, values: I)
  where
    I: IntoIterator<Item = InnerGraphMapSetValue>,
  {
    for value in values {
      self.insert(value);
    }
  }

  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  pub fn iter(&self) -> std::slice::Iter<'_, InnerGraphMapSetValue> {
    self.0.iter()
  }
}

impl PartialEq for InnerGraphSet {
  fn eq(&self, other: &Self) -> bool {
    self.0.len() == other.0.len() && self.0.iter().all(|value| other.contains(value))
  }
}

impl Eq for InnerGraphSet {}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub enum InnerGraphMapValue {
  Set(InnerGraphSet),
  True,
  #[default]
  Nil,
}

#[derive(PartialEq, Eq, Debug)]
pub enum InnerGraphMapUsage {
  TopLevel(TopLevelSymbol),
  Value(Atom),
  True,
}

pub struct InnerGraphPlugin {
  unresolved_context: SyntaxContext,
}

pub static TOP_LEVEL_SYMBOL: &str = "inner graph top level symbol";
static TOP_LEVEL_SYMBOL_ID: AtomicU32 = AtomicU32::new(1);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) struct TopLevelSymbol {
  id: u32,
  name: Atom,
}

impl TopLevelSymbol {
  pub fn new(name: Atom) -> Self {
    Self {
      name,
      id: TOP_LEVEL_SYMBOL_ID.fetch_add(1, Ordering::Relaxed),
    }
  }

  pub fn global() -> Self {
    Self {
      name: Atom::from(""),
      id: 0,
    }
  }

  fn is_global(&self) -> bool {
    self.name.is_empty() && self.id == 0
  }
}

impl InnerGraphPlugin {
  fn cached_top_level_symbol(parser: &JavascriptParser, name: &Atom) -> Option<TopLevelSymbol> {
    parser
      .inner_graph
      .top_level_symbol_by_name
      .get(name)
      .cloned()
  }

  fn get_top_level_symbol(parser: &mut JavascriptParser, name: &Atom) -> Option<TopLevelSymbol> {
    if let Some(symbol) = Self::cached_top_level_symbol(parser, name) {
      return Some(symbol);
    }

    let symbol = parser
      .get_tag_data(name, TOP_LEVEL_SYMBOL)
      .map(TopLevelSymbol::downcast)?;
    parser
      .inner_graph
      .top_level_symbol_by_name
      .insert(name.clone(), symbol.clone());
    Some(symbol)
  }

  pub fn new(unresolved_mark: Mark) -> Self {
    Self {
      unresolved_context: SyntaxContext::empty().apply_mark(unresolved_mark),
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
      parser.inner_graph.set_top_level_symbol(Some(v.clone()));

      if let Some(pure_part) = parser.inner_graph.statement_pure_part.get(stmt_span) {
        let pure_part_start = pure_part.real_lo();
        let pure_part_end = pure_part.real_hi();
        Self::on_usage(
          parser,
          InnerGraphUsageOperation::PureExpression((pure_part_start, pure_part_end).into()),
        );
      }
    }
  }

  pub fn infer_dependency_usage(parser: &mut JavascriptParser) {
    // fun will reference it self
    if !parser.inner_graph.is_enabled() {
      return;
    }
    let state: &mut super::state::InnerGraphState = &mut parser.inner_graph;
    let mut non_terminal: Vec<TopLevelSymbol> = state.inner_graph.keys().cloned().collect();
    let mut processed =
      HashMap::with_capacity_and_hasher(state.inner_graph.len(), Default::default());

    while !non_terminal.is_empty() {
      let mut remaining = Vec::with_capacity(non_terminal.len());
      for key in non_terminal.drain(..) {
        // Using enum to manipulate original is pretty hard, so I use an extra variable to
        // flagging the new set has changed to boolean `true`
        // you could refer https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/optimize/InnerGraph.js#L150
        let mut is_terminal = true;
        let next_value = if let Some(InnerGraphMapValue::Set(names)) = state.inner_graph.get(&key) {
          let mut new_set = InnerGraphSet::with_capacity(names.iter().len());
          let mut set_is_true = false;
          let already_processed = processed
            .entry(key.clone())
            .or_insert_with(|| InnerGraphSet::with_capacity(names.iter().len()));

          for name in names.iter() {
            already_processed.insert(name.clone());
          }

          for name in names.iter() {
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
                    for i in item_value.iter() {
                      if matches!(i, InnerGraphMapSetValue::TopLevel(value) if value == &key) {
                        continue;
                      }
                      if already_processed.contains(i) {
                        continue;
                      }
                      if new_set.insert(i.clone())
                        && matches!(i, InnerGraphMapSetValue::TopLevel(_))
                      {
                        is_terminal = false;
                      }
                    }
                  }
                  _ => {}
                }
              }
            }
          }
          Some(if set_is_true {
            InnerGraphMapValue::True
          } else if new_set.is_empty() {
            InnerGraphMapValue::Nil
          } else {
            InnerGraphMapValue::Set(new_set)
          })
        } else {
          None
        };

        if let Some(next_value) = next_value
          && let Some(value) = state.inner_graph.get_mut(&key)
        {
          *value = next_value;
        }

        if is_terminal {
          // We use `""` to represent global_key
          if key.is_global() {
            let global_value = state.inner_graph.get(&key).cloned();
            if let Some(global_value) = global_value {
              for (key, value) in state.inner_graph.iter_mut() {
                if key.is_global() || value == &InnerGraphMapValue::True {
                  continue;
                }
                match (&global_value, value) {
                  (InnerGraphMapValue::True, value) => {
                    *value = InnerGraphMapValue::True;
                  }
                  (InnerGraphMapValue::Set(global_set), InnerGraphMapValue::Set(set)) => {
                    set.extend(global_set.iter().cloned());
                  }
                  (InnerGraphMapValue::Set(global_set), value @ InnerGraphMapValue::Nil) => {
                    *value = InnerGraphMapValue::Set(global_set.clone());
                  }
                  (InnerGraphMapValue::Nil, value @ InnerGraphMapValue::Nil) => {
                    *value = InnerGraphMapValue::Set(InnerGraphSet::default());
                  }
                  (InnerGraphMapValue::Nil, InnerGraphMapValue::Set(_)) => {}
                  (_, InnerGraphMapValue::True) => unreachable!(),
                }
              }
            }
          }
        } else {
          remaining.push(key);
        }
      }
      non_terminal = remaining;
    }

    let total_usage_callbacks = state.usage_map.values().map(Vec::len).sum();
    let mut finalized = Vec::with_capacity(total_usage_callbacks);
    for (symbol, cbs) in state.usage_map.drain() {
      let usage = state.inner_graph.get(&symbol);
      let used_by_exports = if let Some(usage) = usage {
        match usage {
          InnerGraphMapValue::Set(set) => {
            let finalized_set = set.iter().map(|item| item.to_atom().clone()).collect();
            UsedByExports::Set(finalized_set)
          }
          InnerGraphMapValue::True => UsedByExports::Bool(true),
          InnerGraphMapValue::Nil => UsedByExports::Bool(false),
        }
      } else {
        UsedByExports::Bool(false)
      };

      let cbs_len = cbs.len();
      let mut used_by_exports = Some(used_by_exports);
      for (index, cb) in cbs.into_iter().enumerate() {
        let used_by_exports = if index + 1 == cbs_len {
          used_by_exports
            .take()
            .expect("last callback should keep the original used_by_exports")
        } else {
          used_by_exports
            .as_ref()
            .expect("used_by_exports should still be available")
            .clone()
        };
        finalized.push((cb, used_by_exports));
      }
    }

    for (op, used_by_exports) in finalized {
      match op {
        InnerGraphUsageOperation::PureExpression(range) => {
          // Only create dependency when the expression is conditionally used
          if !matches!(used_by_exports, UsedByExports::Bool(true)) {
            let mut dep = PureExpressionDependency::new(range, *parser.module_identifier);
            dep.set_used_by_exports(Some(used_by_exports));
            parser.add_dependency(Box::new(dep));
          }
        }
        InnerGraphUsageOperation::ESMImportSpecifier(dep_idx) => {
          if let Some(dep) = parser.get_dependency_mut(dep_idx)
            && let Some(dep) = dep.downcast_mut::<crate::dependency::ESMImportSpecifierDependency>()
          {
            dep.set_used_by_exports(Some(used_by_exports));
          }
        }
        InnerGraphUsageOperation::URLDependency(dep_idx) => {
          if let Some(dep) = parser.get_dependency_mut(dep_idx)
            && let Some(dep) = dep.downcast_mut::<crate::dependency::URLDependency>()
          {
            dep.set_used_by_exports(Some(used_by_exports));
          }
        }
      }
    }
  }

  pub fn add_variable_usage(parser: &mut JavascriptParser, name: &Atom, usage: InnerGraphMapUsage) {
    let symbol = Self::get_top_level_symbol(parser, name)
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
    if let Some(symbol) = Self::cached_top_level_symbol(parser, name) {
      return symbol;
    }

    parser.define_variable(name.clone());

    let symbol = TopLevelSymbol::new(name.clone());
    parser.tag_variable_with_flags(
      name.clone(),
      TOP_LEVEL_SYMBOL,
      Some(symbol.clone()),
      VariableInfoFlags::NORMAL,
    );
    parser
      .inner_graph
      .top_level_symbol_by_name
      .insert(name.clone(), symbol.clone());
    symbol
  }
}

#[cfg(test)]
mod tests {
  use swc_core::atoms::Atom;

  use super::{InnerGraphMapSetValue, InnerGraphSet, TopLevelSymbol};

  #[test]
  fn inner_graph_set_preserves_set_semantics() {
    let symbol = TopLevelSymbol::new(Atom::from("symbol"));
    let first = InnerGraphMapSetValue::Str(Atom::from("a"));
    let second = InnerGraphMapSetValue::TopLevel(symbol);

    let mut left = InnerGraphSet::default();
    assert!(left.insert(first.clone()));
    assert!(left.insert(second.clone()));
    assert!(!left.insert(first.clone()));

    let mut right = InnerGraphSet::default();
    assert!(right.insert(second));
    assert!(right.insert(first));

    assert_eq!(left, right);
  }
}

#[rspack_macros::implemented_javascript_parser_hooks]
impl JavascriptParserPlugin for InnerGraphPlugin {
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

    Self::infer_dependency_usage(parser);

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
        .map_or_else(|| DEFAULT_STAR_JS_WORD.clone(), |ident| ident.sym.clone());
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
        class_decl.class(),
        self.unresolved_context,
        parser.comments,
      )
    {
      let name = &class_decl
        .ident()
        .map_or_else(|| DEFAULT_STAR_JS_WORD.clone(), |ident| ident.sym.clone());
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
          &class_expr.class,
          self.unresolved_context,
          parser.comments,
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
          &fn_expr.function,
          self.unresolved_context,
          parser.comments,
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
    if let ModuleDecl::ExportDefaultExpr(ExportDefaultExpr { expr, .. }) = export_decl
      && is_pure_expression(parser, expr, self.unresolved_context, parser.comments)
    {
      let export_part = &**expr;
      let variable = Self::tag_top_level_symbol(parser, &DEFAULT_STAR_JS_WORD);
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

      if init.is_class()
        && is_pure_class(
          parser,
          &init.as_class().expect("should be class").class,
          self.unresolved_context,
          parser.comments,
        )
      {
        let v = Self::tag_top_level_symbol(parser, name);

        parser
          .inner_graph
          .class_with_top_level_symbol
          .insert(init.span(), v);
      } else if is_pure_expression(parser, init, self.unresolved_context, parser.comments) {
        let v = Self::tag_top_level_symbol(parser, name);
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
      parser.inner_graph.set_top_level_symbol(Some(v.clone()));

      if let Some(pure_part) = parser.inner_graph.statement_pure_part.get(&stmt_span) {
        let pure_part_start = pure_part.real_lo();
        let pure_part_end = pure_part.real_hi();
        Self::on_usage(
          parser,
          InnerGraphUsageOperation::PureExpression((pure_part_start, pure_part_end).into()),
        );
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
      super_class,
      self.unresolved_context,
      parser.comments,
    );

    if let Some(v) = parser
      .inner_graph
      .class_with_top_level_symbol
      .get(&class_decl_or_expr.span())
      && is_pure_super_class
    {
      parser.inner_graph.set_top_level_symbol(Some(v.clone()));

      let expr_span = super_class.span();

      Self::on_usage(
        parser,
        InnerGraphUsageOperation::PureExpression(expr_span.into()),
      );
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
      let top_level_symbol_variable_name = top_level_symbol.name.clone();
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
    let pure_member =
      is_pure_class_member(parser, element, self.unresolved_context, parser.comments);
    if let Some(v) = parser
      .inner_graph
      .class_with_top_level_symbol
      .get(&class_decl_or_expr.span())
    {
      if !element.is_static() || pure_member {
        parser.inner_graph.set_top_level_symbol(Some(v.clone()));
        if !matches!(element, ClassMember::Method(_)) && element.is_static() {
          Self::on_usage(
            parser,
            InnerGraphUsageOperation::PureExpression(expr_span.into()),
          );
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
      parser.inner_graph.set_top_level_symbol(Some(v.clone()));

      if parser.inner_graph.pure_declarators.contains(&decl.span) {
        // class Foo extends Bar {}
        // if Foo is not used, we can ignore extends Bar
        if let Some(init) = &decl.init
          && let Expr::Class(class_expr) = init.as_ref()
        {
          let super_span = class_expr.class.super_class.span();

          InnerGraphPlugin::on_usage(
            parser,
            InnerGraphUsageOperation::PureExpression(super_span.into()),
          );
        } else if decl.init.is_none() || !decl.init.as_ref().expect("unreachable").is_class() {
          let init = decl.init.as_ref().expect("should have initialization");
          let init_span = init.span();

          InnerGraphPlugin::on_usage(
            parser,
            InnerGraphUsageOperation::PureExpression(init_span.into()),
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
