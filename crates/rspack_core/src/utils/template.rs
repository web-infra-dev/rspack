pub fn to_normal_comment(str: &str) -> String {
  if str.is_empty() {
    return String::new();
  }
  return format!("/* {} */", str.replace("*/", "* /"));
}
