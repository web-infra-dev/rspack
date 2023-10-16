use std::{collections::hash_map::Entry, hash::Hash};

use rspack_core::{Dependency, SpanExt, UsedByExports};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_core::{
  common::{Span, Spanned, SyntaxContext},
  ecma::{
    ast::{
      ArrowExpr, CallExpr, Callee, Class, ClassDecl, ClassMember, DefaultDecl, ExportDecl,
      ExportDefaultDecl, ExportDefaultExpr, ExportSpecifier, Expr, FnDecl, FnExpr, Ident,
      MemberExpr, NamedExport, Pat, Program, Prop, VarDeclarator,
    },
    atoms::JsWord,
    visit::{noop_visit_type, Visit, VisitWith},
  },
};

use crate::{
  dependency::PureExpressionDependency,
  plugin::side_effects_flag_plugin::{is_pure_class, is_pure_expression},
  visitors::{harmony_import_dependency_scanner::ImportMap, ExtraSpanInfo},
  ClassKey,
};

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
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

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub enum InnerGraphMapValue {
  Set(HashSet<InnerGraphMapSetValue>),
  True,
  #[default]
  Nil,
}

#[derive(PartialEq, Eq, Debug)]
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
  scope_level: usize,
  rewrite_usage_span: &'a mut HashMap<Span, ExtraSpanInfo>,
  import_map: &'a ImportMap,
}

impl<'a> Visit for InnerGraphPlugin<'a> {
  noop_visit_type!();
  fn visit_program(&mut self, program: &Program) {
    if !self.is_enabled() {
      return;
    }
    program.visit_children_with(self);
  }

  fn visit_call_expr(&mut self, call_expr: &CallExpr) {
    // https://github.com/webpack/webpack/blob/main/lib/JavascriptMetaInfoPlugin.js#L46
    if let Callee::Expr(box Expr::Ident(ident)) = &call_expr.callee && &ident.sym == "eval" {
      if let Some(current_symbol) = self.get_top_level_symbol() {
        // We use `""` to represent `null
        self.add_usage("".into(), InnerGraphMapUsage::TopLevel(current_symbol));
      } else {
        self.bailout();
        return;
      }
    }
    call_expr.visit_children_with(self);
  }

  fn visit_member_expr(&mut self, member_expr: &MemberExpr) {
    match self.rewrite_usage_span.get(&member_expr.span) {
      Some(ExtraSpanInfo::ReWriteUsedByExports) => {
        let span = member_expr.span;
        self.on_usage(Box::new(move |deps, used_by_exports| {
          let target_dep = deps.iter_mut().find(|item| item.is_span_equal(&span));
          if let Some(dep) = target_dep {
            dep.set_used_by_exports(used_by_exports);
          }
        }));
      }
      // member_expr is not possible to add a variable usage
      _ => {}
    };
  }

  fn visit_class_member(&mut self, node: &ClassMember) {
    if let Some(key) = node.class_key() && key.is_computed() {
      key.visit_with(self);
    }

    let increase_level = match node {
      ClassMember::Constructor(_) => 1,
      ClassMember::Method(_) => 1,
      ClassMember::PrivateMethod(_) => 1,
      ClassMember::ClassProp(_) => 0,
      ClassMember::PrivateProp(_) => 0,
      ClassMember::TsIndexSignature(_) => unreachable!(),
      ClassMember::Empty(_) => 0,
      ClassMember::StaticBlock(_) => 1,
      ClassMember::AutoAccessor(_) => 0,
    };
    let scope_level = self.scope_level;
    self.scope_level += increase_level;
    node.visit_children_with(self);
    self.scope_level = scope_level;
  }

  fn visit_fn_decl(&mut self, node: &FnDecl) {
    self.set_symbol_if_is_top_level(node.ident.sym.clone());
    let scope_level = self.scope_level;
    self.scope_level += 1;
    node.function.visit_children_with(self);
    self.scope_level = scope_level;
    self.clear_symbol_if_is_top_level();
  }

  // TODO: expr
  fn visit_fn_expr(&mut self, node: &FnExpr) {
    let scope_level = self.scope_level;
    self.scope_level += 1;
    node.visit_children_with(self);
    self.scope_level = scope_level;
  }

  fn visit_arrow_expr(&mut self, node: &ArrowExpr) {
    let scope_level = self.scope_level;
    self.scope_level += 1;
    node.visit_children_with(self);
    self.scope_level = scope_level;
  }

  fn visit_class_decl(&mut self, node: &ClassDecl) {
    if !self.is_enabled() {
      return;
    }

    let scope_level = self.scope_level;
    self.scope_level += 1;
    // TODO: consider class
    node.visit_children_with(self);
    self.scope_level = scope_level;
  }

  fn visit_ident(&mut self, ident: &Ident) {
    match self.rewrite_usage_span.get(&ident.span) {
      Some(ExtraSpanInfo::ReWriteUsedByExports) => {
        let span = ident.span;
        self.on_usage(Box::new(move |deps, used_by_exports| {
          let target_dep = deps.iter_mut().find(|item| item.is_span_equal(&span));
          if let Some(dep) = target_dep {
            dep.set_used_by_exports(used_by_exports);
          }
        }));
      }
      // ident is impossible to add a variable usage
      _ => {}
    };
    // imported binding isn't considered as a top level symbol.
    if self.import_map.contains_key(&ident.to_id()) {
      return;
    };
    if ident.span.ctxt == self.top_level_ctxt {
      let usage = if let Some(symbol) = self.get_top_level_symbol() {
        InnerGraphMapUsage::Value(symbol)
      } else {
        InnerGraphMapUsage::True
      };
      self.add_usage(ident.sym.clone(), usage);
    }
  }

  fn visit_var_declarator(&mut self, n: &VarDeclarator) {
    if !self.is_enabled() {
      return;
    }

    if let Pat::Ident(ident) = &n.name {
      if let Some(box init) = &n.init && is_pure_expression(init, self.unresolved_ctxt) {
        let symbol = ident.id.sym.clone();
        self.set_symbol_if_is_top_level(symbol);
        match init {
          Expr::Fn(_) | Expr::Arrow(_) | Expr::Lit(_) => {},
          Expr::Class(class) => {
              // TODO: consider class 
            class.class.visit_children_with(self);
          }
          _ => {
            init.visit_children_with(self);
            if self.has_toplevel_symbol() && is_pure_expression(init, self.unresolved_ctxt) {
              let start = init.span().real_lo();
              let end = init.span().real_hi();
              self.on_usage(Box::new(move |deps, used_by_exports| {
                match used_by_exports {
                  Some(UsedByExports::Bool(true)) | None=> {},
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
    n.visit_children_with(self);
    self.clear_symbol_if_is_top_level();
  }

  fn visit_prop(&mut self, n: &Prop) {
    match n {
      Prop::Shorthand(shorthand) => {
        match self.rewrite_usage_span.get(&shorthand.span) {
          Some(ExtraSpanInfo::ReWriteUsedByExports) => {
            let span = shorthand.span;
            self.on_usage(Box::new(move |deps, used_by_exports| {
              let target_dep = deps.iter_mut().find(|item| item.is_span_equal(&span));
              if let Some(dep) = target_dep {
                dep.set_used_by_exports(used_by_exports);
              }
            }));
          }
          // prop is impossible to add a variable usage
          _ => {}
        };
      }
      _ => n.visit_children_with(self),
    }
  }
  fn visit_export_decl(&mut self, export_decl: &ExportDecl) {
    match self.rewrite_usage_span.get(&export_decl.span) {
      Some(ExtraSpanInfo::AddVariableUsage(sym, usage)) => {
        self.add_variable_usage(sym.clone(), usage.clone());
      }
      _ => {}
    }
    // match &export_decl.decl {
    //   Decl::Class(ClassDecl { ident, .. }) | Decl::Fn(FnDecl { ident, .. }) => {
    //     // self.add_variable_usage(ident.sym.clone(), ident.sym.clone());
    //   }
    //   Decl::Var(v) => {
    //     find_pat_ids::<_, Ident>(&v.decls)
    //       .into_iter()
    //       .for_each(|ident| {
    //         // self.add_variable_usage(ident.sym.clone(), ident.sym.clone());
    //       });
    //   }
    //   _ => {}
    // }
    export_decl.visit_children_with(self);
  }

  fn visit_named_export(&mut self, named_export: &NamedExport) {
    if named_export.src.is_none() {
      named_export
        .specifiers
        .iter()
        .for_each(|specifier| match specifier {
          ExportSpecifier::Named(named) => match self.rewrite_usage_span.get(&named.span) {
            Some(ExtraSpanInfo::AddVariableUsage(sym, usage)) => {
              self.add_variable_usage(sym.clone(), usage.clone());
            }
            _ => {}
          },
          _ => unreachable!(),
        });
    }
  }
  fn visit_export_default_expr(&mut self, node: &ExportDefaultExpr) {
    if !self.is_enabled() {
      return;
    }
    // TODO: use rewrite Usage span instead
    match node.expr {
      box Expr::Fn(_) | box Expr::Arrow(_) | box Expr::Lit(_) => {
        node.expr.visit_children_with(self);
      }
      box Expr::Class(ref class) => {
        // TODO: class
        class.visit_with(self);
      }
      _ => {
        node.expr.visit_children_with(self);
        if is_pure_expression(&node.expr, self.unresolved_ctxt) {
          let start = node.expr.span().real_lo();
          let end = node.expr.span().real_hi();
          self.on_usage(Box::new(
            move |deps, used_by_exports| match used_by_exports {
              Some(UsedByExports::Bool(true)) | None => {}
              _ => {
                let mut dep = PureExpressionDependency::new(start, end);
                dep.used_by_exports = used_by_exports;
                deps.push(Box::new(dep));
              }
            },
          ));
        }
      }
    }
  }

  fn visit_export_default_decl(&mut self, node: &ExportDefaultDecl) {
    if !self.is_enabled() {
      return;
    }
    // TODO:
    // let symbol: JsWord = "*default*".into();
    match &node.decl {
      DefaultDecl::Class(class) => {
        // self.visit_class(symbol, &class.class);
        class.visit_children_with(self);
      }
      DefaultDecl::Fn(func) => {
        func.visit_with(self);
      }
      DefaultDecl::TsInterfaceDecl(_) => unreachable!(),
    }
  }
}

impl<'a> InnerGraphPlugin<'a> {
  pub fn new(
    dependencies: &'a mut Vec<Box<dyn Dependency>>,
    unresolved_ctxt: SyntaxContext,
    top_level_ctxt: SyntaxContext,
    rewrite_usage_span: &'a mut HashMap<Span, ExtraSpanInfo>,
    import_map: &'a ImportMap,
  ) -> Self {
    Self {
      dependencies,
      unresolved_ctxt,
      top_level_ctxt,
      state: InnerGraphState::default(),
      scope_level: 0,
      rewrite_usage_span,
      import_map,
    }
  }

  pub fn enable(&mut self) {
    self.state.enable = true;
  }

  fn is_toplevel(&self) -> bool {
    self.scope_level == 0
  }

  fn has_toplevel_symbol(&self) -> bool {
    self.state.current_top_level_symbol.is_some()
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
      InnerGraphMapUsage::Value(_) | InnerGraphMapUsage::TopLevel(_) => {
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

  pub fn visit_class_custom(&mut self, symbol: JsWord, class: &Class) {
    self.set_top_level_symbol(Some(symbol));
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

  pub fn set_symbol_if_is_top_level(&mut self, symbol: JsWord) {
    if self.is_toplevel() {
      self.set_top_level_symbol(Some(symbol));
    }
  }

  pub fn clear_symbol_if_is_top_level(&mut self) {
    if self.is_toplevel() {
      self.set_top_level_symbol(None);
    }
  }

  pub fn get_top_level_symbol(&self) -> Option<JsWord> {
    if self.is_enabled() {
      self.state.current_top_level_symbol.clone()
    } else {
      None
    }
  }

  pub fn infer_dependency_usage(&mut self) {
    // fun will reference it self
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
        // flagging the new set has changed to boolean `true`
        // you could refer https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/optimize/InnerGraph.js#L150
        let mut set_is_true = false;
        let mut is_terminal = true;
        let already_processed = processed.entry(key.clone()).or_default();
        if let Some(InnerGraphMapValue::Set(names)) = state.inner_graph.get(key) {
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
                    set_is_true = true;
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
            let global_value = state.inner_graph.get(&JsWord::from("")).cloned();
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

    dbg!(&state.inner_graph);
    for (symbol, cbs) in state.usage_callback_map.iter() {
      let usage = state.inner_graph.get(symbol);
      dbg!(symbol, usage);
      for cb in cbs {
        let used_by_exports = if let Some(usage) = usage {
          match usage {
            InnerGraphMapValue::Set(set) => {
              let finalized_set =
                HashSet::from_iter(set.iter().map(|item| item.to_jsword().clone()));
              UsedByExports::Set(finalized_set)
            }
            InnerGraphMapValue::True => UsedByExports::Bool(true),
            InnerGraphMapValue::Nil => UsedByExports::Bool(false),
          }
        } else {
          UsedByExports::Bool(false)
        };

        cb(self.dependencies, Some(used_by_exports));
      }
    }
  }
}
