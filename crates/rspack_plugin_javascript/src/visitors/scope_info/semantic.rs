use std::num::NonZeroU32;

use swc_next_ecma_ast::{IdentifierReference, NodeId, ScopeId, SymbolId};
use swc_next_ecma_semantic::{ReferenceSpace, Semantic, SymbolFlags, scope::ScopeKind};

use super::{
  Atom, AtomRef, BindingState, ParsedJavaScriptAst, ScopeInfoDB, ScopeInfoId, SemanticScopeId,
  SymbolBinding,
};

/// Append-only registry of immutable semantic lookup data for all retained ASTs.
#[derive(Default)]
pub(crate) struct SemanticStore<'ast> {
  /// Per-AST lookup records, with the original module registered first.
  asts: Vec<AstSemantic<'ast>>,
  /// Total size of the disjoint symbol ranges reserved by registered ASTs.
  symbol_count: u32,
  /// Total size of the disjoint semantic-scope ranges reserved by registered ASTs.
  scope_count: u32,
}

/// Active semantic environment, separate from stored AST data and mutable bindings.
pub(crate) struct SemanticContext<'ast> {
  /// Index of the currently active AST in the semantic store.
  active_ast: usize,
  /// Cached active AST reference, avoiding a registry lookup per identifier.
  ast: Option<&'ast ParsedJavaScriptAst<'ast>>,
  /// Cached offset mapping active AST symbol indices into the binding-state array.
  symbol_start: u32,
  /// Cached offset mapping active AST scopes into the shared default-owner array.
  scope_start: u32,
  /// Saved caller (AST index, SWC scope) pairs for name lookup and context restoration.
  parents: Vec<(usize, ScopeId)>,
  /// Current SWC scope within the active AST, independent of the Rspack scope.
  pub(crate) scope: ScopeId,
}

impl Default for SemanticContext<'_> {
  /// Creates an inactive context, ready for the first AST to be registered.
  fn default() -> Self {
    Self {
      active_ast: 0,
      ast: None,
      symbol_start: 0,
      scope_start: 0,
      parents: Vec::new(),
      scope: ScopeId::ROOT,
    }
  }
}

/// Per-AST lookup data, immutable after registration and separate from mutable binding state.
struct AstSemantic<'ast> {
  /// Borrowed parsed AST and its existing SWC semantic analysis, without cloning either.
  ast: &'ast ParsedJavaScriptAst<'ast>,
  /// Innermost SWC scope owned by each AST node, used when entering walker scopes.
  scopes_by_node: Vec<Option<ScopeId>>,
  /// Block-function compatibility aliases sorted by their enclosing hoist scope.
  function_aliases: Vec<(ScopeId, SymbolId)>,
  /// Start of this AST's disjoint binding-state range in `ScopeInfoDB::symbols`.
  symbol_start: u32,
  /// Start of this AST's disjoint scope range in `ScopeInfoDB::semantic_scope_owners`.
  scope_start: u32,
  /// SWC program scope activated when this AST is entered.
  root_scope: ScopeId,
}

/// Maps a symbol and its semantic declaration scope into the database-wide ranges.
fn symbol_binding(
  semantic: &Semantic<'_>,
  symbol_start: u32,
  scope_start: u32,
  symbol: SymbolId,
) -> SymbolBinding {
  SymbolBinding {
    index: NonZeroU32::new(symbol_start + symbol.index() as u32 + 1)
      .expect("semantic symbol indices start at one"),
    scope: SemanticScopeId(
      NonZeroU32::new(scope_start + semantic.scope_of(symbol).raw())
        .expect("semantic scope indices start at one"),
    ),
  }
}

/// Immutable lookup information only; mutable binding state must be read again after hooks.
#[derive(Clone, Copy)]
pub(crate) enum IdentifierResolution {
  Symbol(SymbolId),
  Name,
  Unresolved,
}

impl<'ast> SemanticStore<'ast> {
  /// Registers semantic lookup tables and reserves a disjoint range for this AST's symbols.
  fn register_ast(&mut self, ast: &'ast ParsedJavaScriptAst<'ast>) -> usize {
    let mut info = AstSemantic {
      ast,
      scopes_by_node: Vec::new(),
      function_aliases: Vec::new(),
      symbol_start: self.symbol_count,
      scope_start: self.scope_count,
      root_scope: ScopeId::ROOT,
    };
    let mut scope_count = 0;
    for entry in ast.semantic.iter_scopes() {
      scope_count += 1;
      let index = entry.scope.node.index();
      info
        .scopes_by_node
        .resize(info.scopes_by_node.len().max(index + 1), None);
      // Named function expressions can own multiple scopes. The last scope
      // created for a node is its innermost scope, used while walking the body.
      info.scopes_by_node[index] = Some(entry.id);
      if entry.scope.node == ast.program.node_id() {
        info.root_scope = entry.id;
      }
    }
    self.scope_count = self
      .scope_count
      .checked_add(scope_count)
      .filter(|count| *count < u32::MAX - 1)
      .expect("too many semantic scopes");
    let mut symbol_count = 0;
    for entry in ast.semantic.iter_symbols() {
      symbol_count = symbol_count.max(entry.id.index() + 1);
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
        info.function_aliases.push((owner.hoist_target, entry.id));
      }
    }
    info
      .function_aliases
      .sort_unstable_by_key(|(scope, symbol)| (*scope, symbol.index()));
    let symbol_end = info
      .symbol_start
      .checked_add(u32::try_from(symbol_count).expect("too many semantic bindings"))
      .expect("too many semantic bindings");
    self.symbol_count = symbol_end;
    let index = self.asts.len();
    self.asts.push(info);
    index
  }

  /// Finds a value symbol in the active AST, then in saved fragment callers.
  #[inline(never)]
  pub(super) fn symbol_for_name(
    &self,
    context: &SemanticContext<'_>,
    name: &str,
  ) -> Option<SymbolBinding> {
    let semantic = context.ast?.semantic;
    semantic
      .lookup(context.scope, name.as_bytes(), ReferenceSpace::Value)
      .map(|symbol| symbol_binding(semantic, context.symbol_start, context.scope_start, symbol))
      .or_else(|| {
        if context.parents.is_empty() {
          None
        } else {
          self.symbol_in_parent_ast(context, name)
        }
      })
  }

  /// Keeps fragment-only caller lookup out of the inlined ordinary-name path.
  #[cold]
  #[inline(never)]
  fn symbol_in_parent_ast(
    &self,
    context: &SemanticContext<'_>,
    name: &str,
  ) -> Option<SymbolBinding> {
    context.parents.iter().rev().find_map(|&(ast, scope)| {
      let info = &self.asts[ast];
      info
        .ast
        .semantic
        .lookup(scope, name.as_bytes(), ReferenceSpace::Value)
        .map(|symbol| {
          symbol_binding(
            info.ast.semantic,
            info.symbol_start,
            info.scope_start,
            symbol,
          )
        })
    })
  }

  /// Returns the innermost semantic scope owned by a node in the active AST.
  pub(crate) fn scope_for_node(
    &self,
    context: &SemanticContext<'_>,
    node: NodeId,
  ) -> Option<ScopeId> {
    self.asts[context.active_ast]
      .scopes_by_node
      .get(node.index())
      .copied()
      .flatten()
  }
}

impl<'ast> SemanticContext<'ast> {
  /// Activates a fragment's root scope, saving its caller for lookup and restoration.
  fn enter_ast(&mut self, store: &SemanticStore<'ast>, index: usize) {
    self.parents.push((self.active_ast, self.scope));
    self.set_ast(store, index);
    self.scope = store.asts[index].root_scope;
  }

  /// Restores the caller's AST and semantic scope after processing a fragment.
  fn leave_ast(&mut self, store: &SemanticStore<'ast>) {
    let (index, scope) = self.parents.pop().expect("active fragment");
    self.set_ast(store, index);
    self.scope = scope;
  }

  /// Switches the active AST and refreshes the cached reference and symbol offset.
  fn set_ast(&mut self, store: &SemanticStore<'ast>, index: usize) {
    let info = &store.asts[index];
    self.active_ast = index;
    self.ast = Some(info.ast);
    self.symbol_start = info.symbol_start;
    self.scope_start = info.scope_start;
  }

  /// Classifies an identifier using semantic data without reading mutable binding state.
  #[inline]
  pub(crate) fn identifier_resolution(
    &self,
    parsed: &ParsedJavaScriptAst<'_>,
    identifier: IdentifierReference,
  ) -> IdentifierResolution {
    debug_assert!(self.ast.is_some_and(|ast| std::ptr::eq(ast, parsed)));
    let semantic = parsed.semantic;
    let Some(reference) = semantic.reference_of(identifier.node_id()) else {
      return IdentifierResolution::Name;
    };
    let reference = semantic.reference(reference);
    if reference.flags.is_dynamic() {
      return IdentifierResolution::Name;
    }
    match reference.symbol {
      Some(symbol) => IdentifierResolution::Symbol(symbol),
      // A fragment is evaluated in its caller's environment, not as a separate module.
      None if !self.parents.is_empty() => IdentifierResolution::Name,
      None => IdentifierResolution::Unresolved,
    }
  }
}

impl<'ast> ScopeInfoDB<'ast> {
  /// Creates the module scope and initializes its bindings from the original AST's semantic data.
  pub fn with_semantic(ast: &'ast ParsedJavaScriptAst<'ast>) -> Self {
    let mut db = Self::new();
    let index = db.register_ast(ast);
    db.semantic_context.set_ast(&db.semantic, index);
    db.semantic_context.scope = db.semantic.asts[index].root_scope;
    db.create();
    db.initialize_scope_bindings();
    db
  }

  /// Registers an AST and allocates its initially unset binding-state slots.
  pub(crate) fn register_ast(&mut self, ast: &'ast ParsedJavaScriptAst<'ast>) -> usize {
    let index = self.semantic.register_ast(ast);
    self
      .symbols
      .resize(self.semantic.symbol_count as usize, None);
    self
      .semantic_scope_owners
      .resize(self.semantic.scope_count as usize, None);
    index
  }

  /// Activates a registered fragment and initializes its bindings in the current Rspack scope.
  pub(crate) fn enter_ast(&mut self, index: usize) {
    self.semantic_context.enter_ast(&self.semantic, index);
    self.initialize_scope_bindings();
  }

  /// Restores the caller's semantic context; Rspack scope exit handles binding rollback separately.
  pub(crate) fn leave_ast(&mut self) {
    self.semantic_context.leave_ast(&self.semantic);
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
    let symbol = self
      .semantic
      .symbol_for_name(&self.semantic_context, name.as_str());
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
    if let IdentifierResolution::Name = resolution {
      return self.resolve_identifier_fallback(parsed, identifier, resolution);
    }
    if let IdentifierResolution::Symbol(symbol) = resolution {
      let index = self.semantic_context.symbol_start as usize + symbol.index();
      if let Some(state) = self.symbols[index] {
        return state.defined();
      }
      let scope = self.semantic_context.scope_start as usize
        + parsed.semantic.scope_of(symbol).raw() as usize
        - 1;
      if let Some(owner) = self.semantic_scope_owners[scope] {
        return Some(BindingState::Normal(owner));
      }
      return self.resolve_identifier_fallback(parsed, identifier, resolution);
    }
    self.get(self.current_scope(), ast.get_utf8(identifier.name(ast)))
  }

  /// Shares dynamic-name and inactive-symbol handling instead of inlining it at every use.
  #[cold]
  #[inline(never)]
  fn resolve_identifier_fallback(
    &mut self,
    parsed: &ParsedJavaScriptAst<'_>,
    identifier: IdentifierReference,
    resolution: IdentifierResolution,
  ) -> Option<BindingState> {
    let ast = parsed.ast;
    let name = ast.get_utf8(identifier.name(ast));
    if let IdentifierResolution::Name = resolution {
      return self.resolve(name);
    }
    if let IdentifierResolution::Symbol(symbol) = resolution {
      let symbol = symbol_binding(
        parsed.semantic,
        self.semantic_context.symbol_start,
        self.semantic_context.scope_start,
        symbol,
      );
      if let Some(state) = self.bind_existing_symbol(self.current_scope(), name, symbol) {
        return state.defined();
      }
      // Replacement trees and legacy scope mappings can still introduce a
      // binding through the name-based overlay.
    }
    self.get(self.current_scope(), name)
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

  /// Activates shared default owners and initializes legacy block-function aliases.
  pub fn initialize_scope_bindings(&mut self) {
    let active_ast = self.semantic_context.active_ast;
    let parsed = self.semantic_context.ast.expect("active AST");
    let scope = self.current_scope();
    let semantic = parsed.semantic;
    let semantic_scope = self.semantic_context.scope;
    let node = semantic.scope(semantic_scope).node;

    // Activate only scopes owned by this node, including expression-name scopes.
    for semantic_scope in std::iter::successors(Some(semantic_scope), |scope| {
      semantic
        .scope(*scope)
        .parent
        .filter(|parent| semantic.scope(*parent).node == node)
    }) {
      let id = SemanticScopeId::from_index(
        self.semantic_context.scope_start as usize + semantic_scope.raw() as usize - 1,
      );
      if self.semantic_scope_owners[id.index()].is_none() {
        self.semantic_scope_owners[id.index()] = Some(scope);
        self.scope_activations.push(id);
      }
    }

    // Preserve legacy block-function aliases for this scope, not its ancestors.
    let aliases = &self.semantic.asts[active_ast].function_aliases;
    let start = aliases.partition_point(|(target, _)| *target < semantic_scope);
    for index in start..aliases.len() {
      let (target, symbol) = self.semantic.asts[active_ast].function_aliases[index];
      if target != semantic_scope {
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

  /// Hides callback parameter bindings to preserve require.ensure's free-name handling.
  pub fn suppress_scope_parameters(&mut self) {
    let parsed = self.semantic_context.ast.expect("active AST");
    let scope = self.current_scope();
    for symbol in parsed.semantic.bindings(self.semantic_context.scope) {
      if parsed
        .semantic
        .symbol(symbol)
        .flags
        .contains(SymbolFlags::PARAMETER)
      {
        let index = symbol_binding(
          parsed.semantic,
          self.semantic_context.symbol_start,
          self.semantic_context.scope_start,
          symbol,
        );
        self.set_symbol(index, BindingState::Tombstone, scope);
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
    let Some(info) = self.semantic.asts.get(self.semantic_context.active_ast) else {
      return variables;
    };
    let parsed = info.ast;
    for symbol in parsed.semantic.bindings(self.semantic_context.scope) {
      if !parsed
        .semantic
        .symbol(symbol)
        .flags
        .visible_in(ReferenceSpace::Value)
      {
        continue;
      }
      let Some(state) = self
        .symbol_state(symbol_binding(
          parsed.semantic,
          info.symbol_start,
          info.scope_start,
          symbol,
        ))
        .and_then(BindingState::defined)
      else {
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
