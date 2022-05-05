use crate::external_module::ExternalModule;

#[derive(Debug)]
pub struct OutputChunk {
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

#[derive(Debug)]
pub enum ModOrExt {
  Mod(Box<JsModule>),
  Ext(ExternalModule),
}

use std::{collections::HashSet, hash::Hash};

use rspack_core::JsModule;
use smol_str::SmolStr;
use swc_atoms::JsWord;
use swc_common::Mark;

pub use rspack_core::ResolvedId;
pub type ResolveIdResult = Option<ResolvedId>;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Specifier {
  /// The original defined name
  pub original: JsWord,
  /// The name importer used
  pub used: JsWord,
  pub mark: Mark,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RelationInfo {
  pub source: JsWord,
  // Empty HashSet represents `import './side-effect'` or `import {} from './foo'`
  pub names: HashSet<Specifier>,
}

// impl From<RelationInfo> for Rel {
//     fn from(info: RelationInfo) -> Self {
//         Self::Import(info)
//     }
// }

impl RelationInfo {
  pub fn new(source: JsWord) -> Self {
    Self {
      source,
      names: Default::default(),
      // namespace: Default::default(),
    }
  }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ExportDesc {
  // export foo; foo is identifier;
  pub identifier: Option<JsWord>,
  pub local_name: JsWord,
  pub mark: Mark,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ReExportDesc {
  // name in importee
  pub original: JsWord,
  // locally defined name
  pub local_name: JsWord,
  pub source: JsWord,
  pub mark: Mark,
}

pub use rspack_core::DynImportDesc;
