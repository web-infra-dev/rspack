pub fn has_client_directive(directives: &Vec<String>) -> bool {
  // TODO: client directives should config by plugin options
  let client_directives = vec!["use client", "use client:island"];
  directives
    .iter()
    .any(|item| client_directives.contains(&item.as_str()))
}
