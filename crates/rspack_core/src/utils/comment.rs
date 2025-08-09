use cow_utils::CowUtils;

#[inline]
pub fn to_comment(str: &str) -> String {
  if str.is_empty() {
    return String::new();
  }

  let result = str.cow_replace("*/", "* /");
  format!("/*! {result} */")
}

#[inline]
pub fn to_comment_with_nl(str: &str) -> String {
  if str.is_empty() {
    return String::new();
  }

  let result = str.cow_replace("*/", "* /");
  format!("/*! {result} */\n")
}
