use std::collections::HashMap;

#[derive(Debug)]
pub enum External {
  Object(HashMap<String, String>),
  String(String),
}
