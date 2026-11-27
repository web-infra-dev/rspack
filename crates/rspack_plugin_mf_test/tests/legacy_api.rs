use std::sync::Arc;

use rspack_collections::Identifiable;
use rspack_core::{Context, LibraryOptions, runtime_mode::RuntimeMode};
use rspack_plugin_mf::{
  CollectSharedEntryPlugin, CollectSharedEntryPluginOptions, ConsumeOptions, ConsumeSharedModule,
  ConsumeSharedPlugin, ConsumeSharedPluginOptions, OptimizeSharedConfig, ProvideOptions,
  ProvideSharedModule, ProvideSharedPlugin, ProvideVersion, ShareScope, SharedContainerPlugin,
  SharedContainerPluginOptions, SharedUsedExportsOptimizerPlugin,
  SharedUsedExportsOptimizerPluginOptions,
};

fn consume_options() -> ConsumeOptions {
  ConsumeOptions {
    import: None,
    import_resolved: Some("/shared.js".into()),
    share_key: "react".into(),
    share_scope: ShareScope::Single("default".into()),
    required_version: None,
    package_name: None,
    strict_version: false,
    singleton: false,
    eager: false,
    tree_shaking_mode: None,
  }
}

// These literals and constructor calls deliberately use the public API from main,
// without defaults, builders, or any fields introduced by layer support.
#[test]
fn legacy_literals_and_constructors_compile() {
  let _provide = ProvideSharedPlugin::new(vec![(
    "react".into(),
    ProvideOptions {
      share_key: "react".into(),
      share_scope: ShareScope::Single("default".into()),
      version: None,
      eager: false,
      singleton: None,
      required_version: None,
      strict_version: None,
      tree_shaking_mode: None,
    },
  )]);
  let _consume = ConsumeSharedPlugin::new(ConsumeSharedPluginOptions {
    consumes: vec![("react".into(), Arc::new(consume_options()))],
    enhanced: false,
  });
  let _collect = CollectSharedEntryPlugin::new(CollectSharedEntryPluginOptions {
    consumes: vec![("react".into(), Arc::new(consume_options()))],
    filename: None,
  });
  let _optimizer = SharedUsedExportsOptimizerPlugin::new(SharedUsedExportsOptimizerPluginOptions {
    shared: vec![OptimizeSharedConfig {
      share_key: "react".into(),
      tree_shaking: true,
      used_exports: vec!["default".into()],
    }],
    inject_tree_shaking_used_exports: true,
    stats_file_name: None,
    manifest_file_name: None,
  });
  let _container = SharedContainerPlugin::new(SharedContainerPluginOptions {
    name: "react".into(),
    request: "/shared.js".into(),
    version: "1.0.0".into(),
    file_name: None,
    library: LibraryOptions {
      library_type: "var".into(),
      name: None,
      export: None,
      umd_named_define: None,
      auxiliary_comment: None,
      amd_container: None,
    },
  });
  let consume =
    ConsumeSharedModule::new(Context::from(""), consume_options(), RuntimeMode::Webpack);
  let provide = ProvideSharedModule::new(
    ShareScope::Single("default".into()),
    "react".into(),
    ProvideVersion::Version("1.0.0".into()),
    "/shared.js".into(),
    false,
    None,
    None,
    None,
    None,
    RuntimeMode::Webpack,
  );
  assert!(
    consume
      .identifier()
      .to_string()
      .starts_with("consume shared module (default) react@*")
  );
  assert!(
    provide
      .identifier()
      .to_string()
      .starts_with("provide shared module (default) react@1.0.0")
  );
}

#[test]
fn legacy_options_use_unlayered_defaults() {
  let consume: rspack_plugin_mf::EnhancedConsumeOptions = consume_options().into();
  assert!(consume.request.is_none());
  assert!(consume.issuer_layer.is_none());
  assert!(consume.layer.is_none());
  let optimize: rspack_plugin_mf::EnhancedOptimizeSharedConfig = OptimizeSharedConfig {
    share_key: "react".into(),
    tree_shaking: true,
    used_exports: vec![],
  }
  .into();
  assert_eq!(optimize.request, "react");
  assert_eq!(optimize.share_scope, ShareScope::Single("default".into()));
  assert!(optimize.issuer_layer.is_none());
  assert!(optimize.layer.is_none());
  let container: rspack_plugin_mf::EnhancedSharedContainerPluginOptions =
    SharedContainerPluginOptions {
      name: "react".into(),
      request: "/shared.js".into(),
      version: "1.0.0".into(),
      file_name: None,
      library: LibraryOptions {
        library_type: "var".into(),
        name: None,
        export: None,
        umd_named_define: None,
        auxiliary_comment: None,
        amd_container: None,
      },
    }
    .into();
  assert_eq!(container.share_key, "react");
  assert_eq!(container.share_scope, ShareScope::Single("default".into()));
  assert!(container.layer.is_none());
}
