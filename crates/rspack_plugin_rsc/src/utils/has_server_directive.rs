pub fn has_server_directive(directives: &Vec<String>) -> bool {
  let server_directives = vec!["use server"];
  directives
    .iter()
    .any(|item| server_directives.contains(&item.as_str()))
}
