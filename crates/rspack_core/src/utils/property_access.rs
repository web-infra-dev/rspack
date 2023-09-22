use once_cell::sync::Lazy;
use regex::Regex;

static SAFE_IDENTIFIER_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^[_a-zA-Z$][_a-zA-Z$0-9]*$").expect("should init regex"));
const RESERVED_IDENTIFIER: [&str; 37] = [
  "break",
  "case",
  "catch",
  "class",
  "const",
  "continue",
  "debugger",
  "default",
  "delete",
  "do",
  "else",
  "enum",
  "export",
  "extends",
  "false",
  "finally",
  "for",
  "function",
  "if",
  "import",
  "in",
  "instanceof",
  "new",
  "null",
  "package",
  "return",
  "super",
  "switch",
  "this",
  "throw",
  "true",
  "try",
  "typeof",
  "var",
  "void",
  "while",
  "with",
];

pub fn property_access<S: AsRef<str>>(o: impl IntoIterator<Item = S>, start: usize) -> String {
  o.into_iter()
    .skip(start)
    .fold(String::default(), |mut str, property| {
      let property = property.as_ref();
      if SAFE_IDENTIFIER_REGEX.is_match(property)
        && !RESERVED_IDENTIFIER.contains(&property.as_ref())
      {
        str.push_str(format!(".{property}").as_str());
      } else {
        str.push_str(
          format!(
            "[{}]",
            serde_json::to_string(property).expect("should render property")
          )
          .as_str(),
        );
      }
      str
    })
}
