use std::{
  collections::hash_map::Entry,
  io,
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
use xxhash_rust::xxh64::xxh64;

use super::{AsInner, AsPreset};
use crate::{Error, Result};

/// Shares decoded JSON without changing its string archive or serializer.
///
/// Process-wide sharing is safe because values are immutable and every hash hit
/// is verified against the full serialized content, including object key order.
/// Retained metadata consists of fixed-size keys and weak handles. A dead weak
/// handle keeps its small Arc allocation (about 96 bytes) until the next amortized
/// prune. The table is bounded at about twice the live count at the last prune,
/// with a minimum slack of 64 entries. Pruning and shrinking run only on misses,
/// so the last table remains if decoding stops. No verification buffer is retained.
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

struct ComparingWriter<'a> {
  expected: &'a [u8],
  position: usize,
}

impl io::Write for ComparingWriter<'_> {
  fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
    if !self.expected[self.position..].starts_with(bytes) {
      return Err(io::ErrorKind::InvalidData.into());
    }
    self.position += bytes.len();
    Ok(bytes.len())
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}

fn matches_json(value: &Value, text: &str) -> bool {
  let mut writer = ComparingWriter {
    expected: text.as_bytes(),
    position: 0,
  };
  // to_string uses the same serializer with a Vec writer. Neither path changes
  // object iteration order or number formatting based on the writer.
  simd_json::to_writer(&mut writer, value).is_ok() && writer.position == writer.expected.len()
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
      && matches_json(&existing, text)
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
      // Parsing can change number formatting. Compare serializations of both
      // parsed values so those round trips share without ignoring object key order.
      if simd_json::to_string(value.as_ref()).is_ok_and(|text| matches_json(&existing, &text)) {
        return Ok(existing);
      }
      // A racing replacement only loses sharing; it never changes decoded content.
      JSON_INTERNER
        .lock()
        .expect("JSON interner lock poisoned")
        .values
        .insert(key, Arc::downgrade(&value));
    }
    Ok(value)
  }
}
