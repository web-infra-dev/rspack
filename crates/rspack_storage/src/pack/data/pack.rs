use std::sync::Arc;

use rspack_paths::Utf8PathBuf;

use crate::{StorageItemKey, StorageItemValue};

pub type PackKeys = Vec<Arc<StorageItemKey>>;
pub type PackContents = Vec<Arc<StorageItemValue>>;

#[derive(Debug, Default)]
pub enum PackKeysState {
  #[default]
  Pending,
  Value(PackKeys),
}

impl PackKeysState {
  pub fn loaded(&self) -> bool {
    matches!(self, Self::Value(_))
  }
  pub fn set_value(&mut self, value: PackKeys) {
    *self = PackKeysState::Value(value);
  }
  pub fn get_value(&self) -> Option<&PackKeys> {
    match self {
      PackKeysState::Value(v) => Some(v),
      PackKeysState::Pending => None,
    }
  }
  pub fn expect_value(&self) -> &PackKeys {
    match self {
      PackKeysState::Value(v) => v,
      PackKeysState::Pending => panic!("pack key is not ready"),
    }
  }
  pub fn take_value(&mut self) -> Option<PackKeys> {
    match self {
      PackKeysState::Value(v) => Some(std::mem::take(&mut *v)),
      _ => None,
    }
  }
}

#[derive(Debug, Default)]
pub enum PackContentsState {
  #[default]
  Pending,
  Value(PackContents),
}

impl PackContentsState {
  pub fn loaded(&self) -> bool {
    matches!(self, Self::Value(_))
  }
  pub fn set_value(&mut self, value: PackContents) {
    *self = PackContentsState::Value(value);
  }
  pub fn get_value(&self) -> Option<&PackContents> {
    match self {
      PackContentsState::Value(v) => Some(v),
      PackContentsState::Pending => None,
    }
  }
  pub fn expect_value(&self) -> &PackContents {
    match self {
      PackContentsState::Value(v) => v,
      PackContentsState::Pending => panic!("pack content is not ready"),
    }
  }
  pub fn take_value(&mut self) -> Option<PackContents> {
    match self {
      PackContentsState::Value(v) => Some(std::mem::take(&mut *v)),
      _ => None,
    }
  }
}

#[derive(Debug)]
pub struct Pack {
  pub path: Utf8PathBuf,
  pub keys: PackKeysState,
  pub contents: PackContentsState,
}

impl Pack {
  pub fn new(path: Utf8PathBuf) -> Self {
    Self {
      path,
      keys: Default::default(),
      contents: Default::default(),
    }
  }

  pub fn loaded(&self) -> bool {
    matches!(self.keys, PackKeysState::Value(_))
      && matches!(self.contents, PackContentsState::Value(_))
  }

  pub fn size(&self) -> usize {
    self
      .keys
      .expect_value()
      .iter()
      .chain(self.contents.expect_value().iter())
      .fold(0_usize, |acc, item| acc + item.len())
  }
}
