use derive_more::Debug;
use napi::Either;
use rspack_plugin_rstest::{
  RstestDynamicImportOriginOptions, RstestPluginOptions, RstestRequireResolveOriginOptions,
};

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawRstestRequireResolveOriginOptions {
  // Override the callee that replaces `require.resolve` in the rewrite.
  // When omitted, rstest uses `__rstest_require_resolve__`.
  pub function_name: Option<String>,
}

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawRstestDynamicImportOriginOptions {
  // Override the callee that replaces `import` in the rewrite. When omitted,
  // rstest falls back to `output.importFunctionName`.
  pub function_name: Option<String>,
}

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawRstestPluginOptions {
  // Inject __dirname and __filename to each module.
  pub inject_module_path_name: bool,
  // Replace import.meta.dirname and import.meta.filename of each module.
  pub import_meta_path_name: bool,
  // Hoist mock module to the top of the module.
  pub hoist_mock_module: bool,
  // Root of the manual mock directory.
  pub manual_mock_root: String,
  // Preserve `new URL("*.<ext>", import.meta.url)` expressions for specified extensions
  // instead of transforming them to asset imports.
  // This allows rstest to dynamically load modules (e.g., wasm) at runtime.
  // Example: [".wasm"] to preserve wasm URL expressions.
  pub preserve_new_url: Option<Vec<String>>,
  // Whether to handle global `rs` and `rstest` variables.
  // When false, only ESM imported variables are processed. Default is true.
  pub globals: Option<bool>,
  // When enabled, rewrite non-string-literal `import()` calls (template
  // literals, variables) to the configured callee and append the source
  // module's absolute path as an extra argument. The runtime uses it as the
  // base for relative specifier resolution so paths inside bundled deps
  // resolve to the source file's directory rather than the test entry's.
  //
  // Pass `true` to enable with the callee taken from `output.importFunctionName`,
  // or pass an object with `functionName` to override the callee independently.
  #[napi(ts_type = "boolean | { functionName?: string }")]
  pub inject_dynamic_import_origin: Option<Either<bool, RawRstestDynamicImportOriginOptions>>,

  // When enabled, rewrite `require.resolve()` calls to the configured callee
  // and append the source module's absolute path as an extra argument. The
  // runtime uses it as the base for relative specifier resolution so paths
  // inside bundled deps resolve to the source file's directory rather than
  // the test entry's.
  //
  // Pass `true` to enable with the default `__rstest_require_resolve__` callee,
  // or pass an object with `functionName` to override the callee independently.
  #[napi(ts_type = "boolean | { functionName?: string }")]
  pub inject_require_resolve_origin: Option<Either<bool, RawRstestRequireResolveOriginOptions>>,
}

impl From<RawRstestPluginOptions> for RstestPluginOptions {
  fn from(value: RawRstestPluginOptions) -> Self {
    let inject_dynamic_import_origin = match value.inject_dynamic_import_origin {
      None | Some(Either::A(false)) => None,
      Some(Either::A(true)) => Some(RstestDynamicImportOriginOptions::default()),
      Some(Either::B(opts)) => Some(RstestDynamicImportOriginOptions {
        function_name: opts.function_name,
      }),
    };

    let inject_require_resolve_origin = match value.inject_require_resolve_origin {
      None | Some(Either::A(false)) => None,
      Some(Either::A(true)) => Some(RstestRequireResolveOriginOptions::default()),
      Some(Either::B(opts)) => Some(RstestRequireResolveOriginOptions {
        function_name: opts.function_name,
      }),
    };
    Self {
      module_path_name: value.inject_module_path_name,
      hoist_mock_module: value.hoist_mock_module,
      import_meta_path_name: value.import_meta_path_name,
      manual_mock_root: value.manual_mock_root,
      preserve_new_url: value.preserve_new_url.unwrap_or_default(),
      globals: value.globals.unwrap_or(true),
      inject_dynamic_import_origin,
      inject_require_resolve_origin,
    }
  }
}
