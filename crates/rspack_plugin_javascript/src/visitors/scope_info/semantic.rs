use swc_next_ecma_ast::{BindingIdentifier, IdentifierReference, NodeId, ScopeId, SymbolId};
use swc_next_ecma_semantic::{ReferenceSpace, SymbolFlags};

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
    db.create();
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
      // A plugin can deliberately suppress lexical parameters (require.ensure
      // is one example). A static resolution alone must not activate them.
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
    let ast = parsed.ast;
    if self.owns_ast(parsed)
      && let Some(symbol) = parsed.semantic.symbol_of(identifier.node_id())
    {
      let scope = self.current_scope();
      let name = ast.get_utf8(identifier.name(ast));
      let existing = self
        .bind_existing_symbol(scope, name, symbol)
        .or_else(|| self.symbol_state(symbol));
      if let Some(state) = existing.and_then(BindingState::defined) {
        let info = self.expect_get_variable(state);
        if info.tag_info.is_some() && info.declared_scope == scope {
          self.set_symbol(symbol, state, scope);
          return;
        }
      }
      self.set_symbol(symbol, BindingState::Normal(scope), scope);
    } else {
      self.define(Atom::from(ast.get_utf8(identifier.name(ast))));
    }
  }

  fn ordinary_declaration(flags: SymbolFlags) -> bool {
    flags.intersects(
      SymbolFlags::FUNCTION_SCOPED_VAR
        | SymbolFlags::BLOCK_SCOPED_VAR
        | SymbolFlags::FUNCTION
        | SymbolFlags::CLASS,
    ) && !flags
      .intersects(SymbolFlags::PARAMETER | SymbolFlags::CATCH_VAR | SymbolFlags::ANY_IMPORT)
  }

  pub fn activate_scope_bindings(&mut self, parsed: &ParsedJavaScriptAst<'_>) {
    if !self.owns_ast(parsed) {
      return;
    }
    let scope = self.current_scope();
    for symbol in parsed.semantic.bindings(self.semantic_scope) {
      if Self::ordinary_declaration(parsed.semantic.symbol(symbol).flags)
        && self.symbol_state(symbol).is_none()
      {
        self.set_symbol(symbol, BindingState::Normal(scope), scope);
      }
    }
  }

  pub fn pre_define_identifier(
    &mut self,
    parsed: &ParsedJavaScriptAst<'_>,
    identifier: BindingIdentifier,
  ) {
    if self.owns_ast(parsed)
      && let Some(symbol) = parsed.semantic.symbol_of(identifier.node_id())
      && parsed.semantic.scope_of(symbol) == self.semantic_scope
      && Self::ordinary_declaration(parsed.semantic.symbol(symbol).flags)
      && self
        .symbol_state(symbol)
        .is_none_or(|state| state == BindingState::Normal(self.current_scope()))
    {
      return;
    }
    // Parameters merged with var, scope mismatches and replacement ASTs retain
    // their explicit registration path; ordinary declarations activate in bulk.
    self.define_identifier(parsed, identifier);
  }

  pub fn define_function_declaration(
    &mut self,
    parsed: &ParsedJavaScriptAst<'_>,
    identifier: BindingIdentifier,
  ) {
    if self.owns_ast(parsed)
      && let Some(symbol) = parsed.semantic.symbol_of(identifier.node_id())
      && parsed.semantic.scope_of(symbol) != self.semantic_scope
    {
      // Keep the legacy visibility of block functions, including Annex B,
      // even when references outside the lexical block are unresolved by SWC.
      let ast = parsed.ast;
      self.define(Atom::from(ast.get_utf8(identifier.name(ast))));
    } else {
      self.pre_define_identifier(parsed, identifier);
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
