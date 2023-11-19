use std::borrow::Cow;
use std::fmt::{Debug, Display};

use crate::ErrorSpan;

// Used to describe dependencies' types, see webpack's `type` getter in `Dependency`
// Note: This is almost the same with the old `ResolveKind`
#[derive(Default, Clone, PartialEq, Eq, Hash, Debug)]
pub enum DependencyType {
  #[default]
  Unknown,
  ExportInfoApi,
  Entry,
  // Harmony import
  EsmImport(/* HarmonyImportSideEffectDependency.span */ ErrorSpan), /* TODO: remove span after old tree shaking is removed */
  EsmImportSpecifier,
  // Harmony export
  EsmExport(ErrorSpan),
  EsmExportImportedSpecifier,
  EsmExportSpecifier,
  // import()
  DynamicImport,
  // import() eager
  DynamicImportEager,
  // cjs require
  CjsRequire,
  // new URL("./foo", import.meta.url)
  NewUrl,
  // new Worker()
  NewWorker,
  // import.meta.webpackHot.accept
  ImportMetaHotAccept,
  // import.meta.webpackHot.decline
  ImportMetaHotDecline,
  // module.hot.accept
  ModuleHotAccept,
  // module.hot.decline
  ModuleHotDecline,
  // css url()
  CssUrl,
  // css @import
  CssImport,
  // css modules compose
  CssCompose,
  // context element
  ContextElement,
  // import context
  ImportContext,
  // import.meta.webpackContext
  ImportMetaContext,
  // commonjs require context
  CommonJSRequireContext,
  // require.context
  RequireContext,
  // require.resolve
  RequireResolve,
  /// wasm import
  WasmImport,
  /// wasm export import
  WasmExportImported,
  /// static exports
  StaticExports,
  Custom(Box<str>), // TODO it will increase large layout size
}

impl DependencyType {
  pub fn as_str(&self) -> Cow<str> {
    match self {
      DependencyType::Unknown => Cow::Borrowed("unknown"),
      DependencyType::Entry => Cow::Borrowed("entry"),
      DependencyType::EsmImport(_) => Cow::Borrowed("esm import"),
      DependencyType::EsmExport(_) => Cow::Borrowed("esm export"),
      DependencyType::EsmExportSpecifier => Cow::Borrowed("esm export specifier"),
      DependencyType::EsmExportImportedSpecifier => Cow::Borrowed("esm export import specifier"),
      DependencyType::EsmImportSpecifier => Cow::Borrowed("esm import specifier"),
      DependencyType::DynamicImport => Cow::Borrowed("dynamic import"),
      DependencyType::CjsRequire => Cow::Borrowed("cjs require"),
      DependencyType::NewUrl => Cow::Borrowed("new URL()"),
      DependencyType::NewWorker => Cow::Borrowed("new Worker()"),
      DependencyType::ImportMetaHotAccept => Cow::Borrowed("import.meta.webpackHot.accept"),
      DependencyType::ImportMetaHotDecline => Cow::Borrowed("import.meta.webpackHot.decline"),
      DependencyType::ModuleHotAccept => Cow::Borrowed("module.hot.accept"),
      DependencyType::ModuleHotDecline => Cow::Borrowed("module.hot.decline"),
      DependencyType::CssUrl => Cow::Borrowed("css url"),
      DependencyType::CssImport => Cow::Borrowed("css import"),
      DependencyType::CssCompose => Cow::Borrowed("css compose"),
      DependencyType::ContextElement => Cow::Borrowed("context element"),
      // TODO: mode
      DependencyType::ImportContext => Cow::Borrowed("import context"),
      DependencyType::DynamicImportEager => Cow::Borrowed("import() eager"),
      DependencyType::CommonJSRequireContext => Cow::Borrowed("commonjs require context"),
      DependencyType::RequireContext => Cow::Borrowed("require.context"),
      DependencyType::RequireResolve => Cow::Borrowed("require.resolve"),
      DependencyType::WasmImport => Cow::Borrowed("wasm import"),
      DependencyType::WasmExportImported => Cow::Borrowed("wasm export imported"),
      DependencyType::StaticExports => Cow::Borrowed("static exports"),
      DependencyType::Custom(ty) => Cow::Owned(format!("custom {ty}")),
      DependencyType::ExportInfoApi => Cow::Borrowed("export info api"),
      // TODO: mode
      DependencyType::ImportMetaContext => Cow::Borrowed("import.meta context"),
    }
  }
}

impl Display for DependencyType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}
