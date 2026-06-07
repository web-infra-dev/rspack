use bitflags::bitflags;
use rustc_hash::FxHashMap;
use slotmap::{KeyData, SlotMap, new_key_type};
use swc_core::atoms::Atom;

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

#[derive(Debug)]
pub struct ScopeInfoDB {
  map: SlotMap<ScopeInfoId, ScopeInfo>,
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
      variable_info_db: VariableInfoDB::new(),
      tag_info_db: TagInfoDB::new(),
    }
  }

  fn _create(&mut self, parent: Option<ScopeInfoId>) -> ScopeInfoId {
    let (is_strict, flattened_map) = match parent {
      Some(parent) => {
        let parent_scope = self.expect_get_scope(parent);
        (parent_scope.is_strict, parent_scope.flattened_map.clone())
      }
      None => (false, Default::default()),
    };
    let info = ScopeInfo {
      is_strict,
      parent,
      flattened_map,
      children: Default::default(),
      local_overrides: Default::default(),
      declared_variables: Default::default(),
    };
    let id = self.map.insert(info);
    if let Some(parent) = parent {
      self.expect_get_mut_scope(parent).children.push(id);
    }
    id
  }

  pub fn create(&mut self) -> ScopeInfoId {
    self._create(None)
  }

  pub fn create_child(&mut self, parent: ScopeInfoId) -> ScopeInfoId {
    self._create(Some(parent))
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

  pub fn get(&self, id: ScopeInfoId, key: &Atom) -> Option<VariableInfoId> {
    let value = self.expect_get_scope(id).flattened_map.get(key).copied()?;
    if value == VariableInfoId::tombstone() || value == VariableInfoId::undefined() {
      None
    } else {
      Some(value)
    }
  }

  pub fn set(&mut self, id: ScopeInfoId, key: Atom, variable_info_id: VariableInfoId) {
    let scope = self.expect_get_mut_scope(id);
    let propagate_key = key.clone();
    scope.flattened_map.insert(key.clone(), variable_info_id);
    scope.local_overrides.insert(key.clone(), variable_info_id);
    scope.declared_variables.insert(key, variable_info_id);
    self.propagate_to_children(id, &propagate_key, Some(variable_info_id));
  }

  pub fn delete(&mut self, id: ScopeInfoId, key: &Atom) {
    let scope = self.expect_get_mut_scope(id);
    scope.declared_variables.remove(key);
    if scope.parent.is_some() {
      scope
        .flattened_map
        .insert(key.clone(), VariableInfoId::tombstone());
      scope
        .local_overrides
        .insert(key.clone(), VariableInfoId::tombstone());
      self.propagate_to_children(id, key, Some(VariableInfoId::tombstone()));
    } else {
      scope.flattened_map.remove(key);
      scope.local_overrides.remove(key);
      self.propagate_to_children(id, key, None);
    }
  }

  fn propagate_to_children(
    &mut self,
    id: ScopeInfoId,
    key: &Atom,
    variable_info_id: Option<VariableInfoId>,
  ) {
    let children = self.expect_get_scope(id).children.clone();
    for child in children {
      let child_scope = self.expect_get_mut_scope(child);
      if child_scope.local_overrides.contains_key(key) {
        continue;
      }
      match variable_info_id {
        Some(variable_info_id) => {
          child_scope
            .flattened_map
            .insert(key.clone(), variable_info_id);
        }
        None => {
          child_scope.flattened_map.remove(key);
        }
      }
      self.propagate_to_children(child, key, variable_info_id);
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
  flattened_map: FxHashMap<Atom, VariableInfoId>,
  children: Vec<ScopeInfoId>,
  local_overrides: FxHashMap<Atom, VariableInfoId>,
  declared_variables: FxHashMap<Atom, VariableInfoId>,
  pub is_strict: bool,
}

impl ScopeInfo {
  pub fn variables(&self) -> impl Iterator<Item = (&str, &VariableInfoId)> {
    self
      .declared_variables
      .iter()
      .filter(|&(_, &info_id)| info_id != VariableInfoId::tombstone())
      .map(|(name, info_id)| (name.as_str(), info_id))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn variable(db: &mut ScopeInfoDB, scope: ScopeInfoId) -> VariableInfoId {
    VariableInfo::create(db, scope, None, VariableInfoFlags::NORMAL, None)
  }

  #[test]
  fn child_scope_reads_parent_binding_without_recursive_lookup() {
    let mut db = ScopeInfoDB::new();
    let root = db.create();
    let root_value = variable(&mut db, root);
    db.set(root, "require".into(), root_value);

    let child = db.create_child(root);

    assert_eq!(db.get(child, &"require".into()), Some(root_value));
  }

  #[test]
  fn child_delete_shadows_parent_binding() {
    let mut db = ScopeInfoDB::new();
    let root = db.create();
    let root_value = variable(&mut db, root);
    let name = Atom::from("require");
    db.set(root, name.clone(), root_value);

    let child = db.create_child(root);
    db.delete(child, &name);

    assert_eq!(db.get(child, &name), None);
    assert_eq!(db.get(root, &name), Some(root_value));
  }

  #[test]
  fn parent_mutation_after_child_creation_updates_child_flattened_scope() {
    let mut db = ScopeInfoDB::new();
    let root = db.create();
    let child = db.create_child(root);
    let grand_child = db.create_child(child);
    let root_value = variable(&mut db, root);
    let name = Atom::from("require");

    db.set(root, name.clone(), root_value);

    assert_eq!(db.get(child, &name), Some(root_value));
    assert_eq!(db.get(grand_child, &name), Some(root_value));
  }

  #[test]
  fn child_override_blocks_parent_mutation_propagation() {
    let mut db = ScopeInfoDB::new();
    let root = db.create();
    let child = db.create_child(root);
    let grand_child = db.create_child(child);
    let child_value = variable(&mut db, child);
    let root_value = variable(&mut db, root);
    let name = Atom::from("require");

    db.set(child, name.clone(), child_value);
    db.set(root, name.clone(), root_value);

    assert_eq!(db.get(child, &name), Some(child_value));
    assert_eq!(db.get(grand_child, &name), Some(child_value));
  }

  #[test]
  fn variables_only_returns_declarations_from_current_scope() {
    let mut db = ScopeInfoDB::new();
    let root = db.create();
    let root_value = variable(&mut db, root);
    db.set(root, "parent".into(), root_value);

    let child = db.create_child(root);
    let child_value = variable(&mut db, child);
    db.set(child, "child".into(), child_value);

    let scope = db.expect_get_scope(child);
    let variables = scope.variables().collect::<Vec<_>>();

    assert_eq!(variables, vec![("child", &child_value)]);
  }
}
