use std::{
  fmt,
  hash::{Hash, Hasher},
};

use data_encoding::HEXLOWER_PERMISSIVE;
use smol_str::SmolStr;
use xxhash_rust::xxh3;

#[derive(Debug, Clone, Copy)]
pub enum HashFunction {
  Xxhash64,
}

impl From<&str> for HashFunction {
  fn from(value: &str) -> Self {
    match value {
      "xxhash64" => HashFunction::Xxhash64,
      _ => unimplemented!(),
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
  Xxhash64(xxh3::Xxh3),
}

impl fmt::Debug for RspackHash {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Xxhash64(_) => write!(f, "RspackHash(Xxhash64)"),
    }
  }
}

impl RspackHash {
  pub fn new(function: &HashFunction) -> Self {
    match function {
      HashFunction::Xxhash64 => Self::Xxhash64(xxh3::Xxh3::new()),
    }
  }

  pub fn with_salt(function: &HashFunction, salt: &HashSalt) -> Self {
    let mut this = Self::new(function);
    salt.hash(&mut this);
    this
  }

  pub fn digest(self, digest: &HashDigest) -> RspackHashDigest {
    let inner = match self {
      RspackHash::Xxhash64(hasher) => hasher.finish(),
    };
    RspackHashDigest::new(inner, digest)
  }
}

impl Hasher for RspackHash {
  fn finish(&self) -> u64 {
    match self {
      RspackHash::Xxhash64(hasher) => hasher.finish(),
    }
  }

  fn write(&mut self, bytes: &[u8]) {
    match self {
      RspackHash::Xxhash64(hasher) => hasher.write(bytes),
    }
  }
}

#[derive(Debug, Clone, Eq)]
pub struct RspackHashDigest {
  inner: u64,
  encoded: SmolStr,
}

impl RspackHashDigest {
  pub fn new(inner: u64, digest: &HashDigest) -> Self {
    let encoded = match digest {
      HashDigest::Hex => HEXLOWER_PERMISSIVE.encode(&inner.to_le_bytes()).into(),
    };
    Self { inner, encoded }
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
    self.inner.hash(state);
  }
}

impl PartialEq for RspackHashDigest {
  fn eq(&self, other: &Self) -> bool {
    self.inner == other.inner
  }
}
