use inflector::Inflector;

#[test]
fn test_kebab_case() {
  assert_eq!("camelCase".to_kebab_case(), "camel-case");
  assert_eq!("kebab-case".to_kebab_case(), "kebab-case");
  assert_eq!("lower case".to_kebab_case(), "lower-case");
  assert_eq!("PascalCase".to_kebab_case(), "pascal-case");
  assert_eq!(
    "SCREAMING_SNAKE_CASE".to_kebab_case(),
    "screaming-snake-case"
  );
  assert_eq!("Sentence case".to_kebab_case(), "sentence-case");
  assert_eq!("snake_case".to_kebab_case(), "snake-case");
  assert_eq!("Title Case".to_kebab_case(), "title-case");
  assert_eq!("Train-Case".to_kebab_case(), "train-case");
}

#[test]
fn test_snack_case() {
  assert_eq!("camelCase".to_snake_case(), "camel_case");
  assert_eq!("kebab-case".to_snake_case(), "kebab_case");
  assert_eq!("lower case".to_snake_case(), "lower_case");
  assert_eq!("PascalCase".to_snake_case(), "pascal_case");
  assert_eq!(
    "SCREAMING_SNAKE_CASE".to_snake_case(),
    "screaming_snake_case"
  );
  assert_eq!("Sentence case".to_snake_case(), "sentence_case");
  assert_eq!("snake_case".to_snake_case(), "snake_case");
  assert_eq!("Title Case".to_snake_case(), "title_case");
  assert_eq!("Train-Case".to_snake_case(), "train_case");
}

#[test]
fn test_camel_case() {
  assert_eq!("camelCase".to_camel_case(), "camelCase");
  assert_eq!("kebab-case".to_camel_case(), "kebabCase");
  assert_eq!("lower case".to_camel_case(), "lowerCase");
  assert_eq!("PascalCase".to_camel_case(), "pascalCase");
  assert_eq!("SCREAMING_SNAKE_CASE".to_camel_case(), "screamingSnakeCase");
  assert_eq!("Sentence case".to_camel_case(), "sentenceCase");
  assert_eq!("snake_case".to_camel_case(), "snakeCase");
  assert_eq!("Title Case".to_camel_case(), "titleCase");
  assert_eq!("Train-Case".to_camel_case(), "trainCase");
}
