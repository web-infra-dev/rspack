use std::{
  fmt,
  hash::{Hash, Hasher},
  sync::Arc,
};

use base64_simd::{AsOut, STANDARD, URL_SAFE_NO_PAD};
use md4::Digest;
use rspack_cacheable::{cacheable, with::AsPreset};
pub use rspack_macros::RspackHashable;
use rspack_util::MergeFrom;
use smol_str::SmolStr;
use xxhash_rust::xxh64::Xxh64;

mod hashable;

#[cacheable]
#[derive(Debug, Clone, Copy)]
pub enum HashFunction {
  Xxhash64,
  MD4,
  SHA256,
}

impl From<&str> for HashFunction {
  fn from(value: &str) -> Self {
    match value {
      "xxhash64" => HashFunction::Xxhash64,
      "md4" => HashFunction::MD4,
      "sha256" => HashFunction::SHA256,
      _ => panic!("Unsupported hash function: '{value}'. Expected one of: xxhash64, md4, sha256"),
    }
  }
}

impl MergeFrom for HashFunction {
  fn merge_from(self, other: &Self) -> Self {
    *other
  }
}

#[cacheable]
#[derive(Debug, Clone, Copy)]
pub enum HashDigest {
  Hex,
  Base64,
  Base64Url,
  Base62,
  Base58,
  Base52,
  Base49,
  Base36,
  Base32,
  Base26,
}

impl From<&str> for HashDigest {
  fn from(value: &str) -> Self {
    match value {
      "hex" => HashDigest::Hex,
      "base64" => HashDigest::Base64,
      "base64url" => HashDigest::Base64Url,
      "base62" => HashDigest::Base62,
      "base58" => HashDigest::Base58,
      "base52" => HashDigest::Base52,
      "base49" => HashDigest::Base49,
      "base36" => HashDigest::Base36,
      "base32" => HashDigest::Base32,
      "base26" => HashDigest::Base26,
      _ => panic!(
        "Unsupported hash digest: '{value}'. Expected one of: hex, base64, base64url, base62, base58, base52, base49, base36, base32, base26"
      ),
    }
  }
}

impl MergeFrom for HashDigest {
  fn merge_from(self, other: &Self) -> Self {
    *other
  }
}

#[cacheable]
#[derive(Debug, Clone, Hash, Default)]
pub enum HashSalt {
  #[default]
  None,
  Salt(#[cacheable(with=AsPreset)] SmolStr),
}

impl<T> From<Option<T>> for HashSalt
where
  T: Into<SmolStr>,
{
  fn from(value: Option<T>) -> Self {
    match value {
      Some(salt) => Self::Salt(salt.into()),
      None => Self::None,
    }
  }
}

impl MergeFrom for HashSalt {
  fn merge_from(self, other: &Self) -> Self {
    if matches!(other, HashSalt::None) {
      self
    } else {
      other.clone()
    }
  }
}

/// Hasher used for webpack-compatible content hashes.
///
/// `RspackHash` is the stateful writer behind output hashes such as full hash,
/// chunk hash, content hash and module/runtime hashes that affect generated
/// assets or persistent cache correctness. Inputs should be written through
/// [`RspackHashable`] so the serialized form can follow webpack content-hash
/// semantics instead of Rust collection-key hashing semantics.
#[derive(Clone)]
pub enum RspackHash {
  Xxhash64(Box<Xxh64>),
  MD4(Box<md4::Md4>),
  SHA256(Box<sha2::Sha256>),
}

/// Content-hash input contract for values that participate in `RspackHash`.
///
/// This trait is intentionally separate from [`std::hash::Hash`]. `Hash` is for
/// hash-map/set keys and is free to optimize around Rust data-structure needs,
/// including implementation details that may change with the standard library
/// or with local keying requirements. `RspackHashable` is for stable,
/// webpack-aligned content hashing: implement it only for data that should
/// affect emitted asset hashes, runtime/module hashes, or persistent cache
/// content keys.
///
/// Keeping the two traits separate lets Rspack tune key hashing independently
/// without changing content hash behavior, and lets content hashing encode the
/// same logical inputs webpack uses rather than the shape of Rust data
/// structures.
pub trait RspackHashable {
  fn hash(&self, state: &mut RspackHash);
}

#[inline]
pub fn hash_by_json<T: serde::Serialize>(value: &T, state: &mut RspackHash) {
  let json = simd_json::to_string(value).expect("should be able to serialize value for hash");
  state.write(json.as_bytes());
}

#[inline]
pub fn write_u64_hex(value: u64, state: &mut RspackHash) {
  if value == 0 {
    state.write(b"0");
    return;
  }

  let bytes = value.to_be_bytes();
  let first = bytes
    .iter()
    .position(|&byte| byte != 0)
    .expect("zero value should have returned");
  let mut output = [0; 16];
  let encoded = hex(&bytes[first..], &mut output).as_bytes();

  if bytes[first] < 0x10 {
    state.write(&encoded[1..]);
  } else {
    state.write(encoded);
  }
}

#[macro_export]
macro_rules! rspack_hash_object {
  ($state:expr, { $($key:expr => $value:expr),* $(,)? }) => {{
    $state.write(b"{");
    let mut is_first_rspack_hash_field = true;
    $(
      if !is_first_rspack_hash_field {
        $state.write(b",");
      }
      is_first_rspack_hash_field = false;
      $crate::RspackHashable::hash($key, $state);
      $state.write(b":");
      $crate::RspackHashable::hash(&$value, $state);
    )*
    let _ = is_first_rspack_hash_field;
    $state.write(b"}");
  }};
}

#[macro_export]
macro_rules! rspack_hash_update {
  ($state:expr, $($value:expr),+ $(,)?) => {
    $(
      $state.update(&$value);
    )+
  };
}

impl fmt::Debug for RspackHash {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Xxhash64(_) => write!(f, "RspackHash(Xxhash64)"),
      Self::MD4(_) => write!(f, "RspackHash(MD4)"),
      Self::SHA256(_) => write!(f, "RspackHash(SHA256"),
    }
  }
}

impl RspackHash {
  pub fn new(function: &HashFunction) -> Self {
    match function {
      HashFunction::Xxhash64 => Self::Xxhash64(Box::new(Xxh64::new(0))),
      HashFunction::MD4 => Self::MD4(Box::new(md4::Md4::new())),
      HashFunction::SHA256 => Self::SHA256(Box::new(sha2::Sha256::new())),
    }
  }

  pub fn with_salt(function: &HashFunction, salt: &HashSalt) -> Self {
    let mut this = Self::new(function);
    if let HashSalt::Salt(salt) = salt {
      this.write(salt.as_bytes());
    }
    this
  }

  pub fn digest(self, digest: &HashDigest) -> RspackHashDigest {
    match self {
      RspackHash::Xxhash64(hasher) => {
        let buf = hasher.finish().to_be_bytes();
        RspackHashDigest::new(&buf, digest)
      }
      RspackHash::MD4(hash) => {
        let buf = hash.finalize();
        RspackHashDigest::new(&buf, digest)
      }
      RspackHash::SHA256(hash) => {
        let buf = hash.finalize();
        RspackHashDigest::new(&buf, digest)
      }
    }
  }

  pub fn update<T: RspackHashable + ?Sized>(&mut self, value: &T) {
    value.hash(self);
  }

  pub fn write(&mut self, bytes: &[u8]) {
    match self {
      RspackHash::Xxhash64(hasher) => hasher.write(bytes),
      RspackHash::MD4(hasher) => hasher.update(bytes),
      RspackHash::SHA256(hasher) => hasher.update(bytes),
    }
  }

  pub fn finish(&self) -> u64 {
    match self {
      RspackHash::Xxhash64(hasher) => hasher.finish(),
      RspackHash::MD4(hasher) => {
        let hash = (**hasher).clone().finalize();
        u64::from_be_bytes(
          hash[..8]
            .try_into()
            .expect("md4 digest length is at least 8"),
        )
      }
      RspackHash::SHA256(hasher) => {
        let hash = (**hasher).clone().finalize();
        u64::from_be_bytes(
          hash[..8]
            .try_into()
            .expect("sha256 digest length is at least 8"),
        )
      }
    }
  }
}

impl Hasher for RspackHash {
  fn finish(&self) -> u64 {
    match self {
      RspackHash::Xxhash64(hasher) => hasher.finish(),
      RspackHash::MD4(hasher) => {
        let hash = (**hasher).clone().finalize();
        u64::from_be_bytes(
          hash[..8]
            .try_into()
            .expect("md4 digest length is at least 8"),
        )
      }
      RspackHash::SHA256(hasher) => {
        let hash = (**hasher).clone().finalize();
        u64::from_be_bytes(
          hash[..8]
            .try_into()
            .expect("sha256 digest length is at least 8"),
        )
      }
    }
  }

  fn write(&mut self, bytes: &[u8]) {
    match self {
      RspackHash::Xxhash64(hasher) => hasher.write(bytes),
      RspackHash::MD4(hasher) => hasher.update(bytes),
      RspackHash::SHA256(hasher) => hasher.update(bytes),
    }
  }
}

#[cacheable]
#[derive(Debug, Clone, Eq)]
pub struct RspackHashDigest {
  #[cacheable(with=AsPreset)]
  encoded: SmolStr,
}

impl From<&str> for RspackHashDigest {
  fn from(value: &str) -> Self {
    Self {
      encoded: value.into(),
    }
  }
}

impl RspackHashDigest {
  /// `inner ` must be empty or come from a hash up to 256 bits
  pub fn new(inner: &[u8], digest: &HashDigest) -> Self {
    let encoded = match digest {
      HashDigest::Hex => {
        let mut buf = [0; 64];
        let s = hex(inner, &mut buf);
        s.into()
      }
      HashDigest::Base64 => {
        let mut buf = [0; MAX_HASH_ENCODED_LEN];
        STANDARD
          .encode_as_str(inner, buf.as_mut_slice().as_out())
          .into()
      }
      HashDigest::Base64Url => {
        let mut buf = [0; MAX_HASH_ENCODED_LEN];
        URL_SAFE_NO_PAD
          .encode_as_str(inner, buf.as_mut_slice().as_out())
          .into()
      }
      HashDigest::Base62 => encode_base_n(inner, BASE62_CHARSET),
      HashDigest::Base58 => encode_base_n(inner, BASE58_CHARSET),
      HashDigest::Base52 => encode_base_n(inner, BASE52_CHARSET),
      HashDigest::Base49 => encode_base_n(inner, BASE49_CHARSET),
      HashDigest::Base36 => encode_base_n(inner, BASE36_CHARSET),
      HashDigest::Base32 => encode_base_n(inner, BASE32_CHARSET),
      HashDigest::Base26 => encode_base_n(inner, BASE26_CHARSET),
    };
    Self { encoded }
  }

  pub fn encoded(&self) -> &str {
    &self.encoded
  }

  pub fn rendered(&self, length: usize) -> &str {
    let len = self.encoded.len().min(length);
    &self.encoded[..len]
  }
}

impl RspackHashable for RspackHashDigest {
  fn hash(&self, state: &mut RspackHash) {
    RspackHashable::hash(self.encoded.as_str(), state);
  }
}

impl Hash for RspackHashDigest {
  fn hash<H: Hasher>(&self, state: &mut H) {
    std::hash::Hash::hash(&self.encoded, state);
  }
}

impl PartialEq for RspackHashDigest {
  fn eq(&self, other: &Self) -> bool {
    self.encoded == other.encoded
  }
}

/// Implement our own hex that is guaranteed to be inlined.
///
/// This will have good performance as it is simple enough to be understood by compiler.
#[inline]
fn hex<'a>(data: &[u8], output: &'a mut [u8]) -> &'a str {
  const HEX_TABLE: &[u8; 16] = b"0123456789abcdef";

  assert!(data.len() * 2 <= output.len());

  let mut i = 0;
  for byte in data {
    output[i] = HEX_TABLE[(byte >> 4) as usize];
    output[i + 1] = HEX_TABLE[(byte & 0x0f) as usize];
    i += 2;
  }

  // # Safety
  //
  // hex is always ascii
  unsafe { std::str::from_utf8_unchecked(&output[..i]) }
}

// Charsets matching webpack's hash-digest.js ENCODE_TABLE.
// See: https://github.com/webpack/webpack/blob/main/lib/util/hash/hash-digest.js
const BASE26_CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const BASE32_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
const BASE36_CHARSET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
const BASE49_CHARSET: &[u8] = b"abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ";
const BASE52_CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const BASE58_CHARSET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
const BASE62_CHARSET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const MAX_HASH_BYTES_LEN: usize = 32;
const MAX_HASH_ENCODED_LEN: usize = 64;
const SMOL_STR_INLINE_CAP: usize = 23;

/// Encodes raw hash bytes into a base-N string using the given charset.
/// Matches webpack's `encode` in `hash-digest.js`: interprets the buffer as a
/// big-endian integer and converts to the target base by repeated division.
fn encode_base_n(data: &[u8], charset: &[u8]) -> SmolStr {
  if data.is_empty() {
    return SmolStr::default();
  }

  assert!(data.len() <= MAX_HASH_BYTES_LEN);
  let base = charset.len();
  let output_len = encoded_base_n_len(data, base);
  debug_assert!(output_len <= MAX_HASH_ENCODED_LEN);

  if output_len <= SMOL_STR_INLINE_CAP {
    let mut output = vec![0; output_len];
    encode_base_n_into(data, charset, base, output_len, |index, byte| {
      output[index] = byte;
    });
    // SAFETY: all charset bytes are ASCII
    return SmolStr::new_inline(unsafe { std::str::from_utf8_unchecked(&output) });
  }

  let mut output = Arc::<[u8]>::new_uninit_slice(output_len);
  let output_slice = Arc::get_mut(&mut output).expect("new Arc must be unique");
  encode_base_n_into(data, charset, base, output_len, |index, byte| {
    output_slice[index].write(byte);
  });
  // SAFETY: `encode_base_n_into` writes every output slot with ASCII bytes.
  let output = unsafe { output.assume_init() };
  // SAFETY: all charset bytes are ASCII.
  SmolStr::from(unsafe { arc_ascii_bytes_to_str(output) })
}

fn encode_base_n_into(
  data: &[u8],
  charset: &[u8],
  base: usize,
  output_len: usize,
  mut write: impl FnMut(usize, u8),
) {
  let mut output_index = output_len;
  let mut bytes = Vec::with_capacity(data.len());

  let mut remainder = 0usize;
  for &b in data {
    let value = remainder * 256 + b as usize;
    let digit = value / base;
    remainder = value % base;
    if !bytes.is_empty() || digit > 0 {
      bytes.push(digit as u8);
    }
  }
  output_index -= 1;
  write(output_index, charset[remainder]);

  let mut bytes_len = bytes.len();
  while bytes_len > 0 {
    let mut remainder = 0usize;
    let mut next_len = 0;
    for i in 0..bytes_len {
      let b = bytes[i];
      let value = remainder * 256 + b as usize;
      let digit = value / base;
      remainder = value % base;
      if next_len > 0 || digit > 0 {
        bytes[next_len] = digit as u8;
        next_len += 1;
      }
    }
    output_index -= 1;
    write(output_index, charset[remainder]);
    bytes_len = next_len;
  }

  debug_assert_eq!(output_index, 0);
}

fn encoded_base_n_len(data: &[u8], base: usize) -> usize {
  let mut len = 1;
  let mut bytes = Vec::with_capacity(data.len());
  let mut remainder = 0usize;

  for &b in data {
    let value = remainder * 256 + b as usize;
    let digit = value / base;
    remainder = value % base;
    if !bytes.is_empty() || digit > 0 {
      bytes.push(digit as u8);
    }
  }

  let mut bytes_len = bytes.len();
  while bytes_len > 0 {
    let mut remainder = 0usize;
    let mut next_len = 0;
    for i in 0..bytes_len {
      let b = bytes[i];
      let value = remainder * 256 + b as usize;
      let digit = value / base;
      remainder = value % base;
      if next_len > 0 || digit > 0 {
        bytes[next_len] = digit as u8;
        next_len += 1;
      }
    }
    len += 1;
    bytes_len = next_len;
  }

  len
}

unsafe fn arc_ascii_bytes_to_str(output: Arc<[u8]>) -> Arc<str> {
  unsafe { Arc::from_raw(Arc::into_raw(output) as *const str) }
}
