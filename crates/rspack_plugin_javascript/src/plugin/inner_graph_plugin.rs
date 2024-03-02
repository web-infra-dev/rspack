use std::{collections::hash_map::Entry, hash::Hash};

use rspack_core::{Dependency, ModuleIdentifier, SpanExt, UsedByExports, DEFAULT_EXPORT};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_core::{
  common::{Span, Spanned, SyntaxContext},
  ecma::{
    ast::{
      ArrowExpr, CallExpr, Callee, Class, ClassDecl, ClassExpr, ClassMember, DefaultDecl,
      ExportDecl, ExportDefaultDecl, ExportDefaultExpr, Expr, FnDecl, FnExpr, Function, Ident, Key,
      MemberExpr, NamedExport, OptChainExpr, Pat, Program, Prop, VarDeclarator,
    },
    atoms::Atom,
    visit::{noop_visit_type, Visit, VisitWith},
  },
};
use swc_node_comments::SwcComments;

use crate::{
  dependency::PureExpressionDependency,
  is_pure_class, is_pure_class_member,
  plugin::side_effects_flag_plugin::is_pure_expression,
  visitors::{ExtraSpanInfo, ImportMap},
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
  fn to_atom(&self) -> &Atom {
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

pub type UsageCallback = Box<dyn Fn(&mut Vec<Box<dyn Dependency>>, Option<UsedByExports>)>;

#[derive(Default)]
pub struct InnerGraphState {
  inner_graph: HashMap<Atom, InnerGraphMapValue>,
  usage_callback_map: HashMap<Atom, Vec<UsageCallback>>,
  current_top_level_symbol: Option<Atom>,
  enable: bool,
  module_identifier: ModuleIdentifier,
}

pub struct InnerGraphPlugin<'a> {
  dependencies: &'a mut Vec<Box<dyn Dependency>>,
  unresolved_ctxt: SyntaxContext,
  state: InnerGraphState,
  scope_level: usize,
  rewrite_usage_span: &'a mut HashMap<Span, ExtraSpanInfo>,
  import_map: &'a ImportMap,
  in_named: bool,
  top_level_ctxt_set: HashSet<SyntaxContext>,
  pub comments: Option<SwcComments>,
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
    if let Callee::Expr(box Expr::Ident(ident)) = &call_expr.callee
      && &ident.sym == "eval"
    {
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

  fn visit_opt_chain_expr(&mut self, opt_chain_expr: &OptChainExpr) {
    if let Some(ExtraSpanInfo::ReWriteUsedByExports) =
      self.rewrite_usage_span.get(&opt_chain_expr.span)
    {
      let span = opt_chain_expr.span;
      self.on_usage(Box::new(move |deps, used_by_exports| {
        let target_dep = deps.iter_mut().find(|item| item.is_span_equal(&span));
        if let Some(dep) = target_dep {
          dep.set_used_by_exports(used_by_exports);
        }
      }));
    };
    opt_chain_expr.visit_children_with(self);
  }

  fn visit_member_expr(&mut self, member_expr: &MemberExpr) {
    if let Some(ExtraSpanInfo::ReWriteUsedByExports) =
      self.rewrite_usage_span.get(&member_expr.span)
    {
      let span = member_expr.span;
      self.on_usage(Box::new(move |deps, used_by_exports| {
        let target_dep = deps.iter_mut().find(|item| item.is_span_equal(&span));
        if let Some(dep) = target_dep {
          dep.set_used_by_exports(used_by_exports);
        }
      }));
    };
    member_expr.visit_children_with(self);
  }

  fn visit_class_member(&mut self, node: &ClassMember) {
    if self.scope_level != 1 {
      node.visit_children_with(self);
      return;
    }
    let previous_top_level_symbol = self.get_top_level_symbol();
    self.set_top_level_symbol(None);
    if let Some(key) = node.class_key() {
      // key needs with visit a empty toplevel symbol, cause it maybe computed value.
      key.visit_with(self);
    };
    let is_static = node.is_static();
    if !is_static || is_pure_class_member(node, self.unresolved_ctxt, self.comments.as_ref()) {
      self.set_top_level_symbol(previous_top_level_symbol.clone());
      if is_static && !matches!(node, ClassMember::Method(_) | ClassMember::PrivateMethod(_)) {
        let span = match node {
          ClassMember::Constructor(_) => unreachable!(),
          ClassMember::Method(_) => unreachable!(),
          ClassMember::PrivateMethod(_) => unreachable!(),
          ClassMember::ClassProp(prop) => prop.value.as_ref().map(|item| item.span()),
          ClassMember::PrivateProp(prop) => prop.value.as_ref().map(|item| item.span()),
          ClassMember::TsIndexSignature(_) => unreachable!(),
          ClassMember::Empty(_) => None,
          ClassMember::StaticBlock(block) => Some(block.span()),
          ClassMember::AutoAccessor(_) => todo!(),
        };
        if let Some(span) = span {
          let start = span.real_lo();
          let end = span.real_hi();
          let module_identifier = self.state.module_identifier;
          self.on_usage(Box::new(
            move |deps, used_by_exports| match used_by_exports {
              Some(UsedByExports::Bool(true)) | None => {}
              _ => {
                let mut dep = PureExpressionDependency::new(start, end, module_identifier);
                dep.used_by_exports = used_by_exports;
                deps.push(Box::new(dep));
              }
            },
          ));
        }
      }
    }
    let scope_level = self.scope_level;
    match node {
      ClassMember::Constructor(c) => {
        self.scope_level += 1;
        c.params.visit_with(self);
        c.body.visit_with(self);
        c.accessibility.visit_children_with(self);
      }
      ClassMember::Method(m) => {
        self.scope_level += 1;
        m.function.visit_with(self);
        m.kind.visit_with(self);
        m.accessibility.visit_children_with(self);
      }
      ClassMember::PrivateMethod(c) => {
        self.scope_level += 1;
        c.function.visit_with(self);
        c.kind.visit_with(self);
        c.accessibility.visit_children_with(self);
      }
      ClassMember::ClassProp(c) => {
        c.value.visit_with(self);
        c.decorators.visit_with(self);
        c.accessibility.visit_with(self);
      }
      ClassMember::PrivateProp(prop) => {
        prop.value.visit_with(self);
        prop.decorators.visit_with(self);
        prop.accessibility.visit_with(self);
      }
      ClassMember::TsIndexSignature(_) => {}
      ClassMember::Empty(_) => {}
      ClassMember::StaticBlock(block) => {
        self.scope_level += 1;
        block.visit_with(self)
      }
      ClassMember::AutoAccessor(a) => match a.key {
        Key::Private(ref _private) => {}
        Key::Public(ref _public) => {
          // already visited.
        }
      },
    };
    self.scope_level = scope_level;
    self.set_top_level_symbol(previous_top_level_symbol);
  }

  fn visit_fn_decl(&mut self, node: &FnDecl) {
    self.set_symbol_if_is_top_level(node.ident.sym.clone());
    node.function.visit_with(self);
    self.clear_symbol_if_is_top_level();
  }

  fn visit_function(&mut self, node: &Function) {
    let scope_level = self.scope_level;
    self.scope_level += 1;
    node.visit_children_with(self);
    self.scope_level = scope_level;
  }

  fn visit_fn_expr(&mut self, node: &FnExpr) {
    node.function.visit_with(self);
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
    let is_pure_class = is_pure_class(&node.class, self.unresolved_ctxt, self.comments.as_ref());
    if is_pure_class {
      self.set_symbol_if_is_top_level(node.ident.sym.clone());
    }
    let is_toplevel = self.is_toplevel();
    let scope_level = self.scope_level;
    self.scope_level += 1;
    if is_toplevel {
      self.visit_class_custom(&node.class);
    } else {
      node.class.visit_children_with(self);
    }
    self.scope_level = scope_level;
    if is_pure_class {
      self.clear_symbol_if_is_top_level();
    }
  }

  fn visit_class_expr(&mut self, node: &ClassExpr) {
    if !self.is_enabled() {
      return;
    }
    let is_toplevel = self.is_toplevel();
    let scope_level = self.scope_level;
    self.scope_level += 1;
    if is_toplevel {
      self.visit_class_custom(&node.class);
    } else {
      node.class.visit_children_with(self);
    }
    self.scope_level = scope_level;
  }

  fn visit_ident(&mut self, ident: &Ident) {
    let is_toplevel_ident =
      if ident.span.ctxt != self.unresolved_ctxt && ident.span.ctxt != SyntaxContext::empty() {
        if self.is_toplevel() {
          self.top_level_ctxt_set.insert(ident.span.ctxt);
          true
        } else {
          self.top_level_ctxt_set.contains(&ident.span.ctxt)
        }
      } else {
        false
      };

    if let Some(ExtraSpanInfo::ReWriteUsedByExports) = self.rewrite_usage_span.get(&ident.span) {
      let span = ident.span;
      self.on_usage(Box::new(move |deps, used_by_exports| {
        let target_dep = deps.iter_mut().find(|item| item.is_span_equal(&span));
        if let Some(dep) = target_dep {
          dep.set_used_by_exports(used_by_exports);
        }
      }));
    };
    // imported binding isn't considered as a top level symbol.
    if self.import_map.contains_key(&ident.to_id()) {
      return;
    };
    if is_toplevel_ident {
      let usage = if let Some(symbol) = self.get_top_level_symbol() {
        InnerGraphMapUsage::TopLevel(symbol)
      } else {
        if self.in_named {
          return;
        }
        InnerGraphMapUsage::True
      };
      self.add_usage(ident.sym.clone(), usage);
    }
  }

  fn visit_var_declarator(&mut self, n: &VarDeclarator) {
    if !self.is_enabled() {
      return;
    }
    if let Pat::Ident(ident) = &n.name
      && let Some(box init) = &n.init
      && self.is_toplevel()
    {
      let symbol = ident.id.sym.clone();
      match init {
        Expr::Fn(_) | Expr::Arrow(_) | Expr::Lit(_) => {
          self.set_symbol_if_is_top_level(symbol);
          init.visit_children_with(self);
          self.clear_symbol_if_is_top_level();
        }
        Expr::Class(class) => {
          let is_pure = is_pure_class(&class.class, self.unresolved_ctxt, self.comments.as_ref());
          if is_pure {
            self.set_symbol_if_is_top_level(symbol);
          }
          class.visit_with(self);
          self.clear_symbol_if_is_top_level();
        }
        _ => {
          init.visit_children_with(self);
          if is_pure_expression(init, self.unresolved_ctxt, self.comments.as_ref()) {
            self.set_symbol_if_is_top_level(symbol);
            let start = init.span().real_lo();
            let end = init.span().real_hi();
            let module_identifier = self.state.module_identifier;
            self.on_usage(Box::new(
              move |deps, used_by_exports| match used_by_exports {
                Some(UsedByExports::Bool(true)) | None => {}
                _ => {
                  let mut dep = PureExpressionDependency::new(start, end, module_identifier);
                  dep.used_by_exports = used_by_exports;
                  deps.push(Box::new(dep));
                }
              },
            ));
            self.clear_symbol_if_is_top_level();
          }
        }
      }
    } else {
      n.visit_children_with(self);
    }
  }

  fn visit_prop(&mut self, n: &Prop) {
    if let Prop::Shorthand(shorthand) = n {
      if let Some(ExtraSpanInfo::ReWriteUsedByExports) =
        self.rewrite_usage_span.get(&shorthand.span)
      {
        let span = shorthand.span;
        self.on_usage(Box::new(move |deps, used_by_exports| {
          let target_dep = deps.iter_mut().find(|item| item.is_span_equal(&span));
          if let Some(dep) = target_dep {
            dep.set_used_by_exports(used_by_exports);
          }
        }));
      };
    }
    n.visit_children_with(self)
  }

  fn visit_export_decl(&mut self, export_decl: &ExportDecl) {
    let rewrite_usage_span = std::mem::take(self.rewrite_usage_span);
    if let Some(ExtraSpanInfo::AddVariableUsage(usages)) = rewrite_usage_span.get(&export_decl.span)
    {
      for (sym, usage) in usages {
        self.add_variable_usage(sym.clone(), InnerGraphMapUsage::Value(usage.clone()));
      }
    }
    *self.rewrite_usage_span = rewrite_usage_span;

    export_decl.visit_children_with(self);
  }

  fn visit_named_export(&mut self, named_export: &NamedExport) {
    if !self.is_enabled() {
      return;
    }
    let rewrite_usage_span = std::mem::take(self.rewrite_usage_span);
    if named_export.src.is_none() {
      if let Some(ExtraSpanInfo::AddVariableUsage(usages)) =
        rewrite_usage_span.get(&named_export.span)
      {
        for (sym, usage) in usages {
          self.add_variable_usage(sym.clone(), InnerGraphMapUsage::Value(usage.clone()));
        }
      }
    }
    *self.rewrite_usage_span = rewrite_usage_span;
    let in_named = self.in_named;
    self.in_named = true;
    named_export.visit_children_with(self);
    self.in_named = in_named;
  }
  fn visit_export_default_expr(&mut self, node: &ExportDefaultExpr) {
    if !self.is_enabled() {
      return;
    }
    let rewrite_usage_span = std::mem::take(self.rewrite_usage_span);
    if let Some(ExtraSpanInfo::AddVariableUsage(usages)) = rewrite_usage_span.get(&node.span) {
      for (sym, usage) in usages {
        self.add_variable_usage(sym.clone(), InnerGraphMapUsage::Value(usage.clone()));
      }
    }
    *self.rewrite_usage_span = rewrite_usage_span;

    let expr = node.expr.unwrap_parens();
    match expr {
      Expr::Fn(_) | Expr::Arrow(_) | Expr::Lit(_) => {
        self.set_symbol_if_is_top_level(DEFAULT_EXPORT.into());
        expr.visit_children_with(self);
        self.clear_symbol_if_is_top_level();
      }
      Expr::Class(ref class) => {
        let is_pure = is_pure_class(&class.class, self.unresolved_ctxt, self.comments.as_ref());
        if is_pure {
          self.set_symbol_if_is_top_level(DEFAULT_EXPORT.into());
        }
        class.visit_with(self);
        self.clear_symbol_if_is_top_level();
      }
      _ => {
        if is_pure_expression(expr, self.unresolved_ctxt, self.comments.as_ref()) {
          self.set_symbol_if_is_top_level(DEFAULT_EXPORT.into());
          let start = expr.span().real_lo();
          let end = expr.span().real_hi();
          let module_identifier = self.state.module_identifier;
          self.on_usage(Box::new(
            move |deps, used_by_exports| match used_by_exports {
              Some(UsedByExports::Bool(true)) | None => {}
              _ => {
                let mut dep = PureExpressionDependency::new(start, end, module_identifier);
                dep.used_by_exports = used_by_exports;
                deps.push(Box::new(dep));
              }
            },
          ));

          expr.visit_children_with(self);
          self.clear_symbol_if_is_top_level();
        } else {
          expr.visit_children_with(self);
        }
      }
    }
  }

  fn visit_export_default_decl(&mut self, node: &ExportDefaultDecl) {
    if !self.is_enabled() {
      return;
    }

    let rewrite_usage_span = std::mem::take(self.rewrite_usage_span);
    if let Some(ExtraSpanInfo::AddVariableUsage(usages)) = rewrite_usage_span.get(&node.span) {
      for (sym, usage) in usages {
        self.add_variable_usage(sym.clone(), InnerGraphMapUsage::Value(usage.clone()));
      }
    }
    *self.rewrite_usage_span = rewrite_usage_span;

    let ident = match &node.decl {
      DefaultDecl::Class(class) => class.ident.as_ref().map(|item| item.sym.clone()),
      DefaultDecl::Fn(func) => func.ident.as_ref().map(|item| item.sym.clone()),
      DefaultDecl::TsInterfaceDecl(_) => unreachable!(),
    }
    .unwrap_or(DEFAULT_EXPORT.into());

    self.set_symbol_if_is_top_level(ident);
    match &node.decl {
      DefaultDecl::Class(class) => {
        let is_pure = is_pure_class(&class.class, self.unresolved_ctxt, self.comments.as_ref());
        if !is_pure {
          self.set_top_level_symbol(None);
        }
        class.visit_with(self);
      }
      DefaultDecl::Fn(func) => {
        func.visit_with(self);
      }
      DefaultDecl::TsInterfaceDecl(_) => unreachable!(),
    }

    self.clear_symbol_if_is_top_level();
  }
}

impl<'a> InnerGraphPlugin<'a> {
  pub fn new(
    dependencies: &'a mut Vec<Box<dyn Dependency>>,
    unresolved_ctxt: SyntaxContext,
    top_level_ctxt: SyntaxContext,
    rewrite_usage_span: &'a mut HashMap<Span, ExtraSpanInfo>,
    import_map: &'a ImportMap,
    module_identifier: ModuleIdentifier,
    comments: Option<SwcComments>,
  ) -> Self {
    Self {
      dependencies,
      unresolved_ctxt,
      state: InnerGraphState {
        module_identifier,
        ..Default::default()
      },
      scope_level: 0,
      rewrite_usage_span,
      import_map,
      comments,
      in_named: false,
      top_level_ctxt_set: HashSet::from_iter([top_level_ctxt]),
    }
  }

  pub fn enable(&mut self) {
    self.state.enable = true;
  }

  fn is_toplevel(&self) -> bool {
    self.scope_level == 0
  }

  // fn has_toplevel_symbol(&self) -> bool {
  //   self.state.current_top_level_symbol.is_some()
  // }
  pub fn bailout(&mut self) {
    self.state.enable = false;
  }

  pub fn is_enabled(&self) -> bool {
    self.state.enable
  }

  pub fn add_usage(&mut self, symbol: Atom, usage: InnerGraphMapUsage) {
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

  pub fn add_variable_usage(&mut self, name: Atom, usage: InnerGraphMapUsage) {
    self.add_usage(name, usage);
  }

  pub fn on_usage(&mut self, on_usage_callback: UsageCallback) {
    if self.is_enabled() {
      if let Some(symbol) = self.get_top_level_symbol() {
        self
          .state
          .usage_callback_map
          .entry(symbol)
          .or_default()
          .push(on_usage_callback);
      } else {
        on_usage_callback(self.dependencies, Some(UsedByExports::Bool(true)));
      }
    } else {
      on_usage_callback(self.dependencies, None);
    }
  }

  pub fn visit_class_custom(&mut self, class: &Class) {
    if let Some(super_class) = &class.super_class
      && is_pure_expression(super_class, self.unresolved_ctxt, self.comments.as_ref())
    {
      let start = super_class.span().real_lo();
      let end = super_class.span().real_hi();
      let module_identifier = self.state.module_identifier;
      self.on_usage(Box::new(
        move |deps, used_by_exports| match used_by_exports {
          Some(UsedByExports::Bool(true)) | None => {}
          _ => {
            let mut dep = PureExpressionDependency::new(start, end, module_identifier);
            dep.used_by_exports = used_by_exports;
            deps.push(Box::new(dep));
          }
        },
      ));
    }
    class.visit_children_with(self);
  }

  pub fn set_top_level_symbol(&mut self, symbol: Option<Atom>) {
    self.state.current_top_level_symbol = symbol;
  }

  pub fn set_symbol_if_is_top_level(&mut self, symbol: Atom) {
    if self.is_toplevel() {
      self.set_top_level_symbol(Some(symbol));
    }
  }

  pub fn clear_symbol_if_is_top_level(&mut self) {
    if self.is_toplevel() {
      self.set_top_level_symbol(None);
    }
  }

  pub fn get_top_level_symbol(&self) -> Option<Atom> {
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
    let mut processed: HashMap<Atom, HashSet<InnerGraphMapSetValue>> = HashMap::default();

    // dbg!(state.module_identifier, &state.inner_graph,);
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

    // dbg!(state.module_identifier, &state.inner_graph,);
    for (symbol, cbs) in state.usage_callback_map.iter() {
      let usage = state.inner_graph.get(symbol);
      for cb in cbs {
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

        cb(self.dependencies, Some(used_by_exports));
      }
    }
  }
}
