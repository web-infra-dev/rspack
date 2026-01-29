use std::{
  fmt,
  hash::{Hash, Hasher},
};

use md4::Digest;
use rspack_cacheable::{cacheable, with::AsPreset};
use smol_str::SmolStr;
use xxhash_rust::xxh64::Xxh64;

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
      _ => panic!(
        "Unsupported hash function: '{value}'. Expected one of: xxhash64, md4, sha256"
      ),
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum HashDigest {
  Hex,
}

impl From<&str> for HashDigest {
  fn from(value: &str) -> Self {
    match value {
      "hex" => HashDigest::Hex,
      _ => panic!("Unsupported hash digest: '{value}'. Expected: hex"),
    }
  }
}

#[derive(Debug, Clone, Hash)]
pub enum HashSalt {
  None,
  Salt(String),
}

impl From<Option<String>> for HashSalt {
  fn from(value: Option<String>) -> Self {
    match value {
      Some(salt) => Self::Salt(salt),
      None => Self::None,
    }
  }
}

#[derive(Clone)]
pub enum RspackHash {
  Xxhash64(Box<Xxh64>),
  MD4(Box<md4::Md4>),
  SHA256(Box<sha2::Sha256>),
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
    if !matches!(salt, HashSalt::None) {
      salt.hash(&mut this);
    }
    this
  }

  pub fn digest(self, digest: &HashDigest) -> RspackHashDigest {
    // The maximum value of sha256, the largest possible hash
    let mut result = [0; 32];
    let len;

    match self {
      RspackHash::Xxhash64(hasher) => {
        let buf = hasher.finish().to_be_bytes();
        len = buf.len();
        result[..len].copy_from_slice(&buf);
      }
      RspackHash::MD4(hash) => {
        let buf = hash.finalize();
        len = buf.len();
        result[..len].copy_from_slice(&buf);
      }
      RspackHash::SHA256(hash) => {
        let buf = hash.finalize();
        len = buf.len();
        result[..len].copy_from_slice(&buf);
      }
    }

    RspackHashDigest::new(&result[..len], digest)
  }
}

impl Hasher for RspackHash {
  fn finish(&self) -> u64 {
    match self {
      RspackHash::Xxhash64(hasher) => hasher.finish(),
      RspackHash::MD4(hasher) => {
        // finalize take ownership, so we need to clone it
        let hash = (**hasher).clone().finalize();
        let msb_u64: u64 = ((hash[0] as u64) << 56)
          | ((hash[1] as u64) << 48)
          | ((hash[2] as u64) << 40)
          | ((hash[3] as u64) << 32)
          | ((hash[4] as u64) << 24)
          | ((hash[5] as u64) << 16)
          | ((hash[6] as u64) << 8)
          | (hash[7] as u64);
        msb_u64
      }
      RspackHash::SHA256(hasher) => {
        let hash = (**hasher).clone().finalize();
        let msb_u64: u64 = ((hash[0] as u64) << 56)
          | ((hash[1] as u64) << 48)
          | ((hash[2] as u64) << 40)
          | ((hash[3] as u64) << 32)
          | ((hash[4] as u64) << 24)
          | ((hash[5] as u64) << 16)
          | ((hash[6] as u64) << 8)
          | (hash[7] as u64);
        msb_u64
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

impl Hash for RspackHashDigest {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.encoded.hash(state);
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
