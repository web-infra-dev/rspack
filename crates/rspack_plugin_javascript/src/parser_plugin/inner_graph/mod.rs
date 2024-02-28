mod state;
mod top_level_symbol;

use std::ops::Deref;

use rspack_core::{SpanExt, UsedByExports};
use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::atoms::Atom;
use swc_core::common::Spanned;
use swc_core::ecma::ast::{Expr, Ident};

use self::state::UsageCallback;
pub use self::state::{InnerGraphMapSetValue, InnerGraphState};
pub use self::state::{InnerGraphMapUsage, InnerGraphMapValue};
pub use self::top_level_symbol::TopLevelSymbol;
use super::JavascriptParserPlugin;
use crate::dependency::PureExpressionDependency;
use crate::visitors::{JavascriptParser, TagInfoData};
use crate::ClassExt;

const TOP_LEVEL_SYMBOL_TAG: &str = "_identifier__top_level_symbol_tag__";
pub(super) const INNER_GRAPH_DEFAULT_MARK: &str = "*default*";

#[inline]
fn in_top_level(parser: &JavascriptParser) -> bool {
  matches!(parser.top_level_scope, crate::visitors::TopLevelScope::Top)
}

impl JavascriptParser<'_> {
  pub fn add_variable_usage_to_inner_graph(&mut self, name: &Atom, usage: InnerGraphMapUsage) {
    let symbol = self
      .get_tag_data(name.as_str(), TOP_LEVEL_SYMBOL_TAG)
      .map(|value| TopLevelSymbol::deserialize(value.clone()))
      .or_else(|| self.tag_top_level_symbol(name));
    if let Some(symbol) = symbol {
      self.inner_graph_state.add_usage(symbol.0, usage);
    }
  }

  fn tag_top_level_symbol(&mut self, name: &Atom) -> Option<TopLevelSymbol> {
    if !self.inner_graph_state.is_enabled() {
      return None;
    }

    self.define_variable(name.as_str().to_string());

    let res = if let Some(existing_tag) = self.get_tag_data(name.as_str(), TOP_LEVEL_SYMBOL_TAG) {
      TopLevelSymbol::deserialize(existing_tag.clone())
    } else {
      let f = TopLevelSymbol(name.clone());
      self.tag_variable(name.to_string(), TOP_LEVEL_SYMBOL_TAG, Some(f.clone()));
      f
    };
    Some(res)
  }

  fn expect_tag_top_level_symbol(&mut self, name: &Atom) -> TopLevelSymbol {
    assert!(self.inner_graph_state.is_enabled());
    self
      .tag_top_level_symbol(name)
      .expect("tag top level symbol must success if inner graph is enabled")
  }

  pub fn on_usage(&mut self, on_usage_callback: UsageCallback) {
    if self.inner_graph_state.is_enabled() {
      if let Some(symbol) = self.inner_graph_state.get_top_level_symbol() {
        self
          .inner_graph_state
          .usage_callback_map
          .entry(symbol.clone())
          .or_default()
          .push(on_usage_callback);
      } else {
        on_usage_callback(&mut self.dependencies, Some(UsedByExports::Bool(true)));
      }
    } else {
      on_usage_callback(&mut self.dependencies, None);
    }
  }

  fn on_usage_super(&mut self, super_class: &Expr) {
    let module_identifier = *self.module_identifier;
    let span = super_class.span();
    self.on_usage(Box::new(
      move |deps, used_by_exports| match used_by_exports {
        Some(UsedByExports::Bool(false)) | Some(UsedByExports::Set(_)) => {
          let mut dep =
            PureExpressionDependency::new(span.real_lo(), span.real_hi(), module_identifier);
          dep.used_by_exports = used_by_exports;
          deps.push(Box::new(dep));
        }
        Some(UsedByExports::Bool(true)) | None => (),
      },
    ));
  }

  fn infer_dependency_usage(&mut self) {
    if !self.inner_graph_state.is_enabled() {
      return;
    }

    let mut non_terminal = FxHashSet::from_iter(self.inner_graph_state.inner_graph.keys().cloned());
    let mut processed: FxHashMap<Atom, FxHashSet<InnerGraphMapSetValue>> = FxHashMap::default();

    while !non_terminal.is_empty() {
      let mut keys_to_remove = vec![];
      for key in non_terminal.iter() {
        let mut new_set = FxHashSet::default();
        // Using enum to manipulate original is pretty hard, so I use an extra variable to
        // flagging the new set has changed to boolean `true`
        // you could refer https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/optimize/InnerGraph.js#L150
        let mut set_is_true = false;
        let mut is_terminal = true;
        let already_processed = processed.entry(key.clone()).or_default();
        if let Some(InnerGraphMapValue::Set(names)) = self.inner_graph_state.inner_graph.get(key) {
          for name in names.iter() {
            already_processed.insert(name.clone());
          }
          for name in names {
            match name {
              InnerGraphMapSetValue::Str(v) => {
                new_set.insert(InnerGraphMapSetValue::Str(v.clone()));
              }
              InnerGraphMapSetValue::TopLevel(v) => {
                let item_value = self.inner_graph_state.inner_graph.get(v);
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
            self
              .inner_graph_state
              .inner_graph
              .insert(key.clone(), InnerGraphMapValue::True);
          } else if new_set.is_empty() {
            self
              .inner_graph_state
              .inner_graph
              .insert(key.clone(), InnerGraphMapValue::Nil);
          } else {
            self
              .inner_graph_state
              .inner_graph
              .insert(key.clone(), InnerGraphMapValue::Set(new_set));
          }
        }

        if is_terminal {
          keys_to_remove.push(key.clone());
          // We use `""` to represent global_key
          if key == "" {
            let global_value = self
              .inner_graph_state
              .inner_graph
              .get(&Atom::from(""))
              .cloned();
            if let Some(global_value) = global_value {
              for (key, value) in self.inner_graph_state.inner_graph.iter_mut() {
                if key != "" && value != &InnerGraphMapValue::True {
                  if global_value == InnerGraphMapValue::True {
                    *value = InnerGraphMapValue::True;
                  } else {
                    let mut new_set = match value {
                      InnerGraphMapValue::Set(set) => std::mem::take(set),
                      InnerGraphMapValue::True => unreachable!(),
                      InnerGraphMapValue::Nil => FxHashSet::default(),
                    };
                    let extend_value = match global_value.clone() {
                      InnerGraphMapValue::Set(set) => set,
                      InnerGraphMapValue::True => unreachable!(),
                      InnerGraphMapValue::Nil => FxHashSet::default(),
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

    for (symbol, cbs) in self.inner_graph_state.usage_callback_map.iter() {
      let usage = self.inner_graph_state.inner_graph.get(&symbol.0);
      for cb in cbs {
        let used_by_exports = if let Some(usage) = usage {
          match usage {
            InnerGraphMapValue::Set(set) => {
              let finalized_set =
                FxHashSet::from_iter(set.iter().map(|item| item.to_atom().clone()));
              UsedByExports::Set(finalized_set)
            }
            InnerGraphMapValue::True => UsedByExports::Bool(true),
            InnerGraphMapValue::Nil => UsedByExports::Bool(false),
          }
        } else {
          UsedByExports::Bool(false)
        };

        cb(&mut self.dependencies, Some(used_by_exports));
      }
    }
  }
}

pub struct InnerGraphPlugin;

impl JavascriptParserPlugin for InnerGraphPlugin {
  fn program(
    &self,
    parser: &mut JavascriptParser,
    _ast: &swc_core::ecma::ast::Program,
  ) -> Option<bool> {
    parser.inner_graph_state.enable();
    None
  }

  fn finish(
    &self,
    parser: &mut JavascriptParser,
    _ast: &swc_core::ecma::ast::Program,
  ) -> Option<bool> {
    if !parser.inner_graph_state.is_enabled() {
      return None;
    }
    parser.infer_dependency_usage();
    None
  }

  fn pre_statement(
    &self,
    parser: &mut JavascriptParser,
    stmt: &swc_core::ecma::ast::Stmt,
  ) -> Option<bool> {
    if !parser.inner_graph_state.is_enabled() || !in_top_level(parser) {
      None
    } else if let Some(fn_decl) = stmt.as_decl().and_then(|decl| decl.as_fn_decl()) {
      let f = parser.expect_tag_top_level_symbol(&fn_decl.ident.sym);
      parser
        .inner_graph_state
        .insert_statement_with_top_level_symbol_by_stmt(stmt, f);
      Some(true)
    } else {
      None
    }
  }

  fn block_pre_walk_export_default_declaration(
    &self,
    parser: &mut JavascriptParser,
    decl: &swc_core::ecma::ast::ExportDefaultDecl,
  ) -> Option<bool> {
    if !parser.inner_graph_state.is_enabled() || !in_top_level(parser) {
      None
    } else if let Some(fn_decl) = decl.decl.as_fn_expr() {
      let f = if let Some(ident) = &fn_decl.ident {
        parser.expect_tag_top_level_symbol(&ident.sym)
      } else {
        parser.expect_tag_top_level_symbol(&Atom::new(INNER_GRAPH_DEFAULT_MARK))
      };
      parser
        .inner_graph_state
        .insert_statement_with_top_level_symbol_by_export_default_decl(decl, f);
      Some(true)
    } else if let Some(class_decl) = decl.decl.as_class()
      && parser.is_pure_class(&class_decl.class, decl.span_lo())
    {
      let f = if let Some(ident) = &class_decl.ident {
        parser.expect_tag_top_level_symbol(&ident.sym)
      } else {
        parser.expect_tag_top_level_symbol(&Atom::new(INNER_GRAPH_DEFAULT_MARK))
      };
      parser
        .inner_graph_state
        .insert_class_with_top_level_symbol(&class_decl.class, f);
      Some(true)
    } else {
      None
    }
  }

  fn block_pre_statement(
    &self,
    parser: &mut JavascriptParser,
    stmt: &swc_core::ecma::ast::Stmt,
  ) -> Option<bool> {
    if !parser.inner_graph_state.is_enabled() || !in_top_level(parser) {
      None
    } else if let Some(class_decl) = stmt.as_decl().and_then(|decl| decl.as_class())
      && parser.is_pure_stmt(stmt, stmt.span().lo())
    {
      let name = &class_decl.ident.sym;
      let f = parser.expect_tag_top_level_symbol(name);
      parser
        .inner_graph_state
        .insert_class_with_top_level_symbol(&class_decl.class, f);
      Some(true)
    } else {
      None
    }
  }

  fn block_pre_module_declration(
    &self,
    parser: &mut JavascriptParser,
    decl: &swc_core::ecma::ast::ModuleDecl,
  ) -> Option<bool> {
    if !parser.inner_graph_state.is_enabled() || !in_top_level(parser) {
      return None;
    }

    if let Some(export_default_decl) = decl.as_export_default_decl() {
      let f = parser.expect_tag_top_level_symbol(&Atom::new(INNER_GRAPH_DEFAULT_MARK));
      let default_decl = &export_default_decl.decl;
      if let Some(class) = default_decl.as_class()
        && parser.is_pure_class(&class.class, default_decl.span().lo)
      {
        parser
          .inner_graph_state
          .insert_class_with_top_level_symbol(&class.class, f);
      } else if
      /* fn_expr is pure */
      default_decl.is_fn_expr() {
        parser
          .inner_graph_state
          .insert_statement_with_top_level_symbol_by_decl(decl, f);
      }
    } else if let Some(expr) = decl.as_export_default_expr() {
      let f = parser.expect_tag_top_level_symbol(&Atom::new(INNER_GRAPH_DEFAULT_MARK));
      if parser.is_pure_expr(&expr.expr, decl.span_lo()) {
        parser
          .inner_graph_state
          .insert_statement_with_top_level_symbol_by_decl(decl, f);
        if !expr.expr.is_lit() && !expr.expr.is_fn_expr() {
          parser
            .inner_graph_state
            .insert_statement_pure_part_by_decl(decl, expr.expr.span());
        }
      }
    }
    None
  }

  fn pre_declarator(
    &self,
    parser: &mut JavascriptParser,
    decl: &swc_core::ecma::ast::VarDeclarator,
  ) -> Option<bool> {
    if !parser.inner_graph_state.is_enabled() || !in_top_level(parser) {
      return None;
    }

    if let Some(init) = &decl.init
      && let Some(id) = decl.name.as_ident()
    {
      let init_is_pure = parser.is_pure_expr(init, id.span_hi());
      if let Some(class) = init.as_class()
        && init_is_pure
      {
        let f = parser.expect_tag_top_level_symbol(&id.id.sym);
        parser
          .inner_graph_state
          .insert_class_with_top_level_symbol(&class.class, f);
      } else if init_is_pure {
        let f = parser.expect_tag_top_level_symbol(&id.id.sym);
        parser
          .inner_graph_state
          .insert_decl_with_top_level_symbol(decl, f);
        if !init.is_fn_expr() && !init.is_lit() && !init.is_arrow() {
          parser.inner_graph_state.insert_pure_declarator(decl);
        }
      }
    }

    None
  }

  fn statement(
    &self,
    parser: &mut JavascriptParser,
    stmt: &swc_core::ecma::ast::Stmt,
  ) -> Option<bool> {
    if !parser.inner_graph_state.is_enabled() || !in_top_level(parser) {
      return None;
    }

    parser.inner_graph_state.set_top_level_symbol(None);
    if let Some(f) = parser
      .inner_graph_state
      .get_statement_with_top_level_symbol_by_stmt(stmt)
      .cloned()
    {
      parser.inner_graph_state.set_top_level_symbol(Some(f));
    }
    None
  }

  fn module_declaration(
    &self,
    parser: &mut JavascriptParser,
    decl: &swc_core::ecma::ast::ModuleDecl,
  ) -> Option<bool> {
    if !parser.inner_graph_state.is_enabled() || !in_top_level(parser) {
      return None;
    }

    parser.inner_graph_state.set_top_level_symbol(None);
    if let Some(f) = parser
      .inner_graph_state
      .get_statement_with_top_level_symbol_by_decl(decl)
      .cloned()
    {
      parser.inner_graph_state.set_top_level_symbol(Some(f));
      if let Some(pure_part) = parser
        .inner_graph_state
        .get_statement_pure_part_by_decl(decl)
      {
        let module_identifier = *parser.module_identifier;
        parser.on_usage(Box::new(
          move |deps, used_by_exports| match used_by_exports {
            Some(UsedByExports::Bool(false)) | Some(UsedByExports::Set(_)) => {
              let mut dep = PureExpressionDependency::new(
                pure_part.real_lo(),
                pure_part.real_hi(),
                module_identifier,
              );
              dep.used_by_exports = used_by_exports;
              deps.push(Box::new(dep))
            }
            Some(UsedByExports::Bool(true)) | None => (),
          },
        ));
      }
    }

    None
  }

  fn class_extends_expression(
    &self,
    parser: &mut JavascriptParser,
    super_class: &swc_core::ecma::ast::Expr,
    classy: &swc_core::ecma::ast::Class,
    ident: Option<&Ident>,
  ) -> Option<bool> {
    if !parser.inner_graph_state.is_enabled() || !in_top_level(parser) {
      return None;
    }
    let pos = ident
      .map(|ident| ident.span_hi())
      .unwrap_or(classy.span_lo());
    if parser.is_pure_class(classy, pos)
      && let Some(f) = parser
        .inner_graph_state
        .get_class_with_top_level_symbol(classy)
        .cloned()
    {
      parser.inner_graph_state.set_top_level_symbol(Some(f));
      parser.on_usage_super(super_class);
    }
    None
  }

  fn class_body_element(
    &self,
    parser: &mut JavascriptParser,
    _element: &swc_core::ecma::ast::ClassMember,
    classy: &swc_core::ecma::ast::Class,
  ) -> Option<bool> {
    if !parser.inner_graph_state.is_enabled() || !in_top_level(parser) {
      return None;
    }
    if parser
      .inner_graph_state
      .get_class_with_top_level_symbol(classy)
      .is_some()
    {
      parser.inner_graph_state.set_top_level_symbol(None)
    }
    None
  }

  fn class_body_body(
    &self,
    parser: &mut JavascriptParser,
    body: &swc_core::ecma::ast::BlockStmt,
    element: &swc_core::ecma::ast::ClassMember,
    classy: &swc_core::ecma::ast::Class,
  ) -> Option<bool> {
    if !parser.inner_graph_state.is_enabled() || !in_top_level(parser) {
      return None;
    }
    if let Some(f) = parser
      .inner_graph_state
      .get_class_with_top_level_symbol(classy)
      .cloned()
      && (!element.is_static()
        || parser.is_pure_class(
          classy,
          element
            .class_key()
            .map(|key| key.span_hi())
            .unwrap_or(element.span_lo()),
        ))
    {
      parser.inner_graph_state.set_top_level_symbol(Some(f));
      if !element.is_method() && element.is_static() {
        let module_identifier = *parser.module_identifier;
        let span = body.span;
        parser.on_usage(Box::new(
          move |deps, used_by_exports| match used_by_exports {
            Some(UsedByExports::Bool(false)) | Some(UsedByExports::Set(_)) => {
              let mut dep =
                PureExpressionDependency::new(span.real_lo(), span.real_hi(), module_identifier);
              dep.used_by_exports = used_by_exports;
              deps.push(Box::new(dep))
            }
            Some(UsedByExports::Bool(true)) | None => (),
          },
        ))
      } else {
        parser.inner_graph_state.set_top_level_symbol(None);
      }
    }
    None
  }

  fn class_body_value(
    &self,
    parser: &mut JavascriptParser,
    value: &Expr,
    element: &swc_core::ecma::ast::ClassMember,
    classy: &swc_core::ecma::ast::Class,
  ) -> Option<bool> {
    if !parser.inner_graph_state.is_enabled() {
      return None;
    }
    if in_top_level(parser)
      && let Some(f) = parser
        .inner_graph_state
        .get_class_with_top_level_symbol(classy)
        .cloned()
      && (!element.is_static()
        || parser.is_pure_expr(
          value,
          element
            .class_key()
            .map(|key| key.span_hi())
            .unwrap_or(element.span_lo()),
        ))
    {
      parser.inner_graph_state.set_top_level_symbol(Some(f));
      if !element.is_method() && element.is_static() {
        let module_identifier = *parser.module_identifier;
        let span = value.span();
        parser.on_usage(Box::new(
          move |deps, used_by_exports| match used_by_exports {
            Some(UsedByExports::Bool(false)) | Some(UsedByExports::Set(_)) => {
              let mut dep =
                PureExpressionDependency::new(span.real_lo(), span.real_hi(), module_identifier);
              dep.used_by_exports = used_by_exports;
              deps.push(Box::new(dep))
            }
            Some(UsedByExports::Bool(true)) | None => (),
          },
        ))
      } else {
        parser.inner_graph_state.set_top_level_symbol(None);
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
    if !parser.inner_graph_state.is_enabled() {
      return None;
    }

    if let Some(f) = parser
      .inner_graph_state
      .get_decl_with_top_level_symbol(decl)
    {
      parser
        .inner_graph_state
        .set_top_level_symbol(Some(f.clone()));
      if parser.inner_graph_state.get_pure_declarator(decl).is_some()
        && let Some(init) = &decl.init
      {
        if let Some(class) = init.as_class()
          && let Some(super_class) = &class.class.super_class
        {
          parser.on_usage_super(super_class.deref());
        } else {
          let module_identifier = *parser.module_identifier;
          let span = init.span();
          parser.on_usage(Box::new(
            move |deps, used_by_exports| match used_by_exports {
              Some(UsedByExports::Bool(false)) | Some(UsedByExports::Set(_)) => {
                let mut dep =
                  PureExpressionDependency::new(span.real_lo(), span.real_hi(), module_identifier);
                dep.used_by_exports = used_by_exports;
                deps.push(Box::new(dep));
              }
              Some(UsedByExports::Bool(true)) | None => (),
            },
          ))
        }
      }

      if let Some(init) = &decl.init {
        parser.walk_expression(init);
      }
      parser.inner_graph_state.set_top_level_symbol(None);
      Some(true)
    } else if let Some(init) = &decl.init
      && let Some(class) = init.as_class()
      && decl.name.is_ident()
      && parser
        .inner_graph_state
        .get_class_with_top_level_symbol(&class.class)
        .is_some()
    {
      if let Some(init) = &decl.init {
        parser.walk_expression(init);
      }
      parser.inner_graph_state.set_top_level_symbol(None);
      Some(true)
    } else {
      None
    }
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    if for_name != TOP_LEVEL_SYMBOL_TAG || !parser.inner_graph_state.is_enabled() {
      return None;
    }
    let Some(tag_f) = parser.get_tag_data(ident.sym.as_str(), TOP_LEVEL_SYMBOL_TAG) else {
      unreachable!(
        "`{}` must had been tagged with TOP_LEVEL_SYMBOL_TAG",
        ident.sym.as_str()
      )
    };
    let tag_f: TopLevelSymbol = TagInfoData::deserialize(tag_f.clone());
    if let Some(f) = parser.inner_graph_state.get_top_level_symbol() {
      parser
        .inner_graph_state
        .add_usage(tag_f.0, InnerGraphMapUsage::TopLevel(f.clone()));
    } else {
      parser
        .inner_graph_state
        .add_usage(tag_f.0, InnerGraphMapUsage::True);
    }
    None
  }
}
