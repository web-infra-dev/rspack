use std::num::NonZeroU32;

use bitflags::bitflags;
use rustc_hash::FxHashMap;
use swc_core::atoms::Atom;

macro_rules! define_entity_id {
  ($name:ident) => {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct $name(NonZeroU32);

    impl $name {
      pub fn new(id: u32) -> Self {
        Self(NonZeroU32::new(id + 1).unwrap())
      }
      pub fn index(&self) -> usize {
        (self.0.get() - 1) as usize
      }
    }
  };
}

define_entity_id!(ScopeInfoId);
define_entity_id!(VariableInfoId);
define_entity_id!(TagInfoId);

#[derive(Debug)]
pub struct VariableInfoDB {
  map: Vec<VariableInfo>,
}

impl VariableInfoDB {
  fn new() -> Self {
    let mut map = Vec::with_capacity(2);
    map.push(VariableInfo {
      id: VariableInfo::TOMBSTONE,
      declared_scope: ScopeInfoId::new(0),
      name: None,
      flags: VariableInfoFlags::empty(),
      tag_info: None,
    });
    map.push(VariableInfo {
      id: VariableInfo::UNDEFINED,
      declared_scope: ScopeInfoId::new(1),
      name: None,
      flags: VariableInfoFlags::empty(),
      tag_info: None,
    });
    Self { map }
  }
}

#[derive(Debug)]
pub struct TagInfoDB {
  map: Vec<TagInfo>,
}

impl TagInfoDB {
  fn new() -> Self {
    Self {
      map: Default::default(),
    }
  }
}

#[derive(Debug)]
pub struct ScopeInfoDB {
  map: Vec<ScopeInfo>,
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
    let mut map = Vec::with_capacity(2);
    map.push(ScopeInfo {
      parent: None,
      map: Default::default(),
      is_strict: false,
    });
    map.push(ScopeInfo {
      parent: None,
      map: Default::default(),
      is_strict: false,
    });
    Self {
      map,
      variable_info_db: VariableInfoDB::new(),
      tag_info_db: TagInfoDB::new(),
    }
  }

  fn _create(&mut self, parent: Option<ScopeInfoId>) -> ScopeInfoId {
    let id = ScopeInfoId::new(self.map.len() as u32);
    let is_strict = match parent {
      Some(parent) => self.expect_get_scope(parent).is_strict,
      None => false,
    };
    let info = ScopeInfo {
      is_strict,
      parent,
      map: Default::default(),
    };
    self.map.push(info);
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
      .get(id.index())
      .unwrap_or_else(|| panic!("{id:#?} should exist"))
  }

  pub fn expect_get_mut_scope(&mut self, id: ScopeInfoId) -> &mut ScopeInfo {
    self
      .map
      .get_mut(id.index())
      .unwrap_or_else(|| panic!("{id:#?} should exist"))
  }

  pub fn expect_get_variable(&self, id: VariableInfoId) -> &VariableInfo {
    self
      .variable_info_db
      .map
      .get(id.index())
      .unwrap_or_else(|| panic!("{id:#?} should exist"))
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

  pub fn get(&mut self, id: ScopeInfoId, key: &Atom) -> Option<VariableInfoId> {
    let definitions = self.expect_get_scope(id);
    if let Some(&top_value) = definitions.map.get(key) {
      if top_value == VariableInfo::TOMBSTONE || top_value == VariableInfo::UNDEFINED {
        None
      } else {
        Some(top_value)
      }
    } else if let Some(parent) = definitions.parent {
      let mut current = Some(parent);
      while let Some(current_id) = current {
        let scope = self.expect_get_scope(current_id);
        if let Some(&value) = scope.map.get(key) {
          if value == VariableInfo::TOMBSTONE || value == VariableInfo::UNDEFINED {
            return None;
          } else {
            return Some(value);
          }
        }
        current = scope.parent;
      }
      let definitions = self.expect_get_mut_scope(id);
      definitions.map.insert(key.clone(), VariableInfo::TOMBSTONE);
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
      scope.map.insert(key.clone(), VariableInfo::TOMBSTONE);
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
    let id = TagInfoId::new(definitions_db.tag_info_db.map.len() as u32);
    let tag_info = TagInfo { tag, data, next };
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
  const TOMBSTONE: VariableInfoId =
    VariableInfoId(unsafe { std::num::NonZeroU32::new_unchecked(1) });
  const UNDEFINED: VariableInfoId =
    VariableInfoId(unsafe { std::num::NonZeroU32::new_unchecked(2) });

  pub fn create(
    definitions_db: &mut ScopeInfoDB,
    declared_scope: ScopeInfoId,
    name: Option<Atom>,
    flags: VariableInfoFlags,
    tag_info: Option<TagInfoId>,
  ) -> VariableInfoId {
    let id = VariableInfoId::new(definitions_db.variable_info_db.map.len() as u32);
    let variable_info = VariableInfo {
      id,
      declared_scope,
      name,
      flags,
      tag_info,
    };
    definitions_db.variable_info_db.map.push(variable_info);
    id
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
  pub parent: Option<ScopeInfoId>,
  map: FxHashMap<Atom, VariableInfoId>,
  pub is_strict: bool,
}

impl ScopeInfo {
  pub fn variables(&self) -> impl Iterator<Item = (&str, &VariableInfoId)> {
    self
      .map
      .iter()
      .filter(|&(_, &info_id)| info_id != VariableInfo::TOMBSTONE)
      .map(|(name, info_id)| (name.as_str(), info_id))
  }
}
