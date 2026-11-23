use std::num::NonZeroU32;

use swc_next_ecma_ast::{Ast, BindingIdentifier, IdentifierReference, ScopeId, SymbolId};
use swc_next_ecma_semantic::{ReferenceSpace, Semantic, SymbolFlags};

use super::{
  Atom, AtomRef, BindingState, ParsedJavaScriptAst, ScopeInfoDB, ScopeInfoId, SemanticScopeId,
  SymbolBinding, TagInfoDB,
};

/// Maps a module symbol to its override slot and semantic declaration scope.
fn symbol_binding(semantic: &Semantic<'_>, symbol: SymbolId) -> SymbolBinding {
  SymbolBinding {
    index: NonZeroU32::new(symbol.index() as u32 + 1)
      .expect("semantic symbol indices start at one"),
    scope: SemanticScopeId(
      NonZeroU32::new(semantic.scope_of_symbol(symbol).raw())
        .expect("semantic scope indices start at one"),
    ),
  }
}

#[inline(never)]
pub(super) fn symbol_for_name(
  semantic: &Semantic<'_>,
  scope: ScopeId,
  name: &str,
) -> Option<SymbolBinding> {
  // These roots and the synthetic default export live in the name-based overlay.
  if matches!(name, "this" | "import.meta" | "new.target" | "*default*") {
    return None;
  }
  let symbol = semantic.lookup(scope, name.as_bytes(), ReferenceSpace::Value)?;
  Some(symbol_binding(semantic, symbol))
}

/// Immutable lookup information only; mutable binding state must be read again after hooks.
#[derive(Clone, Copy)]
pub(crate) enum IdentifierResolution {
  Symbol(SymbolId),
  Name,
  Unresolved,
}

impl ParsedJavaScriptAst<'_> {
  /// Eval fragments have no semantic data and resolve names in the caller's scope.
  #[inline]
  pub(crate) fn identifier_resolution(
    &self,
    identifier: IdentifierReference,
  ) -> IdentifierResolution {
    let Some(semantic) = self.semantic else {
      return IdentifierResolution::Name;
    };
    let Some(reference) = semantic.reference_of(identifier.node_id()) else {
      return IdentifierResolution::Name;
    };
    match semantic.reference(reference).symbol {
      Some(symbol) => IdentifierResolution::Symbol(symbol),
      None => IdentifierResolution::Unresolved,
    }
  }
}

impl<'ast> ScopeInfoDB<'ast> {
  /// Creates the module scope and initializes bindings from the module's semantic data.
  pub fn new(ast: &'ast Ast<'ast>, semantic: &'ast Semantic<'ast>) -> Self {
    let semantic_scope = semantic
      .scope_of(ast.root_program().node_id())
      .expect("program has a semantic scope");
    let scope_count = semantic.iter_scopes().count();
    assert!(
      scope_count < (u32::MAX - 1) as usize,
      "too many semantic scopes"
    );
    let symbol_count = semantic
      .iter_symbols()
      .map(|entry| entry.id.index() + 1)
      .max()
      .unwrap_or_default();
    assert!(
      u32::try_from(symbol_count).is_ok(),
      "too many semantic bindings"
    );
    let mut db = Self {
      map: Vec::new(),
      bindings: Default::default(),
      current: None,
      variable_metadata: Vec::new(),
      tag_info_db: TagInfoDB::new(),
      module_ast: ast,
      module_semantic: semantic,
      semantic_scope,
      symbols: vec![None; symbol_count],
      semantic_scope_owners: vec![None; scope_count],
      scope_activations: Vec::new(),
      overrides: Vec::new(),
    };
    db.create_scope(None);
    db.initialize_scope_bindings();
    db
  }

  /// Reads an override or derives the default state from the symbol's active semantic scope.
  #[inline]
  pub(super) fn symbol_state(&self, symbol: SymbolBinding) -> Option<BindingState> {
    self.symbols[symbol.index()]
      .or_else(|| self.semantic_scope_owners[symbol.scope.index()].map(BindingState::Normal))
  }

  /// Updates a symbol while preserving scope-exit restoration and persistent outer writes.
  pub(super) fn set_symbol(
    &mut self,
    symbol: SymbolBinding,
    state: BindingState,
    scope: ScopeInfoId,
  ) {
    let index = symbol.index();
    let previous = self.symbols[index];
    if self.current != Some(scope) {
      // Outer writes persist even when an inner alias has queued a restore.
      let mut restore = previous;
      let mut recorded = false;
      let mut changed_restore = false;
      for i in (0..self.overrides.len()).rev() {
        let (overridden, previous, owner) = self.overrides[i];
        if overridden != symbol {
          continue;
        }
        if owner == scope {
          recorded = true;
        } else if self.is_descendant_of(owner, scope) {
          restore = previous;
          self.overrides[i].1 = Some(state);
          changed_restore = true;
        }
      }
      if !changed_restore && self.symbol_state(symbol) == Some(state) {
        return;
      }
      if !recorded && self.expect_get_scope(scope).parent.is_some() {
        // Keep the ancestor's restore before its descendants' ranges, so both
        // scope exit and ancestor enumeration retain their stack ordering.
        let start = self.expect_get_scope(scope).overrides_start;
        self.overrides.insert(start, (symbol, restore, scope));
        let mut child = self.current_scope();
        while child != scope {
          let info = self.expect_get_mut_scope(child);
          info.overrides_start += 1;
          child = info.parent.expect("outer writes target an active ancestor");
        }
      }
    } else {
      if self.symbol_state(symbol) == Some(state) {
        return;
      }
      if self.expect_get_scope(scope).parent.is_some() {
        self.overrides.push((symbol, previous, scope));
      }
    }
    self.symbols[index] = Some(state);
  }

  /// Resolves a name through semantic symbols, falling back to the name-based overlay.
  pub fn resolve<'key>(&mut self, name: impl Into<AtomRef<'key>>) -> Option<BindingState> {
    self.resolve_with_symbol(name).0
  }

  /// Keeps the immutable symbol lookup for an immediate read-modify-write operation.
  #[inline(always)]
  pub(crate) fn resolve_with_symbol<'key>(
    &mut self,
    name: impl Into<AtomRef<'key>>,
  ) -> (Option<BindingState>, Option<SymbolBinding>) {
    let name = name.into();
    let symbol = symbol_for_name(self.module_semantic, self.semantic_scope, name.as_str());
    if let Some(symbol) = symbol {
      if let Some(state) = self.symbol_state(symbol) {
        return (state.defined(), Some(symbol));
      }
      if let Some(state) = self.bind_existing_symbol(self.current_scope(), name, symbol) {
        return (state.defined(), Some(symbol));
      }
    }
    (self.get(self.current_scope(), name), symbol)
  }

  /// Reads the current binding state from a saved resolution, using name lookup when needed.
  #[inline(always)]
  pub(crate) fn resolve_identifier(
    &mut self,
    parsed: &ParsedJavaScriptAst<'_>,
    identifier: IdentifierReference,
    resolution: IdentifierResolution,
  ) -> Option<BindingState> {
    let ast = parsed.ast;
    match resolution {
      IdentifierResolution::Name => {
        self.resolve_identifier_name(ast.get_utf8(identifier.name(ast)))
      }
      IdentifierResolution::Symbol(symbol) => {
        if let Some(state) = self.symbols[symbol.index()] {
          return state.defined();
        }
        let semantic = self.module_semantic;
        let scope = semantic.scope_of_symbol(symbol).raw() as usize - 1;
        if let Some(owner) = self.semantic_scope_owners[scope] {
          return Some(BindingState::Normal(owner));
        }
        self.resolve_inactive_symbol(
          ast.get_utf8(identifier.name(ast)),
          symbol_binding(semantic, symbol),
        )
      }
      IdentifierResolution::Unresolved => {
        self.get(self.current_scope(), ast.get_utf8(identifier.name(ast)))
      }
    }
  }

  #[cold]
  #[inline(never)]
  fn resolve_identifier_name(&mut self, name: &str) -> Option<BindingState> {
    self.resolve(name)
  }

  #[cold]
  #[inline(never)]
  fn resolve_inactive_symbol(&mut self, name: &str, symbol: SymbolBinding) -> Option<BindingState> {
    let scope = self.current_scope();
    if let Some(state) = self.bind_existing_symbol(scope, name, symbol) {
      return state.defined();
    }
    self.get(scope, name)
  }

  /// Reuses a declaration's semantic symbol while still observing mutable tags and aliases.
  pub(crate) fn resolve_binding(
    &mut self,
    parsed: &ParsedJavaScriptAst<'_>,
    identifier: BindingIdentifier,
  ) -> (Option<BindingState>, Option<SymbolBinding>) {
    if let Some(semantic) = parsed.semantic
      && let Some(symbol) = semantic.symbol_of(identifier.node_id())
    {
      let symbol = symbol_binding(semantic, symbol);
      if let Some(state) = self.symbol_state(symbol) {
        return (state.defined(), Some(symbol));
      }
    }
    // Pre-walked block declarations may not have an active semantic owner yet.
    self.resolve_with_symbol(parsed.ast.get_utf8(identifier.name(parsed.ast)))
  }

  /// Defines a normal binding in the current scope without replacing its existing plugin tags.
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

  /// Activates shared default owners for the current scope.
  pub fn initialize_scope_bindings(&mut self) {
    let scope = self.current_scope();
    let id = SemanticScopeId::from_index(self.semantic_scope.raw() as usize - 1);
    if self.semantic_scope_owners[id.index()].is_none() {
      self.semantic_scope_owners[id.index()] = Some(scope);
      self.scope_activations.push(id);
    }
  }

  /// Hides callback parameter bindings to preserve require.ensure's free-name handling.
  pub fn suppress_scope_parameters(&mut self) {
    let semantic = self.module_semantic;
    let scope = self.current_scope();
    for symbol in semantic.bindings(self.semantic_scope) {
      if semantic
        .symbol(symbol)
        .flags
        .contains(SymbolFlags::PARAMETER)
      {
        self.set_symbol(
          symbol_binding(semantic, symbol),
          BindingState::Tombstone,
          scope,
        );
      }
    }
  }

  /// Collects current-scope variables from the name overlay and semantic bindings.
  pub fn current_variables(&self) -> Vec<(Atom, BindingState)> {
    let scope = self.current_scope();
    let mut variables = self
      .scope_variable_bindings(scope)
      .filter_map(|(name, _, visible)| visible.defined().map(|state| (name.clone(), state)))
      .collect::<Vec<_>>();
    let semantic = self.module_semantic;
    for symbol in semantic.bindings(self.semantic_scope) {
      if !semantic
        .symbol(symbol)
        .flags
        .visible_in(ReferenceSpace::Value)
      {
        continue;
      }
      let Some(state) = self
        .symbol_state(symbol_binding(semantic, symbol))
        .and_then(BindingState::defined)
      else {
        continue;
      };
      if self.expect_get_variable(state).declared_scope != scope {
        continue;
      }
      let name = semantic.symbol(symbol).name;
      let name = self.module_ast.get_wtf8(name).to_string_lossy();
      // The overlay has already supplied visible metadata for names it owns,
      // including tombstones and persistent ancestor writes.
      if self.get_raw(scope, name.as_ref()).is_none() {
        variables.push((Atom::from(name.as_ref()), state));
      }
    }
    variables
  }
}
