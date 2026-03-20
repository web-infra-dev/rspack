use bitflags::bitflags;
use rustc_hash::FxHashMap;
use swc_core::atoms::Atom;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VariableInfoId(usize);

impl VariableInfoId {
  #[inline]
  fn from_index(index: usize) -> Self {
    Self(index)
  }

  #[inline]
  fn index(self) -> usize {
    self.0
  }

  #[inline]
  pub fn tombstone() -> Self {
    Self(usize::MAX)
  }

  #[inline]
  pub fn undefined() -> Self {
    Self(usize::MAX - 1)
  }

  #[inline]
  fn is_missing(self) -> bool {
    self == Self::tombstone() || self == Self::undefined()
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TagInfoId(usize);

impl TagInfoId {
  #[inline]
  fn from_index(index: usize) -> Self {
    Self(index)
  }

  #[inline]
  fn index(self) -> usize {
    self.0
  }
}

#[derive(Debug, Default)]
pub struct VariableInfoArena {
  entries: Vec<VariableInfo>,
}

impl VariableInfoArena {
  #[inline]
  fn new() -> Self {
    Self {
      entries: Default::default(),
    }
  }

  #[inline]
  fn insert_with_key(&mut self, f: impl FnOnce(VariableInfoId) -> VariableInfo) -> VariableInfoId {
    let id = VariableInfoId::from_index(self.entries.len());
    self.entries.push(f(id));
    id
  }
}

#[derive(Debug, Default)]
pub struct TagInfoArena {
  entries: Vec<TagInfo>,
}

impl TagInfoArena {
  fn new() -> Self {
    Self {
      entries: Default::default(),
    }
  }

  #[inline]
  fn insert(&mut self, tag_info: TagInfo) -> TagInfoId {
    let id = TagInfoId::from_index(self.entries.len());
    self.entries.push(tag_info);
    id
  }
}

#[derive(Debug)]
pub struct ScopeStack {
  scope_arena: Vec<ScopeInfo>,
  stack_len: usize,
  variable_arena: VariableInfoArena,
  tag_arena: TagInfoArena,
}

impl Default for ScopeStack {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl ScopeStack {
  #[inline]
  pub fn new() -> Self {
    Self {
      scope_arena: Default::default(),
      stack_len: 0,
      variable_arena: VariableInfoArena::new(),
      tag_arena: TagInfoArena::new(),
    }
  }

  #[inline]
  pub fn current_scope_level(&self) -> usize {
    debug_assert!(self.stack_len > 0, "scope should exist");
    self.stack_len - 1
  }

  #[inline]
  fn stack_view_mut(&mut self) -> &mut [ScopeInfo] {
    &mut self.scope_arena[..self.stack_len]
  }

  #[inline]
  pub fn initialize_root_scope(&mut self) {
    debug_assert_eq!(self.stack_len, 0);
    debug_assert!(self.scope_arena.is_empty());
    self.scope_arena.push(ScopeInfo::new(false));
    self.stack_len = 1;
  }

  #[inline]
  pub fn push_scope(&mut self) {
    let is_strict = self.current_scope().is_strict;
    if let Some(scope) = self.scope_arena.get_mut(self.stack_len) {
      scope.reset_for_reuse(is_strict);
    } else {
      self.scope_arena.push(ScopeInfo::new(is_strict));
    }
    self.stack_len += 1;
  }

  #[inline]
  pub fn pop_scope(&mut self) {
    debug_assert!(self.stack_len > 0, "cannot pop scope");
    self.stack_len -= 1;
  }

  #[inline]
  pub fn current_scope(&self) -> &ScopeInfo {
    debug_assert!(
      self.stack_len > 0,
      "scope stack is empty, call `initialize_root_scope` first"
    );
    unsafe { self.scope_arena.get_unchecked(self.stack_len - 1) }
  }

  #[inline]
  pub fn current_scope_mut(&mut self) -> &mut ScopeInfo {
    self
      .stack_view_mut()
      .last_mut()
      .expect("scope should exist")
  }

  #[inline]
  pub fn expect_get_variable(&self, id: VariableInfoId) -> &VariableInfo {
    self
      .variable_arena
      .entries
      .get(id.index())
      .unwrap_or_else(|| panic!("{id:#?} should exist"))
  }

  #[inline]
  pub fn expect_get_tag_info(&self, id: TagInfoId) -> &TagInfo {
    self
      .tag_arena
      .entries
      .get(id.index())
      .unwrap_or_else(|| panic!("{id:#?} should exist"))
  }

  #[inline]
  pub fn expect_get_mut_tag_info(&mut self, id: TagInfoId) -> &mut TagInfo {
    self
      .tag_arena
      .entries
      .get_mut(id.index())
      .unwrap_or_else(|| panic!("{id:#?} should exist"))
  }

  #[inline]
  fn resolve_variable(id: VariableInfoId) -> Option<VariableInfoId> {
    (!id.is_missing()).then_some(id)
  }

  #[inline]
  pub fn get(&mut self, key: &Atom) -> Option<VariableInfoId> {
    let (current_scope, parent_scopes) = self
      .stack_view_mut()
      .split_last_mut()
      .expect("scope should exist");

    if let Some(&variable_info_id) = current_scope.bindings.get(key) {
      return Self::resolve_variable(variable_info_id);
    }

    let found = parent_scopes
      .iter()
      .rev()
      .find_map(|scope| scope.bindings.get(key).copied());

    if let Some(variable_info_id) = found {
      current_scope.bindings.insert(key.clone(), variable_info_id);
      return Self::resolve_variable(variable_info_id);
    }

    if !parent_scopes.is_empty() {
      current_scope
        .bindings
        .insert(key.clone(), VariableInfoId::tombstone());
    }

    None
  }

  #[inline]
  pub fn set(&mut self, key: Atom, variable_info_id: VariableInfoId) {
    self
      .current_scope_mut()
      .bindings
      .insert(key, variable_info_id);
  }

  #[inline]
  pub fn delete(&mut self, key: &Atom) {
    let (current_scope, parent_scopes) = self
      .stack_view_mut()
      .split_last_mut()
      .expect("scope should exist");

    if !parent_scopes.is_empty() {
      current_scope
        .bindings
        .insert(key.clone(), VariableInfoId::tombstone());
    } else {
      current_scope.bindings.remove(key);
    }
  }
}

#[derive(Debug)]
pub struct TagInfo {
  pub tag: &'static str,
  pub data: Option<Box<dyn anymap::CloneAny>>,
  pub next: Option<TagInfoId>,
}

impl TagInfo {
  #[inline]
  pub fn create(
    scope_stack: &mut ScopeStack,
    tag: &'static str,
    data: Option<Box<dyn anymap::CloneAny>>,
    next: Option<TagInfoId>,
  ) -> TagInfoId {
    let tag_info = TagInfo { tag, data, next };
    scope_stack.tag_arena.insert(tag_info)
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
pub struct VariableInfo {
  id: VariableInfoId,
  pub declared_scope_level: usize,

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
  /// already used in the webpack output, we must rename the argument
  /// `__webpack_require__` to something else.
  ///
  /// ```ignore
  /// function f(__webpack_require__) {
  ///  __webpack_require__(something)
  /// }
  /// ```
  ///
  /// Firstly, it tries to define the argument `__webpack_require__` as a
  /// normal variable (`free_name` and `tag_info` both `None`). However, it should
  /// invoke `Javascript::tag_variable` because it has the same name as the
  /// webpack runtime require.
  ///
  /// so the info about the argument `__webpack_require__` becomes:
  ///
  /// ```ignore
  /// VariableInfo {
  ///   free_name: Some("__webpack_require__"),
  ///   tag: Some(Tag {
  ///     tag: COMPACT_WEBPACK_RUNTIME_REQUIRE_IDENTIFIER,
  ///     data: SOME_DATA_TO_RENAME_THIS_IDENTIFIER
  ///   })
  /// }
  /// ```
  ///
  /// Then, when we encounter the callee `__webpack_require__`,
  /// the `tag_info` will help us known how to handle it correctly.
  pub tag_info: Option<TagInfoId>,
}

impl VariableInfo {
  #[inline]
  pub fn create(
    scope_stack: &mut ScopeStack,
    declared_scope_level: usize,
    name: Option<Atom>,
    flags: VariableInfoFlags,
    tag_info: Option<TagInfoId>,
  ) -> VariableInfoId {
    scope_stack
      .variable_arena
      .insert_with_key(|id| VariableInfo {
        id,
        declared_scope_level,
        name,
        flags,
        tag_info,
      })
  }

  #[inline]
  pub fn id(&self) -> VariableInfoId {
    self.id
  }

  #[inline]
  pub fn is_free(&self) -> bool {
    self.flags.contains(VariableInfoFlags::FREE)
  }

  #[inline]
  pub fn is_tagged(&self) -> bool {
    self.flags.contains(VariableInfoFlags::TAGGED)
  }
}

#[derive(Debug)]
pub struct ScopeInfo {
  bindings: FxHashMap<Atom, VariableInfoId>,
  pub is_strict: bool,
}

impl ScopeInfo {
  #[inline]
  fn new(is_strict: bool) -> Self {
    Self {
      bindings: Default::default(),
      is_strict,
    }
  }

  #[inline]
  fn reset_for_reuse(&mut self, is_strict: bool) {
    self.bindings.clear();
    self.is_strict = is_strict;
  }

  #[inline]
  pub fn variables(&self) -> impl Iterator<Item = (&Atom, &VariableInfoId)> {
    self
      .bindings
      .iter()
      .filter(|&(_, &info_id)| !info_id.is_missing())
  }
}
