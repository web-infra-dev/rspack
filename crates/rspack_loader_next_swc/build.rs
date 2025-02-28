fn main() {
  const CARGO_TOML: &str = include_str!("../../Cargo.toml");
  let workspace_toml = cargo_toml::Manifest::from_str(CARGO_TOML)
    .expect("Should parse cargo toml")
    .workspace;
  let swc_core_version = workspace_toml
    .as_ref()
    .and_then(|ws| {
      ws.dependencies.get("swc_core").and_then(|dep| match dep {
        cargo_toml::Dependency::Simple(s) => Some(&**s),
        cargo_toml::Dependency::Inherited(_) => unreachable!(),
        cargo_toml::Dependency::Detailed(d) => d.version.as_deref(),
      })
    })
    .expect("Should have `swc_core` version")
    .to_owned();
  println!("cargo::rustc-env=RSPACK_SWC_CORE_VERSION={swc_core_version}");
}
