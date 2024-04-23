pub fn has_client_directive(directives: &Vec<String>) -> bool {
  // TODO: client directives should config by plugin options
  let client_directives = vec!["use client"];
  directives
    .iter()
    .any(|item| client_directives.contains(&item.as_str()))
}
