use std::hash::Hasher;

use rspack_hash::{HashDigest, HashFunction, HashSalt, RspackHash, RspackHashDigest};

#[test]
fn encodes_base64_with_standard_padding() {
  let digest = RspackHashDigest::new(b"\xfb\xef\xff", &HashDigest::Base64);

  assert_eq!(digest.encoded(), "++//");

  let digest = RspackHashDigest::new(b"hello", &HashDigest::Base64);

  assert_eq!(digest.encoded(), "aGVsbG8=");
}

#[test]
fn encodes_base64url_without_padding() {
  let digest = RspackHashDigest::new(b"\xfb\xef\xff", &HashDigest::Base64Url);

  assert_eq!(digest.encoded(), "--__");

  let digest = RspackHashDigest::new(b"hello", &HashDigest::Base64Url);

  assert_eq!(digest.encoded(), "aGVsbG8");
}

#[test]
fn hash_salt_is_written_as_raw_bytes() {
  let salt = HashSalt::Salt("salt".to_string());
  let salted = RspackHash::with_salt(&HashFunction::Xxhash64, &salt)
    .digest(&HashDigest::Hex)
    .encoded()
    .to_string();

  let mut expected = RspackHash::new(&HashFunction::Xxhash64);
  expected.write(b"salt");
  let expected = expected.digest(&HashDigest::Hex).encoded().to_string();

  assert_eq!(salted, expected);
}
