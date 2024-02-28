use std::collections::hash_map::Entry;

use rspack_core::{Dependency, UsedByExports};
use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::atoms::Atom;
use swc_core::common::{Span, Spanned};
use swc_core::ecma::ast::{Class, ExportDefaultDecl, ModuleDecl, Stmt, VarDeclarator};

use super::TopLevelSymbol;

#[derive(PartialEq, Eq, Debug)]
pub enum InnerGraphMapUsage {
  TopLevel(TopLevelSymbol),
  Value(Atom),
  True,
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum InnerGraphMapSetValue {
  TopLevel(Atom),
  Str(Atom),
}

/// You need to make sure that InnerGraphMapUsage is not a [InnerGraphMapUsage::True] variant
impl From<InnerGraphMapUsage> for InnerGraphMapSetValue {
  fn from(value: InnerGraphMapUsage) -> Self {
    match value {
      InnerGraphMapUsage::TopLevel(str) => Self::TopLevel(str.0),
      InnerGraphMapUsage::Value(str) => Self::Str(str),
      InnerGraphMapUsage::True => unreachable!(),
    }
  }
}

impl InnerGraphMapSetValue {
  pub fn to_atom(&self) -> &Atom {
    match self {
      InnerGraphMapSetValue::TopLevel(v) => v,
      InnerGraphMapSetValue::Str(v) => v,
    }
  }
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub enum InnerGraphMapValue {
  Set(FxHashSet<InnerGraphMapSetValue>),
  True,
  #[default]
  Nil,
}

pub type UsageCallback = Box<dyn Fn(&mut Vec<Box<dyn Dependency>>, Option<UsedByExports>)>;

#[derive(Default)]
pub struct InnerGraphState {
  pub inner_graph: FxHashMap<Atom, InnerGraphMapValue>,
  pub usage_callback_map: FxHashMap<TopLevelSymbol, Vec<UsageCallback>>,
  current_top_level_symbol: Option<TopLevelSymbol>,
  enabled: bool,
  statement_with_top_level_symbol: FxHashMap<Span, TopLevelSymbol>,
  statement_pure_part: FxHashMap<Span, Span>,
  decl_with_top_level_symbol: FxHashMap<Span, TopLevelSymbol>,
  pure_declarator: FxHashSet<Span>,
  class_with_top_level_symbol: FxHashMap<Span, TopLevelSymbol>,
}

impl InnerGraphState {
  #[inline]
  pub fn is_enabled(&self) -> bool {
    self.enabled
  }

  pub fn enable(&mut self) {
    self.enabled = true;
    self.inner_graph = Default::default();
    self.current_top_level_symbol = None;
    self.usage_callback_map = Default::default();
  }

  #[inline]
  pub fn bailout(&mut self) {
    self.enabled = false;
  }

  #[inline]
  pub fn set_top_level_symbol(&mut self, symbol: Option<TopLevelSymbol>) {
    self.current_top_level_symbol = symbol;
  }

  pub fn get_top_level_symbol(&self) -> Option<&TopLevelSymbol> {
    if self.is_enabled() {
      self.current_top_level_symbol.as_ref()
    } else {
      None
    }
  }

  pub fn add_usage(&mut self, symbol: Atom, usage: InnerGraphMapUsage) {
    if !self.is_enabled() {
      return;
    }
    match usage {
      InnerGraphMapUsage::True => {
        self.inner_graph.insert(symbol, InnerGraphMapValue::True);
      }
      InnerGraphMapUsage::Value(_) | InnerGraphMapUsage::TopLevel(_) => {
        // SAFETY: we can make sure that the usage is not a `InnerGraphMapSetValue::True` variant.
        let set_value: InnerGraphMapSetValue = usage.into();
        match self.inner_graph.entry(symbol) {
          Entry::Occupied(mut occ) => {
            let val = occ.get_mut();
            match val {
              InnerGraphMapValue::Set(set) => {
                set.insert(set_value);
              }
              InnerGraphMapValue::True => {
                // do nothing, https://github.com/webpack/webpack/blob/e381884115df2e7b8acd651d3bc2ee6fc35b188e/lib/optimize/InnerGraph.js#L92-L94
              }
              InnerGraphMapValue::Nil => {
                *val = InnerGraphMapValue::Set(FxHashSet::from_iter([set_value]));
              }
            }
          }
          Entry::Vacant(vac) => {
            vac.insert(InnerGraphMapValue::Set(FxHashSet::from_iter([set_value])));
          }
        }
      }
    }
  }

  // ------- statement_with_top_level_symbol ------
  #[inline]
  pub fn insert_statement_with_top_level_symbol_by_stmt(&mut self, stmt: &Stmt, f: TopLevelSymbol) {
    self.statement_with_top_level_symbol.insert(stmt.span(), f);
  }

  #[inline]
  pub fn get_statement_with_top_level_symbol_by_stmt(
    &mut self,
    stmt: &Stmt,
  ) -> Option<&TopLevelSymbol> {
    self.statement_with_top_level_symbol.get(&stmt.span())
  }

  #[inline]
  pub fn insert_statement_with_top_level_symbol_by_export_default_decl(
    &mut self,
    stmt: &ExportDefaultDecl,
    f: TopLevelSymbol,
  ) {
    self.statement_with_top_level_symbol.insert(stmt.span(), f);
  }

  #[inline]
  pub fn insert_statement_with_top_level_symbol_by_decl(
    &mut self,
    decl: &ModuleDecl,
    f: TopLevelSymbol,
  ) {
    self.statement_with_top_level_symbol.insert(decl.span(), f);
  }

  #[inline]
  pub fn get_statement_with_top_level_symbol_by_decl(
    &mut self,
    decl: &ModuleDecl,
  ) -> Option<&TopLevelSymbol> {
    self.statement_with_top_level_symbol.get(&decl.span())
  }

  // ------- class_with_top_level_symbol ---------
  #[inline]
  pub fn insert_class_with_top_level_symbol(&mut self, class: &Class, f: TopLevelSymbol) {
    self.class_with_top_level_symbol.insert(class.span(), f);
  }

  #[inline]
  pub fn get_class_with_top_level_symbol(&self, class: &Class) -> Option<&TopLevelSymbol> {
    self.class_with_top_level_symbol.get(&class.span())
  }

  // ----- statement_pure_part --------
  #[inline]
  pub fn insert_statement_pure_part_by_decl(&mut self, decl: &ModuleDecl, target_span: Span) {
    self.statement_pure_part.insert(decl.span(), target_span);
  }

  #[inline]
  pub fn get_statement_pure_part_by_decl(&self, decl: &ModuleDecl) -> Option<Span> {
    self.statement_pure_part.get(&decl.span()).cloned()
  }

  // ------- decl_with_top_level_symbol --------
  #[inline]
  pub fn insert_decl_with_top_level_symbol(&mut self, decl: &VarDeclarator, f: TopLevelSymbol) {
    self.decl_with_top_level_symbol.insert(decl.span, f);
  }

  #[inline]
  pub fn get_decl_with_top_level_symbol(&self, decl: &VarDeclarator) -> Option<&TopLevelSymbol> {
    self.decl_with_top_level_symbol.get(&decl.span())
  }

  // -------- pure_declarator -----
  #[inline]
  pub fn insert_pure_declarator(&mut self, decl: &VarDeclarator) {
    self.pure_declarator.insert(decl.span);
  }

  pub fn get_pure_declarator(&self, decl: &VarDeclarator) -> Option<Span> {
    self.pure_declarator.get(&decl.span).cloned()
  }
}
