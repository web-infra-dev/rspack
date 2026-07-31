use lz4_flex::block;

use crate::{Error, Result};

const FRAME_MAGIC: &[u8; 8] = b"RSPKLZ01";
const FRAME_HEADER_SIZE: usize = FRAME_MAGIC.len() + size_of::<u64>();
const MIN_VALUE_SIZE: usize = 256;
const MAX_VALUE_SIZE: usize = 128 * 1024 * 1024;

/// Per-worker compression state prevents allocating a fresh LZ4 table per value.
#[derive(Default)]
pub(super) struct Encoder {
  scratch: Vec<u8>,
  table: block::CompressTable,
}

impl Encoder {
  /// Compress a new value only when its complete frame saves at least 10%.
  ///
  /// Existing packed values remain in their physical representation, so
  /// incremental writes never decompress and recompress untouched entries.
  pub(super) fn encode(&mut self, value: Vec<u8>) -> Vec<u8> {
    if !(MIN_VALUE_SIZE..=MAX_VALUE_SIZE).contains(&value.len()) {
      return value;
    }

    self.scratch.resize(
      FRAME_HEADER_SIZE + block::get_maximum_output_size(value.len()),
      0,
    );
    self.scratch[..FRAME_MAGIC.len()].copy_from_slice(FRAME_MAGIC);
    self.scratch[FRAME_MAGIC.len()..FRAME_HEADER_SIZE]
      .copy_from_slice(&(value.len() as u64).to_le_bytes());
    let Ok(compressed_len) = block::compress_into_with_table(
      &value,
      &mut self.scratch[FRAME_HEADER_SIZE..],
      &mut self.table,
    ) else {
      return value;
    };

    let framed_len = FRAME_HEADER_SIZE + compressed_len;
    if framed_len > value.len() - value.len() / 10 {
      return value;
    }

    self.scratch[..framed_len].to_vec()
  }
}

/// Decode a persisted value, preserving compatibility with older uncompressed packs.
pub(super) fn decode(value: Vec<u8>) -> Result<Vec<u8>> {
  if !value.starts_with(FRAME_MAGIC) {
    return Ok(value);
  }

  let expected_bytes = value
    .get(FRAME_MAGIC.len()..FRAME_HEADER_SIZE)
    .ok_or_else(|| Error::CorruptedData("truncated compressed persistent value header".into()))?;
  let expected = u64::from_le_bytes(
    expected_bytes
      .try_into()
      .map_err(|_| Error::CorruptedData("invalid compressed persistent value length".into()))?,
  );
  let expected = usize::try_from(expected).map_err(|_| {
    Error::CorruptedData("compressed persistent value exceeds platform size".into())
  })?;
  if !(MIN_VALUE_SIZE..=MAX_VALUE_SIZE).contains(&expected) {
    return Err(Error::CorruptedData(format!(
      "compressed persistent value length {expected} exceeds the allowed range"
    )));
  }

  let compressed = value
    .get(FRAME_HEADER_SIZE..)
    .ok_or_else(|| Error::CorruptedData("truncated compressed persistent value".into()))?;
  let mut decoded = vec![0; expected];
  let written = block::decompress_into(compressed, &mut decoded).map_err(|error| {
    Error::CorruptedData(format!("invalid compressed persistent value: {error}"))
  })?;
  if written != expected {
    return Err(Error::CorruptedData(format!(
      "compressed persistent value length mismatch: expected {expected}, got {written}"
    )));
  }

  Ok(decoded)
}

#[cfg(test)]
mod tests {
  use super::{Encoder, FRAME_HEADER_SIZE, FRAME_MAGIC, MAX_VALUE_SIZE, MIN_VALUE_SIZE, decode};

  #[test]
  fn compressible_values_round_trip() {
    let original = b"deterministic synthetic graph content ".repeat(2048);
    let mut encoder = Encoder::default();
    let encoded = encoder.encode(original.clone());

    assert!(encoded.starts_with(FRAME_MAGIC));
    assert!(encoded.len() < original.len() / 10);
    assert_eq!(decode(encoded).expect("value should decode"), original);
  }

  #[test]
  fn worker_scratch_is_reused_across_values() {
    let mut encoder = Encoder::default();
    let first = b"first repeated cache value ".repeat(1024);
    let first_encoded = encoder.encode(first.clone());
    let capacity = encoder.scratch.capacity();
    let second = b"second repeated cache value ".repeat(512);
    let second_encoded = encoder.encode(second.clone());

    assert_eq!(encoder.scratch.capacity(), capacity);
    assert_eq!(decode(first_encoded).expect("first should decode"), first);
    assert_eq!(
      decode(second_encoded).expect("second should decode"),
      second
    );
  }

  #[test]
  fn small_and_legacy_values_remain_unmodified() {
    let original = b"existing uncompressed persistent value".to_vec();
    let mut encoder = Encoder::default();

    assert_eq!(encoder.encode(original.clone()), original);
    assert_eq!(
      decode(original.clone()).expect("legacy value should decode"),
      original
    );
  }

  #[test]
  fn incompressible_values_remain_unmodified() {
    let mut state = 0x6b36_2f29_493a_e149_u64;
    let original = (0..32 * 1024)
      .map(|_| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state as u8
      })
      .collect::<Vec<_>>();

    assert_eq!(Encoder::default().encode(original.clone()), original);
  }

  #[test]
  fn truncated_headers_and_payloads_are_rejected() {
    assert!(decode(FRAME_MAGIC.to_vec()).is_err());

    let mut encoded = Encoder::default().encode(b"repeated graph source ".repeat(1024));
    encoded.truncate(FRAME_HEADER_SIZE);
    assert!(decode(encoded).is_err());
  }

  #[test]
  fn declared_sizes_outside_the_allowed_range_are_rejected() {
    for invalid_len in [
      0,
      MIN_VALUE_SIZE as u64 - 1,
      MAX_VALUE_SIZE as u64 + 1,
      u64::MAX,
    ] {
      let mut frame = FRAME_MAGIC.to_vec();
      frame.extend_from_slice(&invalid_len.to_le_bytes());
      assert!(decode(frame).is_err());
    }
  }

  #[test]
  fn malformed_and_trailing_payloads_are_rejected() {
    let encoded = Encoder::default().encode(b"repeated graph source ".repeat(1024));

    let mut truncated = encoded.clone();
    truncated.pop();
    assert!(decode(truncated).is_err());

    for trailing in [0, 0xff] {
      let mut value = encoded.clone();
      value.push(trailing);
      assert!(decode(value).is_err());
    }

    let mut malformed = FRAME_MAGIC.to_vec();
    malformed.extend_from_slice(&(MIN_VALUE_SIZE as u64).to_le_bytes());
    malformed.extend_from_slice(&[0, 0, 0]);
    assert!(decode(malformed).is_err());
  }
}
