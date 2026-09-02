use rspack_macros::StringEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, StringEnum)]
enum TestStringEnum {
  First,
  DefaultMultipleWords,
  #[string_enum(rename = "multiple-words")]
  MultipleWords,
  #[string_enum(fallback)]
  Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, StringEnum)]
enum TestStringEnumWithValueFallback {
  First,
  #[string_enum(fallback)]
  Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, StringEnum)]
#[string_enum(rename_all = "kebab-case")]
enum TestStringEnumWithRenameAll {
  MultipleWords,
  #[string_enum(rename = "renamed")]
  ExplicitRename,
  #[string_enum(fallback)]
  Unknown,
}

#[test]
fn converts_between_enum_and_str() {
  assert_eq!(TestStringEnum::from("first"), TestStringEnum::First);
  assert_eq!(
    TestStringEnum::from("default_multiple_words"),
    TestStringEnum::DefaultMultipleWords
  );
  assert_eq!(
    TestStringEnum::from("multiple-words"),
    TestStringEnum::MultipleWords
  );
  assert_eq!(TestStringEnum::from("unknown"), TestStringEnum::Unknown);
  assert_eq!(TestStringEnum::First.as_str(), "first");
  assert_eq!(
    TestStringEnum::DefaultMultipleWords.as_str(),
    "default_multiple_words"
  );
  assert_eq!(TestStringEnum::MultipleWords.as_str(), "multiple-words");
}

#[test]
fn preserves_unknown_string_in_fallback() {
  let value = TestStringEnumWithValueFallback::from("custom-value");
  assert_eq!(
    value,
    TestStringEnumWithValueFallback::Custom("custom-value".to_string())
  );
  assert_eq!(value.as_str(), "custom-value");
  assert_eq!(TestStringEnumWithValueFallback::First.as_str(), "first");
}

#[test]
fn applies_rename_all_before_variant_rename() {
  assert_eq!(
    TestStringEnumWithRenameAll::from("multiple-words"),
    TestStringEnumWithRenameAll::MultipleWords
  );
  assert_eq!(
    TestStringEnumWithRenameAll::MultipleWords.as_str(),
    "multiple-words"
  );
  assert_eq!(
    TestStringEnumWithRenameAll::from("renamed"),
    TestStringEnumWithRenameAll::ExplicitRename
  );
  assert_eq!(
    TestStringEnumWithRenameAll::ExplicitRename.as_str(),
    "renamed"
  );
}
