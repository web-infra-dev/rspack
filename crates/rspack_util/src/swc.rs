use swc_core::atoms::Atom;

pub fn normalize_custom_filename(source: &str) -> &str {
  if source.starts_with('<') && source.ends_with('>') {
    &source[1..source.len() - 1] // remove '<' and '>' for swc FileName::Custom
  } else {
    source
  }
}

pub fn join_jsword(arr: &[Atom], separator: &str) -> String {
  let mut ret = String::new();
  if let Some(item) = arr.first() {
    ret.push_str(item);
  }
  for item in arr.iter().skip(1) {
    ret.push_str(separator);
    ret.push_str(item);
  }
  ret
}

#[test]
fn test_normalize_custom_filename() {
  let input = "<custom_filename>";
  let expected_output = "custom_filename";
  assert_eq!(normalize_custom_filename(input), expected_output);
}
