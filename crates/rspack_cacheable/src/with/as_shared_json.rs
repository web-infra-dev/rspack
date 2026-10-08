use std::{
  cell::RefCell,
  collections::hash_map::Entry,
  sync::{Arc, LazyLock, Mutex, Weak},
};

use rkyv::{
  Place,
  rancor::Fallible,
  ser::Writer,
  string::ArchivedString,
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};
use rustc_hash::FxHashMap;
use serde_json::Value;
use xxhash_rust::const_xxh64::xxh64;

use super::{AsInner, AsPreset};
use crate::{Error, Result};

/// Shares decoded JSON without changing its string archive or serializer.
///
/// Process-wide sharing is safe because values are immutable and every hash hit
/// is verified against the full serialized content or the parsed value. Only weak handles and fixed-size
/// keys are retained. Dead entries are pruned after an amortized number of misses
/// proportional to the last live entry count (at least 64), and the table is shrunk.
/// Verification retains at most 64 KiB of reusable scratch space per decoding thread.
pub struct AsSharedJson;

type Key = (usize, u64);

#[derive(Default)]
struct JsonInterner {
  values: FxHashMap<Key, Weak<Value>>,
  misses: usize,
  prune_interval: usize,
}

impl JsonInterner {
  fn get_or_insert(&mut self, key: Key, value: &Arc<Value>) -> Option<Arc<Value>> {
    self.misses += 1;
    if self.misses >= self.prune_interval.max(64) {
      self.values.retain(|_, value| value.strong_count() > 0);
      self.values.shrink_to_fit();
      self.misses = 0;
      self.prune_interval = self.values.len();
    }
    match self.values.entry(key) {
      Entry::Occupied(mut entry) => {
        if let Some(existing) = entry.get().upgrade() {
          return Some(existing);
        }
        entry.insert(Arc::downgrade(value));
      }
      Entry::Vacant(entry) => {
        entry.insert(Arc::downgrade(value));
      }
    }
    None
  }
}

static JSON_INTERNER: LazyLock<Mutex<JsonInterner>> = LazyLock::new(Default::default);

thread_local! {
  static JSON_BUFFER: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

fn matches_json(value: &Value, text: &str) -> Result<bool> {
  JSON_BUFFER.with(|buffer| {
    let mut buffer = buffer.borrow_mut();
    buffer.clear();
    // to_string uses the same serializer with a Vec writer. Neither path changes
    // object iteration order or number formatting based on the writer.
    let result = simd_json::to_writer(&mut *buffer, value)
      .map(|()| buffer.as_slice() == text.as_bytes())
      .map_err(|_| Error::MessageError("serialize serde_json value failed"));
    buffer.clear();
    if buffer.capacity() > 64 * 1024 {
      *buffer = Vec::new();
    }
    result
  })
}

impl ArchiveWith<Arc<Value>> for AsSharedJson {
  type Archived = <AsInner<AsPreset> as ArchiveWith<Arc<Value>>>::Archived;
  type Resolver = <AsInner<AsPreset> as ArchiveWith<Arc<Value>>>::Resolver;

  fn resolve_with(field: &Arc<Value>, resolver: Self::Resolver, out: Place<Self::Archived>) {
    AsInner::<AsPreset>::resolve_with(field, resolver, out);
  }
}

impl<S> SerializeWith<Arc<Value>, S> for AsSharedJson
where
  S: Fallible<Error = Error> + Writer,
{
  fn serialize_with(field: &Arc<Value>, serializer: &mut S) -> Result<Self::Resolver> {
    AsInner::<AsPreset>::serialize_with(field, serializer)
  }
}

impl<D> DeserializeWith<ArchivedString, Arc<Value>, D> for AsSharedJson
where
  D: Fallible<Error = Error>,
{
  fn deserialize_with(field: &ArchivedString, deserializer: &mut D) -> Result<Arc<Value>> {
    let text = field.as_str();
    let key = (text.len(), xxh64(text.as_bytes(), 0));
    let existing = JSON_INTERNER
      .lock()
      .expect("JSON interner lock poisoned")
      .values
      .get(&key)
      .and_then(Weak::upgrade);
    if let Some(existing) = existing
      && matches_json(&existing, text)?
    {
      return Ok(existing);
    }

    // Parse and verify outside the lock. A second lookup lets concurrent misses
    // reuse the value that another decoder inserted while this one was parsing.
    let value = AsInner::<AsPreset>::deserialize_with(field, deserializer)?;
    let existing = JSON_INTERNER
      .lock()
      .expect("JSON interner lock poisoned")
      .get_or_insert(key, &value);
    if let Some(existing) = existing {
      // JSON parsing can change floating-point formatting. Comparing the parsed
      // values also handles those byte-different round trips without trusting a hash.
      if existing.as_ref() == value.as_ref() {
        return Ok(existing);
      }
      // A colliding candidate must never be returned for different content.
      JSON_INTERNER
        .lock()
        .expect("JSON interner lock poisoned")
        .values
        .insert(key, Arc::downgrade(&value));
    }
    Ok(value)
  }
}
