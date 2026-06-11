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
    let is_strict = match parent {
      Some(parent) => self.expect_get_scope(parent).is_strict,
      None => false,
    };
    let info = ScopeInfo {
      is_strict,
      parent,
      map: Default::default(),
    };
    self.map.insert(info)
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

  pub fn get(&mut self, id: ScopeInfoId, key: &Atom) -> Option<VariableInfoId> {
    let definitions = self.expect_get_scope(id);
    if let Some(&top_value) = definitions.map.get(key) {
      if top_value == VariableInfoId::tombstone() || top_value == VariableInfoId::undefined() {
        None
      } else {
        Some(top_value)
      }
    } else if let Some(parent) = definitions.parent {
      let mut current = Some(parent);
      while let Some(current_id) = current {
        let scope = self.expect_get_scope(current_id);
        if let Some(&value) = scope.map.get(key) {
          if value == VariableInfoId::tombstone() || value == VariableInfoId::undefined() {
            return None;
          } else {
            return Some(value);
          }
        }
        current = scope.parent;
      }
      let definitions = self.expect_get_mut_scope(id);
      definitions
        .map
        .insert(key.clone(), VariableInfoId::tombstone());
      None
    } else {
      None
    }
  }

  pub fn set(&mut self, id: ScopeInfoId, key: Atom, variable_info_id: VariableInfoId) {
    let scope = self.expect_get_mut_scope(id);
    scope.map.insert(key, variable_info_id);
  }

  pub fn delete(&mut self, id: ScopeInfoId, key: &Atom) {
    let scope = self.expect_get_mut_scope(id);
    if scope.parent.is_some() {
      scope.map.insert(key.clone(), VariableInfoId::tombstone());
    } else {
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
  map: FxHashMap<Atom, VariableInfoId>,
  pub is_strict: bool,
}

impl ScopeInfo {
  pub fn variables(&self) -> impl Iterator<Item = (&str, &VariableInfoId)> {
    self
      .map
      .iter()
      .filter(|&(_, &info_id)| info_id != VariableInfoId::tombstone())
      .map(|(name, info_id)| (name.as_str(), info_id))
  }
}
