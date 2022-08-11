#[derive(Debug)]
pub enum Target {
  String(String),
  // we are not going to support StringArray in the near feature
  // StringArray(Vec<String>),
  None,
}
