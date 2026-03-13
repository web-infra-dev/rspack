use std::{
  array,
  sync::{
    RwLock,
    atomic::{AtomicU64, Ordering},
  },
};

use rspack_cacheable::{
  Result, cacheable,
  with::{AsConverter, AsMap, AsPreset},
};
use rustc_hash::FxHashMap;
use ustr::Ustr;

use crate::SourceType;

const SOURCE_SIZE_CACHE_SLOTS: usize = 13;
const SOURCE_SIZE_UNSET: u64 = u64::MAX;

#[derive(Debug)]
pub struct SourceSizeCache {
  // Fixed slots for builtin SourceType variants to avoid hashing/locking overhead.
  builtins: [AtomicU64; SOURCE_SIZE_CACHE_SLOTS],
  // Rare fallback for SourceType::Custom(...)
  custom: RwLock<FxHashMap<Ustr, f64>>,
}

impl Default for SourceSizeCache {
  fn default() -> Self {
    Self {
      builtins: array::from_fn(|_| AtomicU64::new(SOURCE_SIZE_UNSET)),
      custom: RwLock::default(),
    }
  }
}

impl SourceSizeCache {
  /// Maps builtin SourceType to a fixed slot. `Custom` is handled by the fallback map.
  fn builtin_index(source_type: &SourceType) -> Option<usize> {
    match source_type {
      SourceType::JavaScript => Some(0),
      SourceType::Css => Some(1),
      SourceType::CssUrl => Some(2),
      SourceType::Wasm => Some(3),
      SourceType::Asset => Some(4),
      SourceType::Expose => Some(5),
      SourceType::Remote => Some(6),
      SourceType::ShareInit => Some(7),
      SourceType::ConsumeShared => Some(8),
      SourceType::ShareContainerShared => Some(9),
      SourceType::Unknown => Some(10),
      SourceType::CssImport => Some(11),
      SourceType::Runtime => Some(12),
      SourceType::Custom(_) => None,
    }
  }

  pub fn get(&self, source_type: &SourceType) -> Option<f64> {
    if let Some(index) = Self::builtin_index(source_type) {
      let bits = self.builtins[index].load(Ordering::Relaxed);
      (bits != SOURCE_SIZE_UNSET).then_some(f64::from_bits(bits))
    } else if let SourceType::Custom(custom) = source_type {
      self
        .custom
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(custom)
        .copied()
    } else {
      None
    }
  }

  pub fn get_or_insert(&self, source_type: &SourceType, size: f64) -> f64 {
    if let Some(index) = Self::builtin_index(source_type) {
      let bits = size.to_bits();
      match self.builtins[index].compare_exchange(
        SOURCE_SIZE_UNSET,
        bits,
        Ordering::Relaxed,
        Ordering::Relaxed,
      ) {
        Ok(_) => size,
        Err(existing) => f64::from_bits(existing),
      }
    } else if let SourceType::Custom(custom) = source_type {
      let mut custom_sizes = self
        .custom
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
      *custom_sizes.entry(*custom).or_insert(size)
    } else {
      size
    }
  }
}

#[cacheable(crate=rspack_cacheable)]
pub struct SourceSizeCacheSerde {
  builtins: [Option<f64>; SOURCE_SIZE_CACHE_SLOTS],
  #[cacheable(with=AsMap<AsPreset>)]
  custom: FxHashMap<Ustr, f64>,
}

impl AsConverter<SourceSizeCache> for SourceSizeCacheSerde {
  fn serialize(data: &SourceSizeCache, _guard: &rspack_cacheable::ContextGuard) -> Result<Self> {
    let builtins = array::from_fn(|index| {
      let bits = data.builtins[index].load(Ordering::Relaxed);
      (bits != SOURCE_SIZE_UNSET).then_some(f64::from_bits(bits))
    });
    let custom = data
      .custom
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .clone();
    Ok(Self { builtins, custom })
  }

  fn deserialize(self, _guard: &rspack_cacheable::ContextGuard) -> Result<SourceSizeCache> {
    let builtins = array::from_fn(|index| match self.builtins[index] {
      Some(size) => AtomicU64::new(size.to_bits()),
      None => AtomicU64::new(SOURCE_SIZE_UNSET),
    });
    Ok(SourceSizeCache {
      builtins,
      custom: RwLock::new(self.custom),
    })
  }
}
