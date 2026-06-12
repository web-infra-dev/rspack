use bitflags::bitflags;
use rustc_hash::FxHashMap;
use slotmap::{KeyData, SlotMap, new_key_type};
use smallvec::SmallVec;
use swc_atoms::Atom;

new_key_type! {
  pub struct ScopeInfoId;
  pub struct VariableInfoId;
  pub struct TagInfoId;
}

impl VariableInfoId {
  pub fn tombstone() -> Self {
    Self::from(KeyData::from_ffi(u64::MAX))
  }
  pub fn undefined() -> Self {
    Self::from(KeyData::from_ffi(u64::MAX - 1))
  }
}

#[derive(Debug, Default)]
pub struct VariableInfoDB {
  map: SlotMap<VariableInfoId, VariableInfo>,
}

impl VariableInfoDB {
  fn new() -> Self {
    Self {
      map: SlotMap::with_key(),
    }
  }
}

#[derive(Debug, Default)]
pub struct TagInfoDB {
  pub map: SlotMap<TagInfoId, TagInfo>,
}

impl TagInfoDB {
  fn new() -> Self {
    Self {
      map: SlotMap::with_key(),
    }
  }
}

/// A binding of a name in one scope.
#[derive(Debug, Clone, Copy)]
struct Binding {
  scope: ScopeInfoId,
  value: VariableInfoId,
}

/// Scoped symbol table.
///
/// The parser enters and exits scopes in strict stack order and always reads
/// and writes through the innermost active scope. `ScopeInfoDB` exploits this:
/// instead of one hash map per scope plus a parent-chain walk on lookup, it
/// keeps a single map from name to a stack of bindings (outermost first).
/// Lookup is a single hash probe; the innermost binding is the last element.
///
/// Invariant: `get`/`set`/`delete` must be called with the innermost active
/// scope, `create_child` must be called with the current scope as parent, and
/// every scope created by `create_child` must be exited with `exit_scope`
/// before its parent receives further operations.
#[derive(Debug)]
pub struct ScopeInfoDB {
  map: SlotMap<ScopeInfoId, ScopeInfo>,
  /// For each name, the stack of active bindings, innermost last.
  bindings: FxHashMap<Atom, SmallVec<[Binding; 2]>>,
  /// The innermost active scope, used to validate the stack discipline.
  current: Option<ScopeInfoId>,
  variable_info_db: VariableInfoDB,
  tag_info_db: TagInfoDB,
}

impl Default for ScopeInfoDB {
  fn default() -> Self {
    Self::new()
  }
}

impl ScopeInfoDB {
  pub fn new() -> Self {
    Self {
      map: SlotMap::with_key(),
      bindings: FxHashMap::default(),
      current: None,
      variable_info_db: VariableInfoDB::new(),
      tag_info_db: TagInfoDB::new(),
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
    };
    let id = self.map.insert(info);
    self.current = Some(id);
    id
  }

  pub fn create(&mut self) -> ScopeInfoId {
    self._create(None)
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
    self.current = scope.parent;
    for key in &defined {
      if let Some(stack) = self.bindings.get_mut(key)
        && let Some(top) = stack.last()
        && top.scope == id
      {
        stack.pop();
      }
    }
    // Keep the names for `scope_variables` of scopes that are re-read after
    // walking (only the root scope in practice, which is never exited).
    self.expect_get_mut_scope(id).defined = defined;
  }

  pub fn expect_get_scope(&self, id: ScopeInfoId) -> &ScopeInfo {
    self
      .map
      .get(id)
      .unwrap_or_else(|| panic!("{id:#?} should exist"))
  }

  pub fn expect_get_mut_scope(&mut self, id: ScopeInfoId) -> &mut ScopeInfo {
    self
      .map
      .get_mut(id)
      .unwrap_or_else(|| panic!("{id:#?} should exist"))
  }

  pub fn expect_get_variable(&self, id: VariableInfoId) -> &VariableInfo {
    self
      .variable_info_db
      .map
      .get(id)
      .unwrap_or_else(|| panic!("{id:#?} should exist"))
  }

  pub fn expect_get_tag_info(&self, id: TagInfoId) -> &TagInfo {
    self
      .tag_info_db
      .map
      .get(id)
      .unwrap_or_else(|| panic!("{id:#?} should exist"))
  }

  pub fn expect_get_mut_tag_info(&mut self, id: TagInfoId) -> &mut TagInfo {
    self
      .tag_info_db
      .map
      .get_mut(id)
      .unwrap_or_else(|| panic!("{id:#?} should exist"))
  }

  /// Resolve `key` starting from the innermost active scope `id`.
  pub fn get(&mut self, id: ScopeInfoId, key: &Atom) -> Option<VariableInfoId> {
    debug_assert_eq!(
      self.current,
      Some(id),
      "lookup must start from the innermost active scope"
    );
    let binding = self.bindings.get(key)?.last()?;
    let value = binding.value;
    if value == VariableInfoId::tombstone() || value == VariableInfoId::undefined() {
      None
    } else {
      Some(value)
    }
  }

  pub fn set(&mut self, id: ScopeInfoId, key: Atom, variable_info_id: VariableInfoId) {
    debug_assert_eq!(
      self.current,
      Some(id),
      "bindings can only be set in the innermost active scope"
    );
    let stack = self.bindings.entry(key.clone()).or_default();
    if let Some(top) = stack.last_mut()
      && top.scope == id
    {
      top.value = variable_info_id;
      return;
    }
    stack.push(Binding {
      scope: id,
      value: variable_info_id,
    });
    self.expect_get_mut_scope(id).defined.push(key);
  }

  pub fn delete(&mut self, id: ScopeInfoId, key: &Atom) {
    self.set(id, key.clone(), VariableInfoId::tombstone());
  }

  /// The variables bound in scope `id` itself (not in enclosing scopes).
  /// `id` must be an active scope.
  pub fn scope_variables(&self, id: ScopeInfoId) -> impl Iterator<Item = (&str, VariableInfoId)> {
    let scope = self.expect_get_scope(id);
    scope.defined.iter().filter_map(move |name| {
      let binding = self
        .bindings
        .get(name)?
        .iter()
        .rev()
        .find(|binding| binding.scope == id)?;
      (binding.value != VariableInfoId::tombstone()).then_some((name.as_str(), binding.value))
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
    definitions_db.tag_info_db.map.insert(tag_info)
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
  pub fn create(
    definitions_db: &mut ScopeInfoDB,
    declared_scope: ScopeInfoId,
    name: Option<Atom>,
    flags: VariableInfoFlags,
    tag_info: Option<TagInfoId>,
  ) -> VariableInfoId {
    definitions_db
      .variable_info_db
      .map
      .insert_with_key(|id| VariableInfo {
        id,
        declared_scope,
        name,
        flags,
        tag_info,
      })
  }

  pub fn id(&self) -> VariableInfoId {
    self.id
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
  pub is_strict: bool,
}

#[cfg(test)]
mod tests {
  use super::{ScopeInfoDB, VariableInfo, VariableInfoFlags, VariableInfoId};

  fn new_variable(db: &mut ScopeInfoDB, scope: super::ScopeInfoId) -> VariableInfoId {
    VariableInfo::create(db, scope, None, VariableInfoFlags::NORMAL, None)
  }

  #[test]
  fn inner_scope_shadows_and_unwinds() {
    let mut db = ScopeInfoDB::new();
    let root = db.create();
    let a = "a".into();

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
    let a = "a".into();

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

    let variables: Vec<_> = db.scope_variables(root).collect();
    assert_eq!(variables, vec![("a", a)]);
  }
}
