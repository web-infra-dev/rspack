use rspack_cacheable::{__private::rkyv::util::AlignedVec, from_bytes, to_bytes};

#[test]
fn aligned_bytes_deserialize_without_realignment() {
  let expected = (42_u64, "aligned cache value".to_string(), vec![1_u32, 2, 3]);
  let encoded = to_bytes(&expected, &()).expect("value should serialize");
  let mut aligned = AlignedVec::<16>::new();
  aligned.extend_from_slice(&encoded);

  assert!((aligned.as_ptr() as usize).is_multiple_of(16));
  assert_eq!(
    from_bytes::<(u64, String, Vec<u32>), _>(&aligned, &())
      .expect("aligned bytes should deserialize"),
    expected
  );
}

#[test]
fn large_aligned_archive_deserializes_without_realignment() {
  let expected = vec![0xab_u8; 1024 * 1024];
  let encoded = to_bytes(&expected, &()).expect("large value should serialize");
  let mut aligned = AlignedVec::<16>::new();
  aligned.extend_from_slice(&encoded);

  assert!((aligned.as_ptr() as usize).is_multiple_of(16));
  assert_eq!(
    from_bytes::<Vec<u8>, _>(&aligned, &()).expect("large aligned bytes should deserialize"),
    expected
  );
}

#[test]
fn unaligned_bytes_preserve_realignment_fallback() {
  let expected = (99_u64, "unaligned cache value".to_string(), vec![4_u32, 5]);
  let encoded = to_bytes(&expected, &()).expect("value should serialize");
  let mut aligned = AlignedVec::<16>::new();
  aligned.push(0);
  aligned.extend_from_slice(&encoded);
  let unaligned = &aligned[1..];

  assert!(!(unaligned.as_ptr() as usize).is_multiple_of(16));
  assert_eq!(
    from_bytes::<(u64, String, Vec<u32>), _>(unaligned, &())
      .expect("unaligned bytes should deserialize through the fallback"),
    expected
  );
}

#[test]
fn aligned_bytes_still_validate_archived_data() {
  let expected = (7_u64, "validated cache value".to_string());
  let encoded = to_bytes(&expected, &()).expect("value should serialize");
  let mut aligned = AlignedVec::<16>::new();
  aligned.extend_from_slice(&encoded);

  assert!(from_bytes::<(u64, String), _>(&aligned[..aligned.len() - 1], &()).is_err());
}
