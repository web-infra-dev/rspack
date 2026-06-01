use rustc_hash::FxHashMap;
use swc_core::{atoms::Atom, ecma::ast::Id};

use crate::visitors::{JavascriptParser, TagInfo, TagInfoDB, TagInfoData, TagInfoId, to_custom_id};

#[derive(Debug, Default)]
pub struct DefinitionsDB {
  vars: FxHashMap<Id, VarInfo>,
  tag_info_db: TagInfoDB,
}

impl DefinitionsDB {
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
}

#[derive(Debug, Default)]
pub struct VarInfo {
  tags: Vec<TagInfoId>,
  origin: Option<IdOrName>,
}

impl VarInfo {
  pub fn tags(&self) -> &[TagInfoId] {
    &self.tags
  }

  pub fn origin(&self) -> Option<&IdOrName> {
    self.origin.as_ref()
  }
}

impl JavascriptParser<'_> {
  pub fn get_var(&mut self, id: &Id) -> &VarInfo {
    let is_free = self.is_free_var(id);
    let var_info = self
      .definitions_db2
      .vars
      .entry(id.clone())
      .or_insert_with(Default::default);
    if is_free {
      var_info.origin = Some(id.clone().into());
    }
    var_info
  }

  pub fn get_var_mut(&mut self, id: &Id) -> &mut VarInfo {
    self
      .definitions_db2
      .vars
      .entry(id.clone())
      .or_insert_with(Default::default)
  }

  pub fn get_var_origin(&mut self, id: &Id) -> Option<&IdOrName> {
    self.get_var(id).origin()
  }

  pub fn get_var_name<'a>(&mut self, id: &'a Id) -> &'a Atom {
    &id.0
  }

  pub fn is_free_var(&self, id: &Id) -> bool {
    id.1.outer() == self.unresolved_mark
  }

  pub fn is_tagged_var(&self, id: &Id) -> bool {
    let Some(var_info) = self.definitions_db2.vars.get(id) else {
      return false;
    };
    !var_info.tags.is_empty()
  }

  pub fn is_defined_var(&mut self, id: &Id) -> bool {
    self.get_var_origin(id).is_none()
  }

  pub fn get_tag_data<Data: TagInfoData>(&self, id: &Id, tag: &'static str) -> Option<&Data> {
    let var_info = self.definitions_db2.vars.get(id)?;
    for tag_id in &var_info.tags {
      let tag_info = self.definitions_db2.expect_get_tag_info(*tag_id);
      if tag_info.tag == tag {
        return Some(TagInfoData::downcast_ref(&tag_info.data));
      }
    }
    None
  }

  pub fn tag_var<Data: TagInfoData>(
    &mut self,
    id: &Id,
    tag: &'static str,
    data: Option<Data>,
  ) -> TagInfoId {
    let tag_info_id = self.definitions_db2.tag_info_db.map.insert(TagInfo {
      tag,
      data: data.map(|data| TagInfoData::into_any(data)),
    });
    let var_info = self.get_var_mut(id);
    var_info.tags.push(tag_info_id);
    var_info.origin = Some(id.clone().into());
    tag_info_id
  }

  pub fn tag_var_no_alias<Data: TagInfoData>(
    &mut self,
    id: &Id,
    tag: &'static str,
    data: Option<Data>,
  ) -> TagInfoId {
    let tag_info_id = self.definitions_db2.tag_info_db.map.insert(TagInfo {
      tag,
      data: data.map(|data| TagInfoData::into_any(data)),
    });
    let var_info = self.get_var_mut(id);
    var_info.tags.push(tag_info_id);
    tag_info_id
  }

  pub fn set_var_origin(&mut self, id: &Id, origin: IdOrName) {
    let var_info = self.get_var_mut(id);
    var_info.origin = Some(origin);
  }

  pub fn unset_var_origin(&mut self, id: &Id) {
    let var_info = self.get_var_mut(id);
    var_info.origin = None;
  }
}

#[derive(Debug, Clone)]
pub enum IdOrName {
  Id(Id),
  Name(Atom),
}

impl From<Id> for IdOrName {
  fn from(id: Id) -> Self {
    IdOrName::Id(id)
  }
}

impl From<Atom> for IdOrName {
  fn from(name: Atom) -> Self {
    IdOrName::Name(name)
  }
}

impl IdOrName {
  pub fn name(&self) -> &Atom {
    match self {
      IdOrName::Id(id) => &id.0,
      IdOrName::Name(name) => name,
    }
  }
}

pub static THIS_ID: std::sync::LazyLock<Id> =
  std::sync::LazyLock::new(|| to_custom_id("this".into()));
