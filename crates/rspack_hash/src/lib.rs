use std::{
  fmt,
  hash::{Hash, Hasher},
};

use data_encoding::HEXLOWER_PERMISSIVE;
use md4::Digest;
use smol_str::SmolStr;
use xxhash_rust::xxh3;

#[derive(Debug, Clone, Copy)]
pub enum HashFunction {
  Xxhash64,
  MD4,
}

impl From<&str> for HashFunction {
  fn from(value: &str) -> Self {
    match value {
      "xxhash64" => HashFunction::Xxhash64,
      "md4" => HashFunction::MD4,
      _ => unimplemented!("{}", value),
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
      _ => unimplemented!(),
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
  Xxhash64(Box<xxh3::Xxh3>),
  MD4(md4::Md4),
}

impl fmt::Debug for RspackHash {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Xxhash64(_) => write!(f, "RspackHash(Xxhash64)"),
      Self::MD4(_) => write!(f, "RspackHash(MD4)"),
    }
  }
}

impl RspackHash {
  pub fn new(function: &HashFunction) -> Self {
    match function {
      HashFunction::Xxhash64 => Self::Xxhash64(Box::new(xxh3::Xxh3::new())),
      HashFunction::MD4 => Self::MD4(md4::Md4::new()),
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
    let inner = match self {
      RspackHash::Xxhash64(hasher) => hasher.finish().to_le_bytes().to_vec(),
      RspackHash::MD4(hash) => hash.finalize().to_vec(),
    };
    RspackHashDigest::new(inner, digest)
  }
}

impl Hasher for RspackHash {
  fn finish(&self) -> u64 {
    match self {
      RspackHash::Xxhash64(hasher) => hasher.finish(),
      RspackHash::MD4(hasher) => {
        // finalize take ownership, so we need to clone it
        let hash = hasher.clone().finalize();
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
    }
  }
}

#[derive(Debug, Clone, Eq)]
pub struct RspackHashDigest {
  encoded: SmolStr,
}

impl RspackHashDigest {
  pub fn new(inner: Vec<u8>, digest: &HashDigest) -> Self {
    let encoded = match digest {
      HashDigest::Hex => HEXLOWER_PERMISSIVE.encode(&inner).into(),
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
