use rspack_plugin_mf::{
  EnhancedModuleFederationManifestPluginOptions, ManifestExposeOption, ManifestSharedOption,
  ModuleFederationManifestPlugin, ModuleFederationManifestPluginOptions, ShareScope,
};

fn legacy_options() -> ModuleFederationManifestPluginOptions {
  ModuleFederationManifestPluginOptions {
    name: Some("app".into()),
    global_name: None,
    stats_file_name: "mf-stats.json".into(),
    manifest_file_name: "mf-manifest.json".into(),
    disable_assets_analyze: false,
    remote_alias_map: Default::default(),
    exposes: vec![ManifestExposeOption {
      path: "./component.js".into(),
      name: "component".into(),
    }],
    shared: vec![ManifestSharedOption {
      name: "react".into(),
      version: Some("18.0.0".into()),
      required_version: Some("^18.0.0".into()),
      singleton: Some(true),
    }],
    build_info: None,
  }
}

#[test]
fn legacy_manifest_literals_and_constructor_compile() {
  let _manifest = ModuleFederationManifestPlugin::new(legacy_options());
  let _empty =
    ModuleFederationManifestPlugin::new(ModuleFederationManifestPluginOptions::default());
}

#[test]
fn legacy_manifest_metadata_keeps_default_identity() {
  let options: EnhancedModuleFederationManifestPluginOptions = legacy_options().into();
  assert_eq!(
    options.shared[0].share_scope,
    ShareScope::Single("default".into())
  );
  assert!(options.shared[0].layer.is_none());
  assert_eq!(options.shared[0].version.as_deref(), Some("18.0.0"));
  assert_eq!(
    options.shared[0].required_version.as_deref(),
    Some("^18.0.0")
  );
  assert_eq!(options.shared[0].singleton, Some(true));
  assert!(options.exposes[0].layer.is_none());
  assert_eq!(options.exposes[0].path, "./component.js");
}
