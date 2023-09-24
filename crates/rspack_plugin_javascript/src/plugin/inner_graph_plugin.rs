use std::collections::hash_map::Entry;

use rspack_core::{Dependency, DependencyTemplate, SpanExt, UsedByExports};
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
  plugin::side_effects_flag_plugin::{is_pure_class, is_pure_expression},
};

#[derive(Hash, PartialEq, Eq, Clone)]
pub enum InnerGraphMapSetValue {
  TopLevel(JsWord),
  Str(JsWord),
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
  fn to_jsword(&self) -> &JsWord {
    match self {
      InnerGraphMapSetValue::TopLevel(v) => v,
      InnerGraphMapSetValue::Str(v) => v,
    }
  }
}

#[derive(Default)]
pub enum InnerGraphMapValue {
  Set(HashSet<InnerGraphMapSetValue>),
  True,
  #[default]
  Nil,
}

#[derive(PartialEq, Eq)]
pub enum InnerGraphMapUsage {
  TopLevel(JsWord),
  Value(JsWord),
  True,
}

pub type UsageCallback = Box<dyn Fn(&mut Vec<Box<dyn Dependency>>, Option<UsedByExports>)>;

#[derive(Default)]
pub struct InnerGraphState {
  inner_graph: HashMap<JsWord, InnerGraphMapValue>,
  usage_callback_map: HashMap<JsWord, Vec<UsageCallback>>,
  current_top_level_symbol: Option<JsWord>,
  enable: bool,
}

pub struct InnerGraphPlugin<'a> {
  dependencies: &'a mut Vec<Box<dyn Dependency>>,
  unresolved_ctxt: SyntaxContext,
  top_level_ctxt: SyntaxContext,
  state: InnerGraphState,
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

    if let Pat::Ident(ident) = &n.name &&  let Some(box init) = &n.init && is_pure_expression(init, self.unresolved_ctxt){
      let symbol = ident.id.sym.clone();
      match init {
        Expr::Fn(_) | Expr::Arrow(_) | Expr::Lit(_) => {},
        Expr::Class(class) => {
          self.visit_class(symbol, &class.class);
        }
        _ => {
          if is_pure_expression(init, self.unresolved_ctxt) {
            init.visit_children_with(self);
            let start = init.span().real_lo();
            let end = init.span().real_hi();
            self.on_usage(Box::new(move |deps, used_by_exports| {
              match used_by_exports {
                Some(UsedByExports::Bool(true)) | None=> return,
                _ => {
                  let mut dep = PureExpressionDependency::new(start, end);
                  dep.used_by_exports = used_by_exports;
                  deps.push(Box::new(dep));
                }
              }
            }));
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
    match node.expr {
      box Expr::Fn(_) | box Expr::Arrow(_) | box Expr::Lit(_) => {}
      box Expr::Class(ref class) => {
        self.visit_class(symbol, &class.class);
      }
      _ => {
        if is_pure_expression(&*node.expr, self.unresolved_ctxt) {
          node.expr.visit_children_with(self);
          // self.on_usage_by_span(Some(symbol), node.span.real_lo(), node.span.real_hi());
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
    dependencies: &'a mut Vec<Box<dyn Dependency>>,
    unresolved_ctxt: SyntaxContext,
    top_level_ctxt: SyntaxContext,
  ) -> Self {
    Self {
      dependencies,
      unresolved_ctxt,
      top_level_ctxt,
      state: InnerGraphState::default(),
    }
  }

  pub fn bailout(&mut self) {
    self.state.enable = false;
  }

  pub fn is_enabled(&self) -> bool {
    self.state.enable
  }

  pub fn add_usage(&mut self, symbol: JsWord, usage: InnerGraphMapUsage) {
    if !self.is_enabled() {
      return;
    }
    match usage {
      InnerGraphMapUsage::True => {
        self
          .state
          .inner_graph
          .insert(symbol, InnerGraphMapValue::True);
      }
      InnerGraphMapUsage::Value(ref str) | InnerGraphMapUsage::TopLevel(ref str) => {
        // SAFETY: we can make sure that the usage is not a `InnerGraphMapSetValue::True` variant.
        let set_value: InnerGraphMapSetValue = usage.into();
        match self.state.inner_graph.entry(symbol) {
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
                *val = InnerGraphMapValue::Set(HashSet::from_iter([set_value]));
              }
            }
          }
          Entry::Vacant(vac) => {
            vac.insert(InnerGraphMapValue::Set(HashSet::from_iter([set_value])));
          }
        }
      }
    }
  }

  pub fn add_variable_usage(&mut self, name: JsWord, usage: JsWord) {
    self.add_usage(name, InnerGraphMapUsage::Value(usage));
  }

  pub fn on_usage(&mut self, on_usage_callback: UsageCallback) {
    if self.is_enabled() {
      if let Some(symbol) = self.get_top_level_symbol() {
        self
          .state
          .usage_callback_map
          .entry(symbol)
          .or_insert(vec![])
          .push(on_usage_callback);
      } else {
        on_usage_callback(self.dependencies, Some(UsedByExports::Bool(true)));
      }
    } else {
      on_usage_callback(self.dependencies, None);
    }
  }

  pub fn visit_class(&mut self, symbol: JsWord, class: &Class) {
    self.set_top_level_symbol(Some(symbol.clone()));
    for stmt in class.body.iter() {
      match stmt {
        ClassMember::ClassProp(p) => {
          if let Some(box expr) = &p.value &&  is_pure_expression(expr, self.unresolved_ctxt) && p.is_static  {
            expr.visit_children_with(self);
            // self.on_usage_by_span(Some(symbol.clone()), expr.span().real_lo(), expr.span().real_hi());
          }
        }
        ClassMember::PrivateProp(p) => {
          if let Some(box expr) = &p.value &&  is_pure_expression(expr, self.unresolved_ctxt) && p.is_static  {
            expr.visit_children_with(self);
            // self.on_usage_by_span(Some(symbol.clone()), expr.span().real_lo(), expr.span().real_hi());
          }
        }
        _ => {}
      }
    }
    // `onUsageSuper`
    if let Some(box Expr::Ident(ident)) = &class.super_class && is_pure_class(class, self.unresolved_ctxt) {
      ident.visit_children_with(self);
      // self.on_usage_by_span(Some(symbol), class.span.real_lo(), class.span.real_hi());
    }
    self.set_top_level_symbol(None);
  }

  pub fn set_top_level_symbol(&mut self, symbol: Option<JsWord>) {
    self.state.current_top_level_symbol = symbol;
  }

  pub fn get_top_level_symbol(&self) -> Option<JsWord> {
    if self.is_enabled() {
      self.state.current_top_level_symbol.clone()
    } else {
      None
    }
  }

  pub fn infer_dependency_usage(&mut self) {
    if !self.is_enabled() {
      return;
    }
    let state = &mut self.state;
    let mut non_terminal = HashSet::from_iter(state.inner_graph.keys().cloned());
    let mut processed: HashMap<JsWord, HashSet<JsWord>> = HashMap::default();
    while !non_terminal.is_empty() {
      let mut keys_to_remove = vec![];
      for key in non_terminal.iter() {
        let mut new_set = HashSet::default();
        // Using enum to manipulate original is pretty hard, so I use an extra variable to
        //mark if the new set has changed to an set
        let mut new_set_is_true = false;
        let mut is_terminal = true;
        let mut already_processed = processed.entry(key.clone()).or_default();
        if let Some(InnerGraphMapValue::Set(names)) = state.inner_graph.get(&key) {
          for name in names.iter() {
            already_processed.insert(name.to_jsword().clone());
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
                    new_set_is_true = true;
                    break;
                  }
                  Some(InnerGraphMapValue::Set(item_value)) => {
                    for i in item_value {
                      if i.to_jsword() == key {
                        continue;
                      }
                      if already_processed.contains(i.to_jsword()) {
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
          if new_set_is_true {
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
          // webpack use null, using enum make code is hard to write, also there is no
          // way to export a empty string, so use `""` to represent `null` should be safe
          // https://github.com/IWANABETHATGUY/webpack/blob/d15c73469fd71cf98734685225250148b68ddc79/lib/optimize/InnerGraph.js#L177
          if key == "" {

            // TODO:
          }
        }
      }
      for k in keys_to_remove {
        non_terminal.remove(&k);
      }
    }
    // TODO: invoke callback
  }
}
