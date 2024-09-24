use cow_utils::CowUtils;

pub fn to_normal_comment(str: &str) -> String {
  if str.is_empty() {
    return String::new();
  }
  format!("/* {} */", str.cow_replace("*/", "* /"))
}
