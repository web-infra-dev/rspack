use crate::{external_module::ExternalModule, module::Module};

#[derive(Debug)]
pub struct OutputChunk {
  pub code: String,
  pub file_name: String,
}

#[derive(Debug)]
pub struct RenderedChunk {
  pub code: String,
  pub file_name: String,
}

#[derive(Debug)]
pub enum RolldownOutput {
  Chunk(OutputChunk),
  Asset,
}

impl RolldownOutput {
  #[inline]
  pub fn get_file_name(&self) -> &str {
    match self {
      RolldownOutput::Chunk(c) => c.file_name.as_ref(),
      RolldownOutput::Asset => panic!(""),
    }
  }

  #[inline]
  pub fn get_content(&self) -> &str {
    match self {
      RolldownOutput::Chunk(c) => c.code.as_ref(),
      RolldownOutput::Asset => panic!(""),
    }
  }
}

#[derive(Debug, Hash, Clone)]
pub enum ModOrExt {
  Mod(Box<Module>),
  Ext(ExternalModule),
}


use std::hash::Hash;

use smol_str::SmolStr;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ResolvedId {
    pub id: SmolStr,
    pub external: bool,
}

impl ResolvedId {
    pub fn new<T: Into<SmolStr>>(id: T, external: bool) -> Self {
        Self {
            id: id.into(),
            external,
            // module_side_effects: false,
        }
    }
}

pub type ResolveIdResult = Option<ResolvedId>;