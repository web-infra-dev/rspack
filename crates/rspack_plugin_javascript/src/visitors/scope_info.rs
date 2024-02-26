use rustc_hash::FxHashMap;
use swc_core::atoms::Atom;

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

  fn insert(&mut self, mut variable_info: VariableInfo) -> VariableInfoId {
    let id = self.next();
    variable_info.set_id(id);
    let prev = self.map.insert(id, variable_info);
    assert!(prev.is_none());
    id
  }
}

#[derive(Debug)]
pub struct ScopeInfoDB {
  count: ScopeInfoId,
  map: FxHashMap<ScopeInfoId, ScopeInfo>,
  variable_info_db: VariableInfoDB,
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

  pub fn get(&mut self, id: &ScopeInfoId, key: &Atom) -> Option<VariableInfoId> {
    let definitions = self.expect_get_scope(id);
    if let Some(&top_value) = definitions.map.get(key) {
      if top_value == VariableInfo::TOMBSTONE || top_value == VariableInfo::UNDEFINED {
        None
      } else {
        Some(top_value)
      }
    } else if definitions.stack.len() > 1 {
      for index in (0..definitions.stack.len() - 1).rev() {
        // SAFETY: boundary had been checked
        let id = unsafe { definitions.stack.get_unchecked(index) };
        if let Some(&value) = self.expect_get_scope(id).map.get(key) {
          if value == VariableInfo::TOMBSTONE || value == VariableInfo::UNDEFINED {
            return None;
          } else {
            return Some(value);
          }
        }
      }
      let definitions = self.expect_get_mut_scope(id);
      definitions.map.insert(key.clone(), VariableInfo::TOMBSTONE);
      None
    } else {
      None
    }
  }

  pub fn set(&mut self, id: ScopeInfoId, key: Atom, info: VariableInfo) {
    let variable_info_id = self.variable_info_db.insert(info);
    let scope = self.expect_get_mut_scope(&id);
    scope.map.insert(key, variable_info_id);
  }

  pub fn delete(&mut self, id: ScopeInfoId, key: Atom) {
    let scope = self.expect_get_mut_scope(&id);
    if scope.stack.len() > 1 {
      scope.map.insert(key, VariableInfo::TOMBSTONE);
    } else {
      scope.map.remove(&key);
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TagInfo {
  pub tag: &'static str,
  pub data: Option<serde_json::Value>,
  pub next: Option<Box<TagInfo>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FreeName {
  String(Atom),
  True,
}

#[derive(Debug, PartialEq, Eq)]
pub struct VariableInfo {
  id: Option<VariableInfoId>,
  pub declared_scope: ScopeInfoId,
  pub free_name: Option<FreeName>,
  pub tag_info: Option<TagInfo>,
}

impl VariableInfo {
  const TOMBSTONE: VariableInfoId = VariableInfoId(0);
  const UNDEFINED: VariableInfoId = VariableInfoId(1);

  pub fn new(
    declared_scope: ScopeInfoId,
    free_name: Option<FreeName>,
    tag_info: Option<TagInfo>,
  ) -> Self {
    Self {
      id: None,
      declared_scope,
      free_name,
      tag_info,
    }
  }

  pub fn update_tag_info_data(&mut self, data: Option<serde_json::Value>) {
    let tag_info = self.tag_info.as_mut().expect("make sure `tag_info` exist");
    tag_info.data = data;
  }

  fn set_id(&mut self, id: VariableInfoId) {
    self.id = Some(id);
  }

  pub fn id(&self) -> VariableInfoId {
    self
      .id
      .expect("should already store VariableInfo to VariableInfoDB")
  }
}

#[derive(Debug)]
pub struct ScopeInfo {
  stack: Vec<ScopeInfoId>,
  map: FxHashMap<Atom, VariableInfoId>,
  pub is_strict: bool,
}
