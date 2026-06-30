use rspack_hash::{
  HashDigest, HashFunction, HashSalt, RspackHash, RspackHashDigest, RspackHashable, write_u64_hex,
};

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
  let salt = HashSalt::Salt("salt".into());
  let salted = RspackHash::with_salt(&HashFunction::Xxhash64, &salt)
    .digest(&HashDigest::Hex)
    .encoded()
    .to_string();

  let mut expected = RspackHash::new(&HashFunction::Xxhash64);
  expected.write(b"salt");
  let expected = expected.digest(&HashDigest::Hex).encoded().to_string();

  assert_eq!(salted, expected);
}

#[test]
fn writes_u64_hex_without_leading_zeroes() {
  for (value, expected) in [
    (0, "0"),
    (0x0f, "f"),
    (0x10, "10"),
    (0x0abc, "abc"),
    (u64::MAX, "ffffffffffffffff"),
  ] {
    let mut actual = RspackHash::new(&HashFunction::Xxhash64);
    write_u64_hex(value, &mut actual);
    let actual = actual.digest(&HashDigest::Hex).encoded().to_string();

    let mut expected_hash = RspackHash::new(&HashFunction::Xxhash64);
    expected_hash.write(expected.as_bytes());
    let expected = expected_hash.digest(&HashDigest::Hex).encoded().to_string();

    assert_eq!(actual, expected);
  }
}

#[test]
fn derive_rspack_hashable_skips_marked_fields() {
  #[derive(RspackHashable)]
  struct Value {
    content: &'static str,
    #[rspack_hash(skip)]
    _cached: &'static str,
  }

  let mut derived = RspackHash::new(&HashFunction::Xxhash64);
  derived.update(&Value {
    content: "payload",
    _cached: "cache",
  });
  let derived = derived.digest(&HashDigest::Hex).encoded().to_string();

  let mut expected = RspackHash::new(&HashFunction::Xxhash64);
  expected.write(b"{");
  expected.write(b"content");
  expected.write(b":");
  expected.write(b"payload");
  expected.write(b"}");
  let expected = expected.digest(&HashDigest::Hex).encoded().to_string();

  assert_eq!(derived, expected);
}

#[test]
fn derive_rspack_hashable_respects_explicit_field_order() {
  #[derive(RspackHashable)]
  struct Value {
    #[rspack_hash(order = 1)]
    second: &'static str,
    #[rspack_hash(order = 0)]
    first: &'static str,
  }

  let mut derived = RspackHash::new(&HashFunction::Xxhash64);
  derived.update(&Value {
    first: "a",
    second: "b",
  });
  let derived = derived.digest(&HashDigest::Hex).encoded().to_string();

  let mut expected = RspackHash::new(&HashFunction::Xxhash64);
  expected.write(b"{");
  expected.write(b"first");
  expected.write(b":");
  expected.write(b"a");
  expected.write(b",");
  expected.write(b"second");
  expected.write(b":");
  expected.write(b"b");
  expected.write(b"}");
  let expected = expected.digest(&HashDigest::Hex).encoded().to_string();

  assert_eq!(derived, expected);
}

#[test]
fn option_rspack_hashable_skips_none() {
  let mut none = RspackHash::new(&HashFunction::Xxhash64);
  none.update(&Option::<&str>::None);
  let none = none.digest(&HashDigest::Hex).encoded().to_string();

  let empty = RspackHash::new(&HashFunction::Xxhash64)
    .digest(&HashDigest::Hex)
    .encoded()
    .to_string();

  assert_eq!(none, empty);

  let mut some = RspackHash::new(&HashFunction::Xxhash64);
  some.update(&Some("value"));
  let some = some.digest(&HashDigest::Hex).encoded().to_string();

  let mut expected = RspackHash::new(&HashFunction::Xxhash64);
  expected.write(b"value");
  let expected = expected.digest(&HashDigest::Hex).encoded().to_string();

  assert_eq!(some, expected);
}

#[test]
fn derive_rspack_hashable_can_hash_none_as_null() {
  #[derive(RspackHashable)]
  struct Value {
    #[rspack_hash(null_if_none)]
    value: Option<&'static str>,
  }

  let mut none = RspackHash::new(&HashFunction::Xxhash64);
  none.update(&Value { value: None });
  let none = none.digest(&HashDigest::Hex).encoded().to_string();

  let mut expected_none = RspackHash::new(&HashFunction::Xxhash64);
  expected_none.write(b"{");
  expected_none.write(b"value");
  expected_none.write(b":");
  expected_none.write(b"null");
  expected_none.write(b"}");
  let expected_none = expected_none.digest(&HashDigest::Hex).encoded().to_string();

  assert_eq!(none, expected_none);

  let mut some = RspackHash::new(&HashFunction::Xxhash64);
  some.update(&Value {
    value: Some("present"),
  });
  let some = some.digest(&HashDigest::Hex).encoded().to_string();

  let mut expected_some = RspackHash::new(&HashFunction::Xxhash64);
  expected_some.write(b"{");
  expected_some.write(b"value");
  expected_some.write(b":");
  expected_some.write(b"present");
  expected_some.write(b"}");
  let expected_some = expected_some.digest(&HashDigest::Hex).encoded().to_string();

  assert_eq!(some, expected_some);
}

#[test]
fn derive_rspack_hashable_hashes_option_field_names() {
  #[derive(RspackHashable)]
  struct Value {
    root: Option<&'static str>,
    commonjs: Option<&'static str>,
  }

  let mut root = RspackHash::new(&HashFunction::Xxhash64);
  root.update(&Value {
    root: Some("x"),
    commonjs: None,
  });
  let root = root.digest(&HashDigest::Hex).encoded().to_string();

  let mut commonjs = RspackHash::new(&HashFunction::Xxhash64);
  commonjs.update(&Value {
    root: None,
    commonjs: Some("x"),
  });
  let commonjs = commonjs.digest(&HashDigest::Hex).encoded().to_string();

  assert_ne!(root, commonjs);

  let mut expected_root = RspackHash::new(&HashFunction::Xxhash64);
  expected_root.write(b"{");
  expected_root.write(b"root");
  expected_root.write(b":");
  expected_root.write(b"x");
  expected_root.write(b"}");
  let expected_root = expected_root.digest(&HashDigest::Hex).encoded().to_string();

  assert_eq!(root, expected_root);
}

#[test]
fn derive_rspack_hashable_does_not_use_json_by_default() {
  #[derive(serde::Serialize, RspackHashable)]
  struct Value {
    content: &'static str,
  }

  let mut derived = RspackHash::new(&HashFunction::Xxhash64);
  derived.update(&Value { content: "payload" });
  let derived = derived.digest(&HashDigest::Hex).encoded().to_string();

  let mut expected = RspackHash::new(&HashFunction::Xxhash64);
  expected.write(b"{");
  expected.write(b"content");
  expected.write(b":");
  expected.write(b"payload");
  expected.write(b"}");
  let expected = expected.digest(&HashDigest::Hex).encoded().to_string();

  assert_eq!(derived, expected);
}

#[test]
fn derive_rspack_hashable_can_use_explicit_json() {
  #[derive(serde::Serialize, RspackHashable)]
  #[rspack_hash(json)]
  struct Value {
    content: &'static str,
  }

  let mut derived = RspackHash::new(&HashFunction::Xxhash64);
  derived.update(&Value { content: "payload" });
  let derived = derived.digest(&HashDigest::Hex).encoded().to_string();

  let mut expected = RspackHash::new(&HashFunction::Xxhash64);
  expected.write(br#"{"content":"payload"}"#);
  let expected = expected.digest(&HashDigest::Hex).encoded().to_string();

  assert_eq!(derived, expected);
}

#[test]
fn rspack_hash_object_hashes_object_fields() {
  let mut derived = RspackHash::new(&HashFunction::Xxhash64);
  rspack_hash::rspack_hash_object!(&mut derived, {
    "content" => "payload",
  });
  let derived = derived.digest(&HashDigest::Hex).encoded().to_string();

  let mut expected = RspackHash::new(&HashFunction::Xxhash64);
  expected.write(b"{");
  expected.write(b"content");
  expected.write(b":");
  expected.write(b"payload");
  expected.write(b"}");
  let expected = expected.digest(&HashDigest::Hex).encoded().to_string();

  assert_eq!(derived, expected);
}
