use rustc_hash::FxHashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeInfoId(u32);

impl ScopeInfoId {
  fn init() -> ScopeInfoId {
    // tombstone -> ScopeInfoId(0)
    // undefined -> ScopeInfoId(1)
    ScopeInfoId(2)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VariableInfoId(u32);

impl VariableInfoId {
  fn init() -> VariableInfoId {
    // VariableInfoId(0) -> ScopeInfoId(0)
    // VariableInfoId(1) -> ScopeInfoId(1)
    VariableInfoId(2)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TagInfoId(u32);

impl TagInfoId {
  fn init() -> TagInfoId {
    TagInfoId(0)
  }
}

#[derive(Debug)]
pub struct VariableInfoDB {
  count: VariableInfoId,
  map: FxHashMap<VariableInfoId, VariableInfo>,
}

impl VariableInfoDB {
  fn next(&mut self) -> VariableInfoId {
    let id = self.count;
    self.count.0 += 1;
    id
  }

  fn new() -> Self {
    Self {
      count: VariableInfoId::init(),
      map: Default::default(),
    }
  }
}

#[derive(Debug)]
pub struct TagInfoDB {
  count: TagInfoId,
  map: FxHashMap<TagInfoId, TagInfo>,
}

impl TagInfoDB {
  fn next(&mut self) -> TagInfoId {
    let id = self.count;
    self.count.0 += 1;
    id
  }

  fn new() -> Self {
    Self {
      count: TagInfoId::init(),
      map: Default::default(),
    }
  }
}

#[derive(Debug)]
pub struct ScopeInfoDB {
  count: ScopeInfoId,
  map: FxHashMap<ScopeInfoId, ScopeInfo>,
  variable_info_db: VariableInfoDB,
  tag_info_db: TagInfoDB,
}

impl Default for ScopeInfoDB {
  fn default() -> Self {
    Self::new()
  }
}

impl ScopeInfoDB {
  fn next(&mut self) -> ScopeInfoId {
    let id = self.count;
    self.count.0 += 1;
    id
  }

  pub fn new() -> Self {
    Self {
      count: ScopeInfoId::init(),
      map: Default::default(),
      variable_info_db: VariableInfoDB::new(),
      tag_info_db: TagInfoDB::new(),
    }
  }

  fn _create(&mut self, parent: Option<&ScopeInfoId>) -> ScopeInfoId {
    let id = self.next();
    let stack = match parent {
      Some(parent) => {
        let mut parnet_stack = self.expect_get_scope(parent).stack.clone();
        parnet_stack.push(id);
        parnet_stack
      }
      None => vec![id],
    };
    let is_strict = match parent {
      Some(parent) => self.expect_get_scope(parent).is_strict,
      None => false,
    };
    let info = ScopeInfo {
      is_strict,
      stack,
      map: Default::default(),
    };
    let prev = self.map.insert(id, info);
    assert!(prev.is_none());
    id
  }

  pub fn create(&mut self) -> ScopeInfoId {
    self._create(None)
  }

  pub fn create_child(&mut self, parent: &ScopeInfoId) -> ScopeInfoId {
    self._create(Some(parent))
  }

  pub fn expect_get_scope(&self, id: &ScopeInfoId) -> &ScopeInfo {
    self
      .map
      .get(id)
      .unwrap_or_else(|| panic!("{id:#?} should exist"))
  }

  pub fn expect_get_mut_scope(&mut self, id: &ScopeInfoId) -> &mut ScopeInfo {
    self
      .map
      .get_mut(id)
      .unwrap_or_else(|| panic!("{id:#?} should exist"))
  }

  pub fn expect_get_variable(&self, id: &VariableInfoId) -> &VariableInfo {
    self
      .variable_info_db
      .map
      .get(id)
      .unwrap_or_else(|| panic!("{id:#?} should exist"))
  }

  pub fn expect_get_mut_variable(&mut self, id: &VariableInfoId) -> &mut VariableInfo {
    self
      .variable_info_db
      .map
      .get_mut(id)
      .unwrap_or_else(|| panic!("{id:#?} should exist"))
  }

  pub fn expect_get_tag_info(&self, id: &TagInfoId) -> &TagInfo {
    self
      .tag_info_db
      .map
      .get(id)
      .unwrap_or_else(|| panic!("{id:#?} should exist"))
  }

  pub fn expect_get_mut_tag_info(&mut self, id: &TagInfoId) -> &mut TagInfo {
    self
      .tag_info_db
      .map
      .get_mut(id)
      .unwrap_or_else(|| panic!("{id:#?} should exist"))
  }

  pub fn get<S: AsRef<str>>(&mut self, id: &ScopeInfoId, key: S) -> Option<VariableInfoId> {
    let definitions = self.expect_get_scope(id);
    if let Some(&top_value) = definitions.map.get(key.as_ref()) {
      if top_value == VariableInfo::TOMBSTONE || top_value == VariableInfo::UNDEFINED {
        None
      } else {
        Some(top_value)
      }
    } else if definitions.stack.len() > 1 {
      for index in (0..definitions.stack.len() - 1).rev() {
        // SAFETY: boundary had been checked
        let id = unsafe { definitions.stack.get_unchecked(index) };
        if let Some(&value) = self.expect_get_scope(id).map.get(key.as_ref()) {
          if value == VariableInfo::TOMBSTONE || value == VariableInfo::UNDEFINED {
            return None;
          } else {
            return Some(value);
          }
        }
      }
      let definitions = self.expect_get_mut_scope(id);
      definitions
        .map
        .insert(key.as_ref().to_string(), VariableInfo::TOMBSTONE);
      None
    } else {
      None
    }
  }

  pub fn set(&mut self, id: ScopeInfoId, key: String, variable_info_id: VariableInfoId) {
    let scope = self.expect_get_mut_scope(&id);
    scope.map.insert(key, variable_info_id);
  }

  pub fn delete<S: AsRef<str>>(&mut self, id: ScopeInfoId, key: S) {
    let scope = self.expect_get_mut_scope(&id);
    if scope.stack.len() > 1 {
      scope
        .map
        .insert(key.as_ref().to_string(), VariableInfo::TOMBSTONE);
    } else {
      scope.map.remove(key.as_ref());
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TagInfo {
  id: TagInfoId,
  pub tag: &'static str,
  pub data: Option<serde_json::Value>,
  pub next: Option<TagInfoId>,
}

impl TagInfo {
  pub fn create(
    definitions_db: &mut ScopeInfoDB,
    tag: &'static str,
    data: Option<serde_json::Value>,
    next: Option<TagInfoId>,
  ) -> TagInfoId {
    let id = definitions_db.tag_info_db.next();
    let tag_info = TagInfo {
      id,
      tag,
      data,
      next,
    };
    let prev = definitions_db.tag_info_db.map.insert(id, tag_info);
    assert!(prev.is_none());
    id
  }

  pub fn id(&self) -> TagInfoId {
    self.id
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FreeName {
  String(String),
  True,
}

#[derive(Debug, PartialEq, Eq)]
pub struct VariableInfo {
  id: VariableInfoId,
  pub declared_scope: ScopeInfoId,
  pub free_name: Option<FreeName>,
  pub tag_info: Option<TagInfoId>,
}

impl VariableInfo {
  const TOMBSTONE: VariableInfoId = VariableInfoId(0);
  const UNDEFINED: VariableInfoId = VariableInfoId(1);

  pub fn create(
    definitions_db: &mut ScopeInfoDB,
    declared_scope: ScopeInfoId,
    free_name: Option<FreeName>,
    tag_info: Option<TagInfoId>,
  ) -> VariableInfoId {
    let id = definitions_db.variable_info_db.next();
    let variable_info = VariableInfo {
      id,
      declared_scope,
      free_name,
      tag_info,
    };
    let prev = definitions_db
      .variable_info_db
      .map
      .insert(id, variable_info);
    assert!(prev.is_none());
    id
  }

  pub fn id(&self) -> VariableInfoId {
    self.id
  }
}

#[derive(Debug)]
pub struct ScopeInfo {
  stack: Vec<ScopeInfoId>,
  map: FxHashMap<String, VariableInfoId>,
  pub is_strict: bool,
}

impl ScopeInfo {
  pub fn variable_map(&self) -> &FxHashMap<String, VariableInfoId> {
    &self.map
  }

  pub fn variables(&self) -> impl Iterator<Item = (&str, &VariableInfoId)> {
    self
      .map
      .iter()
      .filter(|(_, &info_id)| info_id != VariableInfo::TOMBSTONE)
      .map(|(name, info_id)| (name.as_str(), info_id))
  }
}
