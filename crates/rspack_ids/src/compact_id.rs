pub(crate) const FULL_IDENTIFIER_LENGTH: usize = 11;

const IDENTIFIER_START_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const IDENTIFIER_CONTINUE_CHARS: &[u8] =
  b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

pub(crate) fn encode_identifier_hash(mut hash: u64) -> [u8; FULL_IDENTIFIER_LENGTH] {
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
