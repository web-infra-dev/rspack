pub(crate) struct ParserType {
  pub(crate) is_typescript: bool,
  pub(crate) is_jsx: bool,
}
// return the parser type of a file based on its extension name
// tsx -> typescript + jsx
// ts -> typescript
// jsx -> jsx
// js -> {}
pub(crate) fn extension_name_to_parser_type(name: &str) -> ParserType {
  match name {
    "tsx" => ParserType {
      is_typescript: true,
      is_jsx: true,
    },
    "ts" => ParserType {
      is_typescript: true,
      is_jsx: false,
    },
    "jsx" => ParserType {
      is_typescript: false,
      is_jsx: true,
    },
    "js" => ParserType {
      is_typescript: false,
      is_jsx: false,
    },
    _ => ParserType {
      is_typescript: false,
      is_jsx: false,
    },
  }
}
