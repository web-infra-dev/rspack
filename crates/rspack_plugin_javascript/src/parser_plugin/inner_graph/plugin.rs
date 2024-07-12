use rspack_core::{Dependency, SpanExt, UsedByExports};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_core::{
  atoms::Atom,
  common::{Mark, Span, Spanned, SyntaxContext},
  ecma::ast::{ClassMember, DefaultDecl, ExportDefaultExpr, Expr, ModuleDecl, Pat},
};

use super::state::UsageCallback;
use crate::{
  dependency::PureExpressionDependency,
  is_pure_class, is_pure_class_member, is_pure_expression, is_pure_function,
  parser_plugin::{JavascriptParserPlugin, DEFAULT_STAR_JS_WORD},
  visitors::{JavascriptParser, Statement, TagInfoData, TopLevelScope},
  ClassExt,
};

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum InnerGraphMapSetValue {
  TopLevel(Atom),
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
      InnerGraphMapSetValue::TopLevel(v) => v,
      InnerGraphMapSetValue::Str(v) => v,
    }
  }
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub enum InnerGraphMapValue {
  Set(HashSet<InnerGraphMapSetValue>),
  True,
  #[default]
  Nil,
}

#[derive(PartialEq, Eq, Debug)]
pub enum InnerGraphMapUsage {
  TopLevel(Atom),
  Value(Atom),
  True,
}

pub struct InnerGraphPlugin {
  unresolved_context: SyntaxContext,
}

pub static TOP_LEVEL_SYMBOL: &str = "inner graph top level symbol";

#[derive(Debug, Clone)]
pub(crate) struct TopLevelSymbol {
  name: Atom,
}

impl TopLevelSymbol {
  pub fn new(name: Atom) -> Self {
    Self { name }
  }
}

impl InnerGraphPlugin {
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
      let tag_info = parser.definitions_db.expect_get_tag_info(&tag_info);
      let symbol = TopLevelSymbol::downcast(tag_info.data.clone().expect("should have data"));
      let usage = parser.inner_graph.get_top_level_symbol();
      parser.inner_graph.add_usage(
        symbol.name,
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
      parser
        .inner_graph
        .set_top_level_symbol(Some(v.name.clone()));

      if let Some(pure_part) = parser.inner_graph.statement_pure_part.get(stmt_span) {
        let pure_part_start = pure_part.real_lo();
        let pure_part_end = pure_part.real_hi();
        Self::on_usage(
          parser,
          Box::new(move |parser, used_by_exports| {
            if !matches!(used_by_exports, Some(UsedByExports::Bool(true)) | None) {
              let mut dep = PureExpressionDependency::new(
                pure_part_start,
                pure_part_end,
                *parser.module_identifier,
              );
              dep.set_used_by_exports(used_by_exports);
              parser.dependencies.push(Box::new(dep));
            }
          }),
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
    let mut non_terminal = HashSet::from_iter(state.inner_graph.keys().cloned());
    let mut processed: HashMap<Atom, HashSet<InnerGraphMapSetValue>> = HashMap::default();

    while !non_terminal.is_empty() {
      let mut keys_to_remove = vec![];
      for key in non_terminal.iter() {
        let mut new_set = HashSet::default();
        // Using enum to manipulate original is pretty hard, so I use an extra variable to
        // flagging the new set has changed to boolean `true`
        // you could refer https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/optimize/InnerGraph.js#L150
        let mut set_is_true = false;
        let mut is_terminal = true;
        let already_processed = processed.entry(key.clone()).or_default();
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
            state
              .inner_graph
              .insert(key.clone(), InnerGraphMapValue::True);
          } else if new_set.is_empty() {
            state
              .inner_graph
              .insert(key.clone(), InnerGraphMapValue::Nil);
          } else {
            state
              .inner_graph
              .insert(key.clone(), InnerGraphMapValue::Set(new_set));
          }
        }

        if is_terminal {
          keys_to_remove.push(key.clone());
          // We use `""` to represent global_key
          if key == "" {
            let global_value = state.inner_graph.get(&Atom::from("")).cloned();
            if let Some(global_value) = global_value {
              for (key, value) in state.inner_graph.iter_mut() {
                if key != "" && value != &InnerGraphMapValue::True {
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
    for (symbol, cbs) in state.usage_callback_map.drain() {
      let usage = state.inner_graph.get(&symbol);
      let used_by_exports = if let Some(usage) = usage {
        match usage {
          InnerGraphMapValue::Set(set) => {
            let finalized_set = HashSet::from_iter(set.iter().map(|item| item.to_atom().clone()));
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

    for (cb, used_by_exports) in finalized {
      cb(parser, Some(used_by_exports));
    }
  }

  pub fn add_variable_usage(parser: &mut JavascriptParser, name: &Atom, usage: InnerGraphMapUsage) {
    let symbol = parser
      .get_tag_data(name, TOP_LEVEL_SYMBOL)
      .map(TopLevelSymbol::downcast)
      .unwrap_or_else(|| Self::tag_top_level_symbol(parser, name));

    parser.inner_graph.add_usage(symbol.name, usage);
  }

  pub fn on_usage(parser: &mut JavascriptParser, on_usage_callback: UsageCallback) {
    if parser.inner_graph.is_enabled() {
      if let Some(symbol) = parser.inner_graph.get_top_level_symbol() {
        parser
          .inner_graph
          .usage_callback_map
          .entry(symbol)
          .or_default()
          .push(on_usage_callback);
      } else {
        on_usage_callback(parser, Some(UsedByExports::Bool(true)));
      }
    } else {
      on_usage_callback(parser, None);
    }
  }

  pub fn tag_top_level_symbol(
    parser: &mut crate::visitors::JavascriptParser,
    name: &Atom,
  ) -> TopLevelSymbol {
    parser.define_variable(name.to_string());

    let existing = parser.get_variable_info(name);
    if let Some(existing) = existing
      && let Some(tag_info) = existing.tag_info
      && let tag_info = parser.definitions_db.expect_get_mut_tag_info(&tag_info)
      && tag_info.tag == TOP_LEVEL_SYMBOL
      && let Some(tag_data) = tag_info.data.clone()
    {
      return TopLevelSymbol::downcast(tag_data);
    }

    let symbol = TopLevelSymbol::new(name.clone());
    parser.tag_variable(name.to_string(), TOP_LEVEL_SYMBOL, Some(symbol.clone()));
    symbol
  }
}

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

    if matches!(parser.top_level_scope, TopLevelScope::Top)
      && let Some(fn_decl) = stmt.as_function_decl()
    {
      let name = &fn_decl
        .ident
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
    if !parser.inner_graph.is_enabled() || !matches!(parser.top_level_scope, TopLevelScope::Top) {
      return None;
    }

    if let Some(class_decl) = stmt.as_class_decl()
      && is_pure_class(class_decl.class, self.unresolved_context, parser.comments)
    {
      let name = &class_decl
        .ident
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
    if !parser.inner_graph.is_enabled() || !matches!(parser.top_level_scope, TopLevelScope::Top) {
      return None;
    }

    if let ModuleDecl::ExportDefaultDecl(export_default_decl) = export_decl {
      let decl = &export_default_decl.decl;

      if let DefaultDecl::Class(class_expr) = decl
        && is_pure_class(&class_expr.class, self.unresolved_context, parser.comments)
      {
        let variable = Self::tag_top_level_symbol(parser, &DEFAULT_STAR_JS_WORD);
        parser
          .inner_graph
          .class_with_top_level_symbol
          .insert(decl.span(), variable);
      } else if let DefaultDecl::Fn(fn_expr) = decl
        && is_pure_function(&fn_expr.function, self.unresolved_context, parser.comments)
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
      && is_pure_expression(expr, self.unresolved_context, parser.comments)
    {
      let export_part = expr.unwrap_parens();
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
    decl: &swc_core::ecma::ast::VarDeclarator,
    _stmt: &swc_core::ecma::ast::VarDecl,
  ) -> Option<bool> {
    if !parser.inner_graph.is_enabled() || !matches!(parser.top_level_scope, TopLevelScope::Top) {
      return None;
    }

    if let Pat::Ident(ident) = &decl.name
      && let Some(init) = &decl.init
    {
      let name = &ident.id.sym;

      let unwrapped_init = init.unwrap_parens();
      if unwrapped_init.is_class()
        && is_pure_class(
          &unwrapped_init.as_class().expect("should be class").class,
          self.unresolved_context,
          parser.comments,
        )
      {
        let v = Self::tag_top_level_symbol(parser, name);

        parser
          .inner_graph
          .class_with_top_level_symbol
          .insert(init.span(), v);
      } else if is_pure_expression(init, self.unresolved_context, parser.comments) {
        let v = Self::tag_top_level_symbol(parser, name);
        parser
          .inner_graph
          .decl_with_top_level_symbol
          .insert(decl.span(), v);

        if !unwrapped_init.is_fn_expr() && !unwrapped_init.is_arrow() && !unwrapped_init.is_lit() {
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
    if !parser.inner_graph.is_enabled() || !matches!(parser.top_level_scope, TopLevelScope::Top) {
      return None;
    }

    parser.inner_graph.set_top_level_symbol(None);

    Self::for_each_statement(parser, &stmt.span());

    None
  }

  fn module_declaration(&self, parser: &mut JavascriptParser, stmt: &ModuleDecl) -> Option<bool> {
    if !parser.inner_graph.is_enabled() || !matches!(parser.top_level_scope, TopLevelScope::Top) {
      return None;
    }

    parser.inner_graph.set_top_level_symbol(None);
    let stmt_span = stmt.span();

    if let Some(v) = parser
      .inner_graph
      .statement_with_top_level_symbol
      .get(&stmt_span)
    {
      let v = v.clone();
      parser.inner_graph.set_top_level_symbol(Some(v.name));

      if let Some(pure_part) = parser.inner_graph.statement_pure_part.get(&stmt_span) {
        let pure_part_start = pure_part.real_lo();
        let pure_part_end = pure_part.real_hi();
        Self::on_usage(
          parser,
          Box::new(move |parser, used_by_exports| {
            if !matches!(used_by_exports, Some(UsedByExports::Bool(true)) | None) {
              let mut dep = PureExpressionDependency::new(
                pure_part_start,
                pure_part_end,
                *parser.module_identifier,
              );
              dep.set_used_by_exports(used_by_exports);
              parser.dependencies.push(Box::new(dep));
            }
          }),
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
    if !parser.inner_graph.is_enabled() || !matches!(parser.top_level_scope, TopLevelScope::Top) {
      return None;
    }

    if let Some(v) = parser
      .inner_graph
      .class_with_top_level_symbol
      .get(&class_decl_or_expr.span())
      && is_pure_expression(super_class, self.unresolved_context, parser.comments)
    {
      parser
        .inner_graph
        .set_top_level_symbol(Some(v.name.clone()));

      let expr_span = super_class.span();

      Self::on_usage(
        parser,
        Box::new(move |parser, used_by_exports| {
          if !matches!(used_by_exports, Some(UsedByExports::Bool(true)) | None) {
            let mut dep = PureExpressionDependency::new(
              expr_span.real_lo(),
              expr_span.real_hi(),
              *parser.module_identifier,
            );
            dep.set_used_by_exports(used_by_exports);
            parser.dependencies.push(Box::new(dep));
          }
        }),
      );
    }

    None
  }

  fn class_body_element(
    &self,
    parser: &mut JavascriptParser,
    _element: &swc_core::ecma::ast::ClassMember,
    class_decl_or_expr: crate::visitors::ClassDeclOrExpr,
  ) -> Option<bool> {
    if !parser.inner_graph.is_enabled() || !matches!(parser.top_level_scope, TopLevelScope::Top) {
      return None;
    }

    if parser
      .inner_graph
      .class_with_top_level_symbol
      .contains_key(&class_decl_or_expr.span())
    {
      parser.inner_graph.set_top_level_symbol(None);
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
    if !parser.inner_graph.is_enabled() || !matches!(parser.top_level_scope, TopLevelScope::Top) {
      return None;
    }
    if let Some(v) = parser
      .inner_graph
      .class_with_top_level_symbol
      .get(&class_decl_or_expr.span())
    {
      if !element.is_static()
        || is_pure_class_member(element, self.unresolved_context, parser.comments)
      {
        let atom = v.name.clone();
        parser.inner_graph.set_top_level_symbol(Some(atom));
        if !matches!(element, ClassMember::Method(_)) && element.is_static() {
          Self::on_usage(
            parser,
            Box::new(move |parser, used_by_exports| {
              if !matches!(used_by_exports, Some(UsedByExports::Bool(true)) | None) {
                let mut dep = PureExpressionDependency::new(
                  expr_span.real_lo(),
                  expr_span.real_hi(),
                  *parser.module_identifier,
                );
                dep.set_used_by_exports(used_by_exports);
                parser.dependencies.push(Box::new(dep));
              }
            }),
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
    _stmt: &swc_core::ecma::ast::VarDecl,
  ) -> Option<bool> {
    if !parser.inner_graph.is_enabled() {
      return None;
    }

    if let Some(v) = parser
      .inner_graph
      .decl_with_top_level_symbol
      .get(&decl.span())
    {
      parser
        .inner_graph
        .set_top_level_symbol(Some(v.name.clone()));

      if parser.inner_graph.pure_declarators.contains(&decl.span) {
        // class Foo extends Bar {}
        // if Foo is not used, we can ignore extends Bar
        if let Some(init) = &decl.init
          && let Expr::Class(class_expr) = init.as_ref()
        {
          let super_span = class_expr.class.super_class.span();

          InnerGraphPlugin::on_usage(
            parser,
            Box::new(move |parser, used_by_exports| {
              if !matches!(used_by_exports, Some(UsedByExports::Bool(true)) | None) {
                let mut dep = PureExpressionDependency::new(
                  super_span.real_lo(),
                  super_span.real_hi(),
                  *parser.module_identifier,
                );
                dep.set_used_by_exports(used_by_exports);
                parser.dependencies.push(Box::new(dep));
              }
            }),
          );
        } else if decl.init.is_none() || !decl.init.as_ref().expect("unreachable").is_class() {
          let init = decl.init.as_ref().expect("should have initialization");
          let init_span = init.span();

          InnerGraphPlugin::on_usage(
            parser,
            Box::new(move |parser, used_by_exports| {
              if !matches!(used_by_exports, Some(UsedByExports::Bool(true)) | None) {
                let mut dep = PureExpressionDependency::new(
                  init_span.real_lo(),
                  init_span.real_hi(),
                  *parser.module_identifier,
                );
                dep.set_used_by_exports(used_by_exports);
                parser.dependencies.push(Box::new(dep));
              }
            }),
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

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    _ident: &swc_core::ecma::ast::Ident,
    for_name: &str,
  ) -> Option<bool> {
    Self::for_each_expression(parser, for_name);
    None
  }
}
