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
pub struct VariableInfoDB {
  map: Vec<VariableInfo>,
}

impl VariableInfoDB {
  #[inline]
  fn new() -> Self {
    Self {
      map: Default::default(),
    }
  }

  #[inline]
  fn insert_with_key(&mut self, f: impl FnOnce(VariableInfoId) -> VariableInfo) -> VariableInfoId {
    let id = VariableInfoId::from_index(self.map.len());
    self.map.push(f(id));
    id
  }
}

#[derive(Debug, Default)]
pub struct TagInfoDB {
  pub map: Vec<TagInfo>,
}

impl TagInfoDB {
  fn new() -> Self {
    Self {
      map: Default::default(),
    }
  }

  #[inline]
  fn insert(&mut self, tag_info: TagInfo) -> TagInfoId {
    let id = TagInfoId::from_index(self.map.len());
    self.map.push(tag_info);
    id
  }
}

#[derive(Debug)]
pub struct ScopeStack {
  stack: Vec<ScopeInfo>,
  variable_info_db: VariableInfoDB,
  tag_info_db: TagInfoDB,
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
      stack: Default::default(),
      variable_info_db: VariableInfoDB::new(),
      tag_info_db: TagInfoDB::new(),
    }
  }

  #[inline]
  pub fn current_level(&self) -> usize {
    self.stack.len() - 1
  }

  #[inline]
  fn push(&mut self, is_strict: bool) {
    self.stack.push(ScopeInfo {
      is_strict,
      map: Default::default(),
    });
  }

  #[inline]
  pub fn create(&mut self) {
    debug_assert!(self.stack.is_empty());
    self.push(false);
  }

  #[inline]
  pub fn push_scope(&mut self) {
    let is_strict = self.current_scope().is_strict;
    self.push(is_strict);
  }

  #[inline]
  pub fn pop_scope(&mut self) {
    debug_assert!(self.stack.len() > 1);
    self.stack.pop();
  }

  #[inline]
  pub fn current_scope(&self) -> &ScopeInfo {
    self.stack.last().expect("active scope should exist")
  }

  #[inline]
  pub fn current_scope_mut(&mut self) -> &mut ScopeInfo {
    self.stack.last_mut().expect("active scope should exist")
  }

  #[inline]
  pub fn expect_get_variable(&self, id: VariableInfoId) -> &VariableInfo {
    self
      .variable_info_db
      .map
      .get(id.index())
      .unwrap_or_else(|| panic!("{id:#?} should exist"))
  }

  #[inline]
  pub fn expect_get_tag_info(&self, id: TagInfoId) -> &TagInfo {
    self
      .tag_info_db
      .map
      .get(id.index())
      .unwrap_or_else(|| panic!("{id:#?} should exist"))
  }

  #[inline]
  pub fn expect_get_mut_tag_info(&mut self, id: TagInfoId) -> &mut TagInfo {
    self
      .tag_info_db
      .map
      .get_mut(id.index())
      .unwrap_or_else(|| panic!("{id:#?} should exist"))
  }

  #[inline]
  fn resolve_variable(id: VariableInfoId) -> Option<VariableInfoId> {
    (!id.is_missing()).then_some(id)
  }

  pub fn get(&mut self, key: &Atom) -> Option<VariableInfoId> {
    if let Some(value) = self.current_scope().map.get(key) {
      return Self::resolve_variable(*value);
    }

    let resolved = (0..self.stack.len() - 1)
      .rev()
      .find_map(|level| self.stack[level].map.get(key).copied());

    if let Some(value) = resolved {
      let top_scope = self.current_scope_mut();
      top_scope.map.insert(key.clone(), value);
      return Self::resolve_variable(value);
    }

    if self.stack.len() > 1 {
      let top_scope = self.current_scope_mut();
      top_scope
        .map
        .insert(key.clone(), VariableInfoId::tombstone());
    }

    None
  }

  pub fn set(&mut self, key: Atom, variable_info_id: VariableInfoId) {
    let scope = self.current_scope_mut();
    scope.map.insert(key, variable_info_id);
  }

  pub fn delete(&mut self, key: &Atom) {
    if self.current_level() > 0 {
      let scope = self.current_scope_mut();
      scope.map.insert(key.clone(), VariableInfoId::tombstone());
    } else {
      let scope = self.current_scope_mut();
      scope.map.remove(key);
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
  pub fn create(
    scope_stack: &mut ScopeStack,
    tag: &'static str,
    data: Option<Box<dyn anymap::CloneAny>>,
    next: Option<TagInfoId>,
  ) -> TagInfoId {
    let tag_info = TagInfo { tag, data, next };
    scope_stack.tag_info_db.insert(tag_info)
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
      .variable_info_db
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
  map: FxHashMap<Atom, VariableInfoId>,
  pub is_strict: bool,
}

impl ScopeInfo {
  pub fn variables(&self) -> impl Iterator<Item = (&str, &VariableInfoId)> {
    self
      .map
      .iter()
      .filter(|&(_, &info_id)| !info_id.is_missing())
      .map(|(name, info_id)| (name.as_str(), info_id))
  }
}
