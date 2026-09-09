use swc_next_ecma_ast::{
  ArrowFunctionBodyData, BindingIdentifier, FunctionBody, IdentifierReference, NodeId,
  NodeKindData, ScopeId, SymbolId,
};
use swc_next_ecma_semantic::{ReferenceSpace, SymbolFlags, scope::ScopeKind};

use super::{Atom, AtomRef, BindingState, ParsedJavaScriptAst, ScopeInfoDB, ScopeInfoId};

/// Immutable lookup information only; mutable binding state must be read again after hooks.
#[derive(Clone, Copy)]
pub enum IdentifierResolution {
  Symbol(SymbolId),
  Name,
  Unresolved,
}

impl<'ast> ScopeInfoDB<'ast> {
  pub fn with_semantic(ast: &'ast ParsedJavaScriptAst<'ast>) -> Self {
    let mut db = Self::new();
    db.ast = Some(ast);
    for entry in ast.semantic.iter_scopes() {
      let index = entry.scope.node.index();
      db.scopes_by_node
        .resize(db.scopes_by_node.len().max(index + 1), None);
      // Named function expressions can own multiple scopes. The last scope
      // created for a node is its innermost scope, used while walking the body.
      db.scopes_by_node[index] = Some(entry.id);
      if entry.scope.node == ast.program.node_id() {
        db.semantic_scope = entry.id;
      }
    }
    for entry in ast.semantic.iter_symbols() {
      if !entry.symbol.flags.contains(SymbolFlags::FUNCTION) {
        continue;
      }
      let owner = ast.semantic.scope(entry.symbol.scope);
      if matches!(owner.kind, ScopeKind::Block | ScopeKind::With)
        && matches!(
          ast.semantic.scope(owner.hoist_target).kind,
          ScopeKind::Global | ScopeKind::Module | ScopeKind::Function | ScopeKind::FunctionBody
        )
      {
        db.function_aliases.push((owner.hoist_target, entry.id));
      }
    }
    db.function_aliases
      .sort_unstable_by_key(|(scope, symbol)| (*scope, symbol.index()));
    db.create();
    db.initialize_scope_bindings(ast);
    db
  }

  pub(crate) fn owns_ast(&self, ast: &ParsedJavaScriptAst<'_>) -> bool {
    self.ast.is_some_and(|original| std::ptr::eq(original, ast))
  }

  pub fn enter_semantic_scope(&mut self, ast: &ParsedJavaScriptAst<'_>, node: NodeId) -> ScopeId {
    let previous = self.semantic_scope;
    if self.owns_ast(ast)
      && let Some(scope) = self.scopes_by_node.get(node.index()).copied().flatten()
    {
      self.semantic_scope = scope;
    }
    previous
  }

  pub fn leave_semantic_scope(&mut self, previous: ScopeId) {
    self.semantic_scope = previous;
  }

  pub(super) fn symbol_for_name(&self, name: &str) -> Option<SymbolId> {
    self
      .ast?
      .semantic
      .lookup(self.semantic_scope, name.as_bytes(), ReferenceSpace::Value)
  }

  fn symbol_state(&self, symbol: SymbolId) -> Option<BindingState> {
    self.symbols.get(symbol.index()).copied().flatten()
  }

  pub(super) fn set_symbol(&mut self, symbol: SymbolId, state: BindingState, scope: ScopeInfoId) {
    let index = symbol.index();
    if self.symbols.len() <= index {
      self.symbols.resize(index + 1, None);
    }
    if self.current != Some(scope) {
      // Outer writes persist even when an inner alias has queued a restore.
      // Inspect scope ancestry before borrowing the override log mutably.
      for i in 0..self.overrides.len() {
        let (overridden, _, owner) = self.overrides[i];
        if overridden == symbol && self.is_descendant_of(owner, scope) {
          self.overrides[i].1 = Some(state);
        }
      }
    }
    let previous = self.symbols[index];
    if previous == Some(state) {
      return;
    }
    if self.current == Some(scope) && !self.is_root_scope(scope) {
      self.overrides.push((symbol, previous, scope));
    }
    self.symbols[index] = Some(state);
  }

  pub fn resolve<'key>(&mut self, name: impl Into<AtomRef<'key>>) -> Option<BindingState> {
    let name = name.into();
    if let Some(symbol) = self.symbol_for_name(name.as_str()) {
      if let Some(state) = self.symbol_state(symbol) {
        return state.defined();
      }
      if let Some(state) = self.bind_existing_symbol(self.current_scope(), name, symbol) {
        return state.defined();
      }
    }
    self.get(self.current_scope(), name)
  }

  pub fn resolve_identifier(
    &mut self,
    parsed: &ParsedJavaScriptAst<'_>,
    identifier: IdentifierReference,
  ) -> Option<BindingState> {
    let resolution = self.identifier_resolution(parsed, identifier);
    self.resolve_identifier_with(parsed, identifier, resolution)
  }

  pub fn identifier_resolution(
    &self,
    parsed: &ParsedJavaScriptAst<'_>,
    identifier: IdentifierReference,
  ) -> IdentifierResolution {
    if !self.owns_ast(parsed) {
      return IdentifierResolution::Name;
    }
    let semantic = parsed.semantic;
    let Some(reference) = semantic.reference_of(identifier.node_id()) else {
      return IdentifierResolution::Name;
    };
    let reference = semantic.reference(reference);
    if reference.flags.is_dynamic() {
      return IdentifierResolution::Name;
    }
    reference.symbol.map_or(
      IdentifierResolution::Unresolved,
      IdentifierResolution::Symbol,
    )
  }

  pub fn resolve_identifier_with(
    &mut self,
    parsed: &ParsedJavaScriptAst<'_>,
    identifier: IdentifierReference,
    resolution: IdentifierResolution,
  ) -> Option<BindingState> {
    let ast = parsed.ast;
    if let IdentifierResolution::Name = resolution {
      return self.resolve(ast.get_utf8(identifier.name(ast)));
    }
    if let IdentifierResolution::Symbol(symbol) = resolution {
      if let Some(state) = self.symbol_state(symbol) {
        return state.defined();
      }
      let name = ast.get_utf8(identifier.name(ast));
      if let Some(state) = self.bind_existing_symbol(self.current_scope(), name, symbol) {
        return state.defined();
      }
      // Replacement trees and legacy scope mappings can still introduce a
      // binding through the name-based overlay.
    }
    self.get(self.current_scope(), ast.get_utf8(identifier.name(ast)))
  }

  pub fn define(&mut self, name: Atom) {
    let scope = self.current_scope();
    if let Some(state) = self.resolve(&name) {
      let info = self.expect_get_variable(state);
      if info.tag_info.is_some() && info.declared_scope == scope {
        return;
      }
    }
    self.set(scope, name, BindingState::Normal(scope));
  }

  pub fn define_identifier(
    &mut self,
    parsed: &ParsedJavaScriptAst<'_>,
    identifier: BindingIdentifier,
  ) {
    if self.owns_ast(parsed) {
      // Original declarations already have default bindings before hooks run.
      debug_assert!(
        parsed
          .semantic
          .symbol_of(identifier.node_id())
          .and_then(|symbol| self.symbol_state(symbol))
          .is_some(),
        "declaration binding should be initialized before hooks"
      );
      return;
    }
    // Replacement declarations are resolved by name, not by their fragment's SymbolId.
    self.define(Atom::from(parsed.ast.get_utf8(identifier.name(parsed.ast))));
  }

  pub fn initialize_scope_bindings(&mut self, parsed: &ParsedJavaScriptAst<'_>) {
    if !self.owns_ast(parsed) {
      return;
    }
    let scope = self.current_scope();
    let semantic = parsed.semantic;
    let mut semantic_scope = self.semantic_scope;
    let node = semantic.scope(semantic_scope).node;
    loop {
      for symbol in semantic.bindings(semantic_scope) {
        if semantic
          .symbol(symbol)
          .flags
          .visible_in(ReferenceSpace::Value)
          && self.symbol_state(symbol).is_none()
        {
          self.set_symbol(symbol, BindingState::Normal(scope), scope);
        }
      }
      // A named function/class expression has an additional name scope on
      // the same node. It belongs to the same Rspack scope as its body.
      let Some(parent) = semantic
        .scope(semantic_scope)
        .parent
        .filter(|parent| semantic.scope(*parent).node == node)
      else {
        break;
      };
      semantic_scope = parent;
    }
    // The legacy pre-walk made block functions visible by name in the
    // enclosing function/program. Derive only those extra names from semantic
    // scopes; ordinary function bindings are already initialized above.
    let start = self
      .function_aliases
      .partition_point(|(target, _)| *target < self.semantic_scope);
    for index in start..self.function_aliases.len() {
      let (target, symbol) = self.function_aliases[index];
      if target != self.semantic_scope {
        break;
      }
      let name = parsed
        .ast
        .get_wtf8(semantic.symbol(symbol).name)
        .to_string_lossy();
      if self
        .bindings
        .get(name.as_ref())
        .and_then(|bindings| bindings.last())
        .is_none_or(|binding| binding.scope != scope)
      {
        self.set(
          scope,
          Atom::from(name.as_ref()),
          BindingState::Normal(scope),
        );
      }
    }
  }

  pub fn initialize_function_body_bindings(
    &mut self,
    parsed: &ParsedJavaScriptAst<'_>,
    body: FunctionBody,
  ) {
    if self.owns_ast(parsed) {
      // A body without its own semantic scope was already initialized when
      // entering the function. Only parameter expressions create a separate
      // function-body environment whose bindings still need initialization.
      if self
        .scopes_by_node
        .get(body.node_id().index())
        .is_some_and(Option::is_some)
      {
        self.initialize_scope_bindings(parsed);
      }
      return;
    }
    // Replacement ASTs have their own semantic IDs. Consume their declaration
    // information too, but keep bindings in the existing name-based overlay.
    let ast = parsed.ast;
    let semantic = parsed.semantic;
    let Some(scope) = semantic
      .iter_scopes()
      .filter(|entry| match ast.kind_data(entry.scope.node) {
        NodeKindData::Function(function) => function.body(ast) == body,
        NodeKindData::ArrowFunctionExpression(arrow) => matches!(
          ast.arrow_function_body_data(arrow.body(ast)),
          ArrowFunctionBodyData::FunctionBody(function_body) if function_body == body
        ),
        NodeKindData::FunctionBody(function_body) => function_body == body,
        _ => false,
      })
      .last()
    else {
      return;
    };
    for entry in semantic.iter_symbols() {
      let symbol = entry.symbol;
      let owner = semantic.scope(symbol.scope);
      if symbol.flags.contains(SymbolFlags::FUNCTION)
        && owner.kind != ScopeKind::ExpressionName
        && owner.hoist_target == scope.id
      {
        let name = ast.get_wtf8(symbol.name).to_string_lossy();
        self.define(Atom::from(name.as_ref()));
      }
    }
  }

  pub fn suppress_scope_parameters(&mut self, parsed: &ParsedJavaScriptAst<'_>) {
    if !self.owns_ast(parsed) {
      return;
    }
    let scope = self.current_scope();
    for symbol in parsed.semantic.bindings(self.semantic_scope) {
      if parsed
        .semantic
        .symbol(symbol)
        .flags
        .contains(SymbolFlags::PARAMETER)
      {
        self.set_symbol(symbol, BindingState::Tombstone, scope);
      }
    }
  }

  pub fn current_variables(&self) -> Vec<(Atom, BindingState)> {
    let scope = self.current_scope();
    let mut variables = self
      .resolved_scope_variables(scope)
      .map(|(name, state)| (name.clone(), state))
      .collect::<Vec<_>>();
    let Some(parsed) = self.ast else {
      return variables;
    };
    for symbol in parsed.semantic.bindings(self.semantic_scope) {
      let Some(state) = self.symbol_state(symbol).and_then(BindingState::defined) else {
        continue;
      };
      if self.expect_get_variable(state).declared_scope != scope {
        continue;
      }
      let name = parsed.semantic.symbol(symbol).name;
      let name = parsed.ast.get_wtf8(name).to_string_lossy();
      // The overlay has already supplied visible metadata for names it owns,
      // including tombstones and persistent ancestor writes.
      if self.get_raw(scope, name.as_ref()).is_none() {
        variables.push((Atom::from(name.as_ref()), state));
      }
    }
    variables
  }
}
