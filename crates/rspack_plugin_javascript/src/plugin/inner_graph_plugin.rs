use rspack_core::{DependencyTemplate, SpanExt, UsedByExports};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_core::{
  common::{Spanned, SyntaxContext},
  ecma::{
    ast::{
      ArrowExpr, CallExpr, Callee, Class, ClassDecl, ClassExpr, ClassMember, Decl, DefaultDecl,
      ExportDecl, ExportDefaultDecl, ExportDefaultExpr, Expr, FnDecl, FnExpr, Ident, Pat,
      VarDeclarator,
    },
    atoms::JsWord,
    utils::find_pat_ids,
    visit::{noop_visit_type, Visit, VisitWith},
  },
};

use crate::{
  dependency::PureExpressionDependency,
  side_effects_flag_plugin::{is_pure_class, is_pure_expression},
};

#[derive(Default)]
pub enum InnerGraphMapValue {
  Set(HashSet<JsWord>),
  True,
  #[default]
  Nil,
}

#[derive(PartialEq, Eq)]
pub enum InnerGraphMapUsage {
  Value(JsWord),
  True,
}

pub type UsageCallback = Box<dyn Fn(UsedByExports)>;

#[derive(Default)]
pub struct InnerGraphState {
  inner_graph: HashMap<JsWord, InnerGraphMapValue>,
  usage_callback_map: HashMap<JsWord, Vec<UsageCallback>>,
  current_top_level_symbol: Option<JsWord>,
}

pub struct InnerGraphPlugin<'a> {
  presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
  unresolved_ctxt: SyntaxContext,
  top_level_ctxt: SyntaxContext,
  state: Option<InnerGraphState>,
}

impl<'a> Visit for InnerGraphPlugin<'a> {
  noop_visit_type!();

  fn visit_call_expr(&mut self, call_expr: &CallExpr) {
    // https://github.com/webpack/webpack/blob/main/lib/JavascriptMetaInfoPlugin.js#L46
    if let Callee::Expr(box Expr::Ident(ident)) = &call_expr.callee && &ident.sym == "eval" {
      self.bailout();
    }
  }

  fn visit_class_member(&mut self, node: &ClassMember) {}

  fn visit_fn_decl(&mut self, node: &FnDecl) {}

  fn visit_fn_expr(&mut self, node: &FnExpr) {}

  fn visit_arrow_expr(&mut self, node: &ArrowExpr) {}

  fn visit_class_expr(&mut self, node: &ClassExpr) {
    if !self.is_enabled() {
      return;
    }
    if let Some(ident) = &node.ident {
      self.visit_class(ident.sym.clone(), &node.class);
    }
  }

  fn visit_class_decl(&mut self, node: &ClassDecl) {
    if !self.is_enabled() {
      return;
    }
    self.visit_class(node.ident.sym.clone(), &node.class);
  }

  fn visit_ident(&mut self, ident: &Ident) {
    if ident.span.ctxt == self.top_level_ctxt {
      self.add_usage(
        ident.sym.clone(),
        if let Some(symbol) = self.get_top_level_symbol() {
          InnerGraphMapUsage::Value(symbol)
        } else {
          InnerGraphMapUsage::True
        },
      );
    }
  }

  fn visit_var_declarator(&mut self, n: &VarDeclarator) {
    if !self.is_enabled() {
      return;
    }
    if let Pat::Ident(ident) = &n.name &&  let Some(box expr) = &n.init && is_pure_expression(expr, self.unresolved_ctxt){
      let symbol = ident.id.sym.clone();
      match expr {
        Expr::Fn(_) | Expr::Arrow(_) | Expr::Lit(_) => {},
        Expr::Class(class) => {
          self.visit_class(symbol, &class.class);
        }
        _ => {
          if is_pure_expression(expr, self.unresolved_ctxt) {
            expr.visit_children_with(self);
            self.on_usage_by_span(Some(symbol), expr.span().real_lo(), expr.span().real_hi());
          }
        }
      }
    }
  }

  fn visit_export_decl(&mut self, export_decl: &ExportDecl) {
    match &export_decl.decl {
      Decl::Class(ClassDecl { ident, .. }) | Decl::Fn(FnDecl { ident, .. }) => {
        self.add_variable_usage(ident.sym.clone(), ident.sym.clone());
      }
      Decl::Var(v) => {
        find_pat_ids::<_, Ident>(&v.decls)
          .into_iter()
          .for_each(|ident| {
            self.add_variable_usage(ident.sym.clone(), ident.sym.clone());
          });
      }
      _ => {}
    }
    export_decl.visit_children_with(self);
  }
  // TODO add_variable_usage `exportExpression`
  fn visit_export_default_expr(&mut self, node: &ExportDefaultExpr) {
    if !self.is_enabled() {
      return;
    }
    let symbol: JsWord = "*default*".into();
    match *node.expr {
      Expr::Fn(_) | Expr::Arrow(_) | Expr::Lit(_) => {}
      Expr::Class(class) => {
        self.visit_class(symbol, &class.class);
      }
      _ => {
        if is_pure_expression(&*node.expr, self.unresolved_ctxt) {
          node.expr.visit_children_with(self);
          self.on_usage_by_span(Some(symbol), node.span.real_lo(), node.span.real_hi());
        }
      }
    }
  }

  fn visit_export_default_decl(&mut self, node: &ExportDefaultDecl) {
    if !self.is_enabled() {
      return;
    }
    let symbol: JsWord = "*default*".into();
    match &node.decl {
      DefaultDecl::Class(class) => {
        self.visit_class(symbol, &class.class);
      }
      // DefaultDecl::Fn(_) => todo!(),
      _ => {}
    }
  }
}

impl<'a> InnerGraphPlugin<'a> {
  pub fn new(
    presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
    unresolved_ctxt: SyntaxContext,
    top_level_ctxt: SyntaxContext,
  ) -> Self {
    Self {
      presentational_dependencies,
      unresolved_ctxt,
      top_level_ctxt,
      state: Some(InnerGraphState::default()),
    }
  }

  pub fn bailout(&mut self) {
    self.state = None;
  }

  pub fn is_enabled(&self) -> bool {
    self.state.is_some()
  }

  pub fn add_usage(&mut self, symbol: JsWord, usage: InnerGraphMapUsage) {
    if let Some(state) = self.state.as_mut() {
      let info = state.inner_graph.get_mut(&symbol);
      match usage {
        InnerGraphMapUsage::True => {
          state.inner_graph.insert(symbol, InnerGraphMapValue::True);
        }
        InnerGraphMapUsage::Value(value) => {
          if let Some(info) = info {
            if let InnerGraphMapValue::Set(set) = info {
              set.insert(value);
            }
          } else {
            state.inner_graph.insert(
              symbol,
              InnerGraphMapValue::Set(HashSet::from_iter(vec![value])),
            );
          }
        }
      }
    }
  }

  pub fn add_variable_usage(&mut self, name: JsWord, usage: JsWord) {
    self.add_usage(name, InnerGraphMapUsage::Value(usage));
  }

  pub fn on_usage(&mut self, symbol: Option<JsWord>, on_usage_callback: UsageCallback) {
    if let Some(state) = self.state.as_mut() {
      if let Some(symbol) = symbol {
        state
          .usage_callback_map
          .entry(symbol)
          .or_insert(vec![])
          .push(on_usage_callback);
      } else {
        on_usage_callback(UsedByExports::Bool(true));
      }
    } else {
      on_usage_callback(UsedByExports::Nil);
    }
  }

  pub fn on_usage_by_span(&mut self, symbol: Option<JsWord>, start: u32, end: u32) {
    self.on_usage(
      symbol,
      Box::new(|used_by_exports| {
        if matches!(used_by_exports, UsedByExports::Nil)
          || matches!(used_by_exports, UsedByExports::Bool(true))
        {
          return;
        } else {
          // TODO usedByExports
          self
            .presentational_dependencies
            .push(Box::new(PureExpressionDependency::new(start, end)));
        }
      }),
    )
  }

  pub fn visit_class(&mut self, symbol: JsWord, class: &Class) {
    self.set_top_level_symbol(Some(symbol.clone()));
    for stmt in class.body.iter() {
      match stmt {
        ClassMember::ClassProp(p)  => {
          if let Some(box expr) = &p.value &&  is_pure_expression(expr, self.unresolved_ctxt) && p.is_static  {
            expr.visit_children_with(self);
            self.on_usage_by_span(Some(symbol.clone()), expr.span().real_lo(), expr.span().real_hi());
          }
        }
        ClassMember::PrivateProp(p) => {
          if let Some(box expr) = &p.value &&  is_pure_expression(expr, self.unresolved_ctxt) && p.is_static  {
            expr.visit_children_with(self);
            self.on_usage_by_span(Some(symbol.clone()), expr.span().real_lo(), expr.span().real_hi());
          }
        }
        _ => {}
      }
    }
    // `onUsageSuper`
    if let Some(box Expr::Ident(ident)) = &class.super_class && is_pure_class(class, self.unresolved_ctxt) {
      ident.visit_children_with(self);
      self.on_usage_by_span(Some(symbol), class.span.real_lo(), class.span.real_hi());
    }
    self.set_top_level_symbol(None);
  }

  pub fn set_top_level_symbol(&mut self, symbol: Option<JsWord>) {
    if let Some(state) = self.state.as_mut() {
      state.current_top_level_symbol = symbol;
    }
  }

  pub fn get_top_level_symbol(&self) -> Option<JsWord> {
    self.state.and_then(|s| s.current_top_level_symbol)
  }

  pub fn infer_dependency_usage(&mut self) {
    if let Some(state) = self.state.as_mut() {
      let non_terminal = HashSet::from_iter(state.inner_graph.keys());
      let mut processed: HashMap<JsWord, HashSet<JsWord>> = HashMap::default();
      while !non_terminal.is_empty() {
        for key in non_terminal {
          let mut new_set = HashSet::default();
          let mut entry = processed.entry(key.clone()).or_default();
          if let Some(InnerGraphMapValue::Set(value)) = state.inner_graph.get(key) {
            for item in value.iter() {
              entry.insert(item.clone());
            }
            for item in value {
              if let Some(item_value) = state.inner_graph.get(item) {
                match item_value {
                  InnerGraphMapValue::Set(set) => {
                    let mut new_set = HashSet::default();
                    for i in set {
                      if i == key {
                        continue;
                      }
                      if entry.contains(i) {
                        continue;
                      }
                      new_set.insert(i);
                    }
                    if new_set.is_empty() {
                      state
                        .inner_graph
                        .insert(key.clone(), InnerGraphMapValue::Nil);
                    } else {
                      state
                        .inner_graph
                        .insert(key.clone(), InnerGraphMapValue::Set(new_set));
                    }
                  }
                  InnerGraphMapValue::True => {
                    state
                      .inner_graph
                      .insert(key.clone(), InnerGraphMapValue::True);
                  }
                  InnerGraphMapValue::Nil => {}
                }
              }
            }
          }
          // processed.entry(key).and_modify(f).
        }
      }
    }
  }
}
