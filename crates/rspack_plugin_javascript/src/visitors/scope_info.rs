use std::num::NonZeroU32;

use bitflags::bitflags;
use rspack_intern::{AtomMap, AtomRef};
use smallvec::SmallVec;
use swc_next_ecma_ast::{ScopeId, SymbolId};

use crate::{Atom, visitors::ParsedJavaScriptAst};

mod semantic;
pub use semantic::IdentifierResolution;

macro_rules! dense_id {
  ($name:ident) => {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct $name(NonZeroU32);

    impl $name {
      fn from_index(index: usize) -> Self {
        let value = u32::try_from(index)
          .ok()
          .and_then(|index| index.checked_add(1))
          .filter(|value| *value < u32::MAX - 1)
          .unwrap_or_else(|| panic!("too many {} entries", stringify!($name)));
        Self(NonZeroU32::new(value).expect("dense ids start at one"))
      }

      fn index(self) -> usize {
        (self.0.get() - 1) as usize
      }
    }
  };
}

dense_id!(ScopeInfoId);
dense_id!(VariableMetadataId);
dense_id!(TagInfoId);

/// A copyable snapshot, not a second identity for a static binding. Ordinary
/// variables carry their declaration scope directly; only cold metadata needs
/// another lookup. Aliases copy this state rather than follow another symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindingState {
  Normal(ScopeInfoId),
  Metadata(VariableMetadataId),
  Tombstone,
  Undefined,
}

impl BindingState {
  pub fn tombstone() -> Self {
    Self::Tombstone
  }
  pub fn undefined() -> Self {
    Self::Undefined
  }

  fn defined(self) -> Option<Self> {
    match self {
      Self::Normal(_) | Self::Metadata(_) => Some(self),
      Self::Tombstone | Self::Undefined => None,
    }
  }
}

#[derive(Debug, Default)]
pub struct TagInfoDB {
  pub map: Vec<TagInfo>,
}

impl TagInfoDB {
  fn new() -> Self {
    Self { map: Vec::new() }
  }
}

/// A binding of a name in one scope.
#[derive(Debug, Clone, Copy)]
struct Binding {
  scope: ScopeInfoId,
  target: BindingTarget,
}

#[derive(Debug, Clone, Copy)]
enum BindingTarget {
  Symbol(SymbolId),
  Local(BindingState),
}

/// Scoped symbol table.
///
/// The parser enters and exits scopes in strict stack order and reads through
/// the innermost active scope. `ScopeInfoDB` exploits this:
/// instead of one hash map per scope plus a parent-chain walk on lookup, it
/// keeps a single map from name to a stack of bindings (outermost first).
/// Lookup is a single hash probe; the innermost binding is the last element.
///
/// `get` must use the innermost active scope; writes may also target an active
/// ancestor when invalidating plugin tags. `create_child` must use the current
/// scope as parent, and child scopes must be exited in reverse creation order.
pub struct ScopeInfoDB<'ast> {
  // Entries are append-only for the lifetime of this parser, so IDs never
  // need generations or slot reuse.
  map: Vec<ScopeInfo>,
  /// For each name, the stack of active bindings, innermost last.
  bindings: AtomMap<SmallVec<[Binding; 2]>>,
  /// The innermost active scope, used to validate the stack discipline.
  current: Option<ScopeInfoId>,
  variable_metadata: Vec<VariableMetadata>,
  tag_info_db: TagInfoDB,
  // Read-only SWC semantics belong to the original module, never to temporary
  // expression trees. All mutable binding state is owned by this database.
  ast: Option<&'ast ParsedJavaScriptAst<'ast>>,
  scopes_by_node: Vec<Option<ScopeId>>,
  semantic_scope: ScopeId,
  symbols: Vec<Option<BindingState>>,
  overrides: Vec<(SymbolId, Option<BindingState>, ScopeInfoId)>,
}

impl Default for ScopeInfoDB<'_> {
  fn default() -> Self {
    Self::new()
  }
}

impl<'ast> ScopeInfoDB<'ast> {
  pub fn new() -> Self {
    Self {
      map: Vec::new(),
      bindings: AtomMap::default(),
      current: None,
      variable_metadata: Vec::new(),
      tag_info_db: TagInfoDB::new(),
      ast: None,
      scopes_by_node: Vec::new(),
      semantic_scope: ScopeId::ROOT,
      symbols: Vec::new(),
      overrides: Vec::new(),
    }
  }

  fn _create(&mut self, parent: Option<ScopeInfoId>) -> ScopeInfoId {
    let is_strict = match parent {
      Some(parent) => self.expect_get_scope(parent).is_strict,
      None => false,
    };
    let info = ScopeInfo {
      is_strict,
      parent,
      defined: Vec::new(),
      overrides_start: self.overrides.len(),
    };
    let id = ScopeInfoId::from_index(self.map.len());
    self.map.push(info);
    self.current = Some(id);
    id
  }

  pub fn create(&mut self) -> ScopeInfoId {
    self._create(None)
  }

  pub fn current_scope(&self) -> ScopeInfoId {
    self.current.expect("an active scope should exist")
  }

  pub fn is_root_scope(&self, id: ScopeInfoId) -> bool {
    self.expect_get_scope(id).parent.is_none()
  }

  pub fn is_descendant_of(&self, mut scope: ScopeInfoId, ancestor: ScopeInfoId) -> bool {
    while let Some(parent) = self.expect_get_scope(scope).parent {
      if parent == ancestor {
        return true;
      }
      scope = parent;
    }
    false
  }

  pub fn create_child(&mut self, parent: ScopeInfoId) -> ScopeInfoId {
    debug_assert_eq!(
      self.current,
      Some(parent),
      "scope must be entered from the innermost active scope"
    );
    self._create(Some(parent))
  }

  /// Exit `id`, dropping all bindings introduced in it. `id` must be the
  /// innermost active scope.
  pub fn exit_scope(&mut self, id: ScopeInfoId) {
    debug_assert_eq!(
      self.current,
      Some(id),
      "only the innermost active scope can be exited"
    );
    let scope = self.expect_get_mut_scope(id);
    let defined = std::mem::take(&mut scope.defined);
    let overrides_start = scope.overrides_start;
    self.current = scope.parent;
    for key in &defined {
      if let Some(stack) = self.bindings.get_mut(key)
        && let Some(top) = stack.last()
        && top.scope == id
      {
        stack.pop();
      }
    }
    self.expect_get_mut_scope(id).defined = defined;
    while self.overrides.len() > overrides_start {
      let (symbol, previous, _) = self.overrides.pop().expect("valid scope override range");
      self.symbols[symbol.index()] = previous;
    }
  }

  pub fn expect_get_scope(&self, id: ScopeInfoId) -> &ScopeInfo {
    self
      .map
      .get(id.index())
      .unwrap_or_else(|| panic!("{id:#?} should exist"))
  }

  pub fn expect_get_mut_scope(&mut self, id: ScopeInfoId) -> &mut ScopeInfo {
    self
      .map
      .get_mut(id.index())
      .unwrap_or_else(|| panic!("{id:#?} should exist"))
  }

  pub fn expect_get_variable(&self, state: BindingState) -> VariableInfo<'_> {
    match state {
      BindingState::Normal(declared_scope) => VariableInfo {
        state,
        declared_scope,
        name: None,
        flags: VariableInfoFlags::NORMAL,
        tag_info: None,
      },
      BindingState::Metadata(id) => {
        let info = &self.variable_metadata[id.index()];
        VariableInfo {
          state,
          declared_scope: info.declared_scope,
          name: info.name.as_ref(),
          flags: info.flags,
          tag_info: info.tag_info,
        }
      }
      BindingState::Tombstone | BindingState::Undefined => panic!("{state:#?} is not defined"),
    }
  }

  pub fn expect_get_tag_info(&self, id: TagInfoId) -> &TagInfo {
    self
      .tag_info_db
      .map
      .get(id.index())
      .unwrap_or_else(|| panic!("{id:#?} should exist"))
  }

  pub fn expect_get_mut_tag_info(&mut self, id: TagInfoId) -> &mut TagInfo {
    self
      .tag_info_db
      .map
      .get_mut(id.index())
      .unwrap_or_else(|| panic!("{id:#?} should exist"))
  }

  /// Resolve `key` starting from the innermost active scope `id`.
  pub fn get<'key>(
    &mut self,
    id: ScopeInfoId,
    key: impl Into<AtomRef<'key>>,
  ) -> Option<BindingState> {
    self.get_raw(id, key)?.defined()
  }

  pub fn get_raw<'key>(
    &self,
    id: ScopeInfoId,
    key: impl Into<AtomRef<'key>>,
  ) -> Option<BindingState> {
    debug_assert_eq!(
      self.current,
      Some(id),
      "lookup must start from the innermost active scope"
    );
    self.binding_state(self.bindings.get(key)?.last()?.target)
  }

  fn binding_state(&self, target: BindingTarget) -> Option<BindingState> {
    match target {
      BindingTarget::Symbol(symbol) => self.symbols.get(symbol.index()).copied().flatten(),
      BindingTarget::Local(state) => Some(state),
    }
  }

  fn declared_binding_state(&self, binding: &Binding) -> Option<BindingState> {
    let mut state = self.binding_state(binding.target);
    if let BindingTarget::Symbol(symbol) = binding.target {
      // Enumerating an active ancestor needs its own snapshot, not a temporary
      // override in a descendant (for example a separately parsed expression).
      for &(overridden, previous, owner) in self.overrides.iter().rev() {
        if overridden == symbol && self.is_descendant_of(owner, binding.scope) {
          state = previous;
        }
      }
    }
    state
  }

  fn bind_existing_symbol<'key>(
    &mut self,
    id: ScopeInfoId,
    key: impl Into<AtomRef<'key>>,
    symbol: SymbolId,
  ) -> Option<BindingState> {
    debug_assert_eq!(self.current, Some(id));
    let binding = self.bindings.get_mut(key)?.last_mut()?;
    if binding.scope != id {
      return None;
    }
    let state = match binding.target {
      BindingTarget::Symbol(symbol) => self.symbols.get(symbol.index()).copied().flatten()?,
      BindingTarget::Local(state) => state,
    };
    binding.target = BindingTarget::Symbol(symbol);
    self.set_symbol(symbol, state, id);
    Some(state)
  }

  pub fn set(&mut self, id: ScopeInfoId, key: Atom, state: BindingState) {
    let symbol = self.symbol_for_name(&key);
    let stack = self.bindings.entry(key.clone()).or_default();
    let symbol = if let Some(top) = stack.last_mut()
      && top.scope == id
    {
      let symbol = match top.target {
        BindingTarget::Symbol(symbol) => Some(symbol),
        BindingTarget::Local(_) => symbol,
      };
      top.target = symbol.map_or(BindingTarget::Local(state), BindingTarget::Symbol);
      symbol
    } else {
      stack.push(Binding {
        scope: id,
        target: symbol.map_or(BindingTarget::Local(state), BindingTarget::Symbol),
      });
      self.expect_get_mut_scope(id).defined.push(key.clone());
      symbol
    };
    if let Some(symbol) = symbol {
      self.set_symbol(symbol, state, id);
    }
    if self.current != Some(id) {
      // A persistent outer write also invalidates visible inner aliases, and
      // their pending restores must not resurrect the old tag on scope exit.
      let symbols = self
        .bindings
        .get(&key)
        .into_iter()
        .flatten()
        .filter_map(|binding| match binding.target {
          BindingTarget::Symbol(other) if Some(other) != symbol => Some(other),
          _ => None,
        })
        .collect::<SmallVec<[_; 2]>>();
      for symbol in symbols {
        self.set_symbol(symbol, state, id);
      }
    }
  }

  pub fn delete(&mut self, id: ScopeInfoId, key: &Atom) {
    self.set(id, key.clone(), BindingState::tombstone())
  }

  /// The variables bound in scope `id` itself (not in enclosing scopes).
  /// `id` must be an active scope.
  pub fn scope_variables(&self, id: ScopeInfoId) -> impl Iterator<Item = (&Atom, BindingState)> {
    self
      .scope_variable_bindings(id)
      .map(|(name, declared, _)| (name, declared))
  }

  /// Names bound in `id`, paired with their innermost visible variable metadata.
  pub fn resolved_scope_variables(
    &self,
    id: ScopeInfoId,
  ) -> impl Iterator<Item = (&Atom, BindingState)> {
    self
      .scope_variable_bindings(id)
      .filter_map(|(name, _, visible)| visible.defined().map(|visible| (name, visible)))
  }

  fn scope_variable_bindings(
    &self,
    id: ScopeInfoId,
  ) -> impl Iterator<Item = (&Atom, BindingState, BindingState)> {
    let scope = self.expect_get_scope(id);
    scope.defined.iter().filter_map(move |name| {
      let bindings = self.bindings.get(name)?;
      let binding = bindings.iter().rev().find(|binding| binding.scope == id)?;
      let value = self.declared_binding_state(binding)?;
      if value == BindingState::tombstone() {
        return None;
      }
      // Outer tag writes can leave a different binding visible above the
      // one owned by this scope. Reuse this probe for both kinds of metadata.
      let visible =
        self.binding_state(bindings.last().expect("the scope binding exists").target)?;
      Some((name, value, visible))
    })
  }
}

#[derive(Debug)]
pub struct TagInfo {
  pub tag: &'static str,
  pub data: Option<Box<dyn anymap::CloneAny>>,
  pub next: Option<TagInfoId>,
}

impl TagInfo {
  pub fn create(
    definitions_db: &mut ScopeInfoDB,
    tag: &'static str,
    data: Option<Box<dyn anymap::CloneAny>>,
    next: Option<TagInfoId>,
  ) -> TagInfoId {
    let tag_info = TagInfo { tag, data, next };
    let id = TagInfoId::from_index(definitions_db.tag_info_db.map.len());
    definitions_db.tag_info_db.map.push(tag_info);
    id
  }
}

bitflags! {
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
  pub struct VariableInfoFlags: u8 {
    const EVALUATED = 0b000;
    const FREE = 0b001;
    const NORMAL = 0b010;
    const TAGGED = 0b100;
  }
}

/// Similar to `VariableInfo` in webpack but more general.
/// For example, webpack will only store a string when both
/// `free_name` and `tag_info` are `None`, but we use `VariableInfo` instead.
#[derive(Debug, PartialEq, Eq)]
struct VariableMetadata {
  pub declared_scope: ScopeInfoId,

  /// `name` is alias name for free variable or tagged variable.
  ///
  /// For free variable:
  ///
  /// ```ignore
  /// let alias = require;
  /// ```
  ///
  /// The name for variable `alias` is `Some("require")`, so `call_hooks_name`
  /// will call the aliased name `"require"` for hooks.
  ///
  /// For tagged variable:
  ///
  /// ```ignore
  /// import { a } from "./m";
  /// a.b;
  /// ```
  ///
  /// The variable `a` is tagged as `ESM_SPECIFIER_TAG`, so `call_hooks_name`
  /// will call the aliased name `"a"` for hooks.
  pub name: Option<Atom>,

  pub flags: VariableInfoFlags,

  /// For example, if we want to bundle a case that has the same name as one
  /// already used in the rspack output, we must rename the argument
  /// `__rspack_require` to something else.
  ///
  /// ```ignore
  /// function f(__rspack_require) {
  ///  __rspack_require(something)
  /// }
  /// ```
  ///
  /// Firstly, it tries to define the argument `__rspack_require` as a
  /// normal variable (`free_name` and `tag_info` both `None`). However, it should
  /// invoke `Javascript::tag_variable` because it has the same name as the
  /// rspack runtime require.
  ///
  /// so the info about the argument `__rspack_require` becomes:
  ///
  /// ```ignore
  /// VariableInfo {
  ///   free_name: Some("__rspack_require"),
  ///   tag: Some(Tag {
  ///     tag: COMPACT_WEBPACK_RUNTIME_REQUIRE_IDENTIFIER,
  ///     data: SOME_DATA_TO_RENAME_THIS_IDENTIFIER
  ///   })
  /// }
  /// ```
  ///
  /// Then, when we encounter the callee `__rspack_require`,
  /// the `tag_info` will help us known how to handle it correctly.
  pub tag_info: Option<TagInfoId>,
}

/// A borrowed view of a binding snapshot. Normal variables are represented
/// directly, without allocating or looking up a metadata record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VariableInfo<'a> {
  state: BindingState,
  pub declared_scope: ScopeInfoId,
  pub name: Option<&'a Atom>,
  pub flags: VariableInfoFlags,
  pub tag_info: Option<TagInfoId>,
}

impl VariableInfo<'_> {
  pub fn create(
    definitions_db: &mut ScopeInfoDB,
    declared_scope: ScopeInfoId,
    name: Option<Atom>,
    flags: VariableInfoFlags,
    tag_info: Option<TagInfoId>,
  ) -> BindingState {
    if name.is_none() && flags == VariableInfoFlags::NORMAL && tag_info.is_none() {
      return BindingState::Normal(declared_scope);
    }
    let id = VariableMetadataId::from_index(definitions_db.variable_metadata.len());
    definitions_db.variable_metadata.push(VariableMetadata {
      declared_scope,
      name,
      flags,
      tag_info,
    });
    BindingState::Metadata(id)
  }

  pub fn snapshot(self) -> BindingState {
    self.state
  }

  pub fn is_free(&self) -> bool {
    self.flags.contains(VariableInfoFlags::FREE)
  }

  pub fn is_tagged(&self) -> bool {
    self.flags.contains(VariableInfoFlags::TAGGED)
  }
}

#[derive(Debug)]
pub struct ScopeInfo {
  parent: Option<ScopeInfoId>,
  /// Names bound in this scope, in definition order.
  defined: Vec<Atom>,
  overrides_start: usize,
  pub is_strict: bool,
}

#[cfg(test)]
mod tests {
  use super::{BindingState, ScopeInfoDB, VariableInfo, VariableInfoFlags};
  use crate::Atom;

  fn new_variable(db: &mut ScopeInfoDB, scope: super::ScopeInfoId) -> BindingState {
    VariableInfo::create(db, scope, None, VariableInfoFlags::NORMAL, None)
  }

  #[test]
  fn inner_scope_shadows_and_unwinds() {
    let mut db = ScopeInfoDB::new();
    let root = db.create();
    let a = Atom::from("a");
    let outer = new_variable(&mut db, root);
    db.set(root, "a".into(), outer);
    assert_eq!(db.get(root, &a), Some(outer));

    let child = db.create_child(root);
    assert_eq!(db.get(child, &a), Some(outer));

    let inner = new_variable(&mut db, child);
    db.set(child, "a".into(), inner);
    assert_eq!(db.get(child, &a), Some(inner));

    db.exit_scope(child);
    assert_eq!(db.get(root, &a), Some(outer));
  }

  #[test]
  fn delete_masks_outer_binding_until_exit() {
    let mut db = ScopeInfoDB::new();
    let root = db.create();
    let a = Atom::from("a");

    let outer = new_variable(&mut db, root);
    db.set(root, "a".into(), outer);

    let child = db.create_child(root);
    db.delete(child, &a);
    assert_eq!(db.get(child, &a), None);

    db.exit_scope(child);
    assert_eq!(db.get(root, &a), Some(outer));
  }

  #[test]
  fn scope_variables_skip_tombstones() {
    let mut db = ScopeInfoDB::new();
    let root = db.create();

    let a = new_variable(&mut db, root);
    db.set(root, "a".into(), a);
    let b = new_variable(&mut db, root);
    db.set(root, "b".into(), b);
    db.delete(root, &"b".into());

    let variables: Vec<_> = db
      .scope_variables(root)
      .map(|(name, id)| (name.as_str(), id))
      .collect();
    assert_eq!(variables, vec![("a", a)]);
  }
}
