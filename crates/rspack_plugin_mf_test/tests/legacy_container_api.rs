use rspack_core::LibraryOptions;
use rspack_plugin_mf::{ContainerPlugin, ContainerPluginOptions, ExposeOptions, ShareScope};

#[test]
fn legacy_container_literals_and_constructor_compile() {
  let options = ExposeOptions {
    name: Some("component".into()),
    import: vec!["./component.js".into()],
  };
  let _container = ContainerPlugin::new(ContainerPluginOptions {
    name: "app".into(),
    share_scope: ShareScope::Single("default".into()),
    library: LibraryOptions {
      library_type: "var".into(),
      name: None,
      export: None,
      umd_named_define: None,
      auxiliary_comment: None,
      amd_container: None,
    },
    runtime: None,
    filename: None,
    exposes: vec![("./component".into(), options)],
    enhanced: false,
  });
}

#[test]
fn legacy_expose_conversion_keeps_unlayered_imports() {
  let options: rspack_plugin_mf::EnhancedExposeOptions = ExposeOptions {
    name: Some("component".into()),
    import: vec!["./component.js".into()],
  }
  .into();
  assert_eq!(options.name.as_deref(), Some("component"));
  assert_eq!(options.import, ["./component.js"]);
  assert!(options.layer.is_none());
}
