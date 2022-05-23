use once_cell::sync::Lazy;
use regex::Regex;
static RE_LIST: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| {
  let v = vec![
    (
      Regex::new(r"(?is)<svg (.*?)>").unwrap(),
      "<svg $1 {...props}>",
    ),
    (Regex::new(r"(?is)<\?(.*?)\?>").unwrap(), ""),
    (Regex::new(r"(?is)<!--(.*?)-->").unwrap(), ""),
    (Regex::new(r"(?is)<!(.*?)>").unwrap(), ""),
    (
      Regex::new(r"(?is)<style(.*?)>(.*?)</style>").unwrap(),
      "<style$1>{`$2`}</style>",
    ),
  ];
  v
});

pub fn clean(text: &str) -> String {
  let result = RE_LIST.iter().fold(text.to_string(), |text, re| {
    re.0.replace_all(&text, re.1).to_string()
  });
  result
}
