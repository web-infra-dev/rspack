use std::sync::Arc;

use rspack_paths::Utf8PathBuf;

use crate::{ItemKey, ItemValue};

pub(crate) type PackKeys = Vec<Arc<ItemKey>>;
pub(crate) type PackContents = Vec<Arc<ItemValue>>;
pub(crate) type PackGenerations = Vec<usize>;

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
  Released,
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
      PackContentsState::Released => None,
    }
  }
  pub fn expect_value(&self) -> &PackContents {
    match self {
      PackContentsState::Value(v) => v,
      PackContentsState::Pending => panic!("pack content is not ready"),
      PackContentsState::Released => panic!("pack content has been released"),
    }
  }
  pub fn take_value(&mut self) -> Option<PackContents> {
    match self {
      PackContentsState::Value(v) => Some(std::mem::take(&mut *v)),
      _ => None,
    }
  }
  pub fn release(&mut self) {
    *self = PackContentsState::Released;
  }
  pub fn is_released(&self) -> bool {
    matches!(self, Self::Released)
  }
}

#[derive(Debug)]
pub struct Pack {
  pub path: Utf8PathBuf,
  pub keys: PackKeysState,
  pub contents: PackContentsState,
  pub generations: PackGenerations,
}

impl Pack {
  pub fn new(path: Utf8PathBuf) -> Self {
    Self {
      path,
      keys: Default::default(),
      contents: Default::default(),
      generations: Default::default(),
    }
  }

  pub fn loaded(&self) -> bool {
    matches!(self.keys, PackKeysState::Value(_))
      && (matches!(self.contents, PackContentsState::Value(_))
        || matches!(self.contents, PackContentsState::Released))
  }

  pub fn size(&self) -> usize {
    let key_size = self
      .keys
      .expect_value()
      .iter()
      .fold(0_usize, |acc, item| acc + item.len());
    let content_size = self
      .contents
      .expect_value()
      .iter()
      .fold(0_usize, |acc, item| acc + item.len());
    let generation_size = self
      .generations
      .iter()
      .fold(0_usize, |acc, item| acc + item.to_string().len());
    key_size + content_size + generation_size
  }
}
