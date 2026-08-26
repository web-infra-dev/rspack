use rspack_error::{Result, error};
use rspack_hash::{HashFunction, RspackHasher};
use rustc_hash::FxHashSet;

pub(crate) const FULL_IDENTIFIER_LENGTH: usize = 11;
pub(crate) const FULL_LOWERCASE_ALPHANUMERIC_LENGTH: usize = 13;

const IDENTIFIER_START_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const IDENTIFIER_CONTINUE_CHARS: &[u8] =
  b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const LOWERCASE_ALPHANUMERIC_CHARS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";

pub(crate) fn normalize_min_length(min_length: Option<usize>) -> usize {
  min_length
    .filter(|min_length| *min_length != 0)
    .unwrap_or(1)
}

pub(crate) fn validate_min_length(
  min_length: usize,
  max_length: usize,
  plugin_name: &str,
) -> Result<()> {
  if min_length > max_length {
    return Err(error!(
      "'minLength' must not exceed {max_length} for {plugin_name}"
    ));
  }
  Ok(())
}

pub(crate) fn hash_identifier(identifier: &str) -> [u8; FULL_IDENTIFIER_LENGTH] {
  encode_identifier_hash(hash(identifier))
}

pub(crate) fn hash_lowercase_alphanumeric(
  identifier: &str,
) -> [u8; FULL_LOWERCASE_ALPHANUMERIC_LENGTH] {
  encode_lowercase_alphanumeric_hash(hash(identifier))
}

fn hash(identifier: &str) -> u64 {
  let mut hasher = RspackHasher::new(&HashFunction::Xxhash64);
  hasher.write(identifier.as_bytes());
  hasher.finish()
}

fn encode_identifier_hash(mut hash: u64) -> [u8; FULL_IDENTIFIER_LENGTH] {
  let mut identifier = [0; FULL_IDENTIFIER_LENGTH];
  identifier[0] = IDENTIFIER_START_CHARS[(hash % IDENTIFIER_START_CHARS.len() as u64) as usize];
  hash /= IDENTIFIER_START_CHARS.len() as u64;
  for character in &mut identifier[1..] {
    *character =
      IDENTIFIER_CONTINUE_CHARS[(hash % IDENTIFIER_CONTINUE_CHARS.len() as u64) as usize];
    hash /= IDENTIFIER_CONTINUE_CHARS.len() as u64;
  }
  debug_assert_eq!(hash, 0);
  identifier
}

fn encode_lowercase_alphanumeric_hash(mut hash: u64) -> [u8; FULL_LOWERCASE_ALPHANUMERIC_LENGTH] {
  let mut identifier = [0; FULL_LOWERCASE_ALPHANUMERIC_LENGTH];
  for character in &mut identifier {
    *character =
      LOWERCASE_ALPHANUMERIC_CHARS[(hash % LOWERCASE_ALPHANUMERIC_CHARS.len() as u64) as usize];
    hash /= LOWERCASE_ALPHANUMERIC_CHARS.len() as u64;
  }
  debug_assert_eq!(hash, 0);
  identifier
}

pub(crate) struct CompatHashedIdAssigner {
  min_length: usize,
  used_ids: FxHashSet<String>,
}

impl CompatHashedIdAssigner {
  pub(crate) fn new(min_length: usize, used_ids: FxHashSet<String>) -> Self {
    Self {
      min_length,
      used_ids,
    }
  }

  pub(crate) fn assign(&mut self, hash: &[u8]) -> Option<String> {
    // SAFETY: The hash encoders only emit ASCII characters.
    let hash = unsafe { std::str::from_utf8_unchecked(hash) };
    let id = (self.min_length..=hash.len()).find_map(|length| {
      let candidate = &hash[..length];
      if self.used_ids.contains(candidate) {
        None
      } else {
        Some(candidate.to_string())
      }
    })?;

    self.used_ids.insert(id.clone());
    Some(id)
  }
}
