use rspack_macros::StringEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, StringEnum)]
enum TestStringEnum {
  First,
  #[string_enum(rename = "multiple-words")]
  MultipleWords,
  #[string_enum(fallback)]
  Unknown,
}

#[test]
fn converts_between_enum_and_str() {
  assert_eq!(TestStringEnum::from("first"), TestStringEnum::First);
  assert_eq!(
    TestStringEnum::from("multiple-words"),
    TestStringEnum::MultipleWords
  );
  assert_eq!(TestStringEnum::from("unknown"), TestStringEnum::Unknown);
  assert_eq!(TestStringEnum::First.as_str(), "first");
  assert_eq!(TestStringEnum::MultipleWords.as_str(), "multiple-words");
}
