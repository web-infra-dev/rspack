use std::fmt::{Debug, Display};

use rspack_cacheable::cacheable;

use crate::ContextTypePrefix;

// Used to describe dependencies' types, see webpack's `type` getter in `Dependency`
// Note: This is almost the same with the old `ResolveKind`
#[cacheable]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DependencyType {
  #[default]
  Unknown,
  ExportInfoApi,
  Entry,
  // ESM import
  EsmImport,
  EsmImportSpecifier,
  // ESM export
  EsmExport,
  EsmExportImportedSpecifier,
  EsmExportSpecifier,
  EsmExportExpression,
  EsmExportHeader,
  // import()
  DynamicImport,
  // import() eager
  DynamicImportEager,
  // cjs require
  CjsRequire,
  // cjs full require
  CjsFullRequire,
  // cjs exports
  CjsExports,
  // consume shared exports (tree-shaking variant)
  ConsumeSharedExports,
  // module.exports = require(), should bailout in old tree shaking
  CjsExportRequire,
  // cjs self reference
  CjsSelfReference,
  // AMD
  AmdDefine,
  AmdRequireArray,
  AmdRequireContext,
  AmdRequire,
  AmdRequireItem,
  // new URL("./foo", import.meta.url)
  NewUrl,
  // new Worker()
  NewWorker,
  // create script url
  CreateScriptUrl,
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
  // css :export
  CssExport,
  // css modules local ident
  CssLocalIdent,
  // css modules self reference
  CssSelfReferenceLocalIdent,
  // context element
  ContextElement(ContextTypePrefix),
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
  // require.resolve context
  RequireResolveContext,
  // require.ensure
  RequireEnsure,
  // require.ensure item
  RequireEnsureItem,
  /// wasm import
  WasmImport,
  /// wasm export import
  WasmExportImported,
  /// static exports
  StaticExports,
  /// container exposed
  ContainerExposed,
  /// container entry,
  ContainerEntry,
  /// remote to external,
  RemoteToExternal,
  /// fallback
  RemoteToFallback,
  /// fallback item
  RemoteToFallbackItem,
  Provided,
  /// provide shared module
  ProvideSharedModule,
  /// provide module for shared
  ProvideModuleForShared,
  /// consume shared fallback
  ConsumeSharedFallback,
  /// Webpack is included
  WebpackIsIncluded,
  LoaderImport,
  LazyImport,
  ModuleDecorator,
  DllEntry,
  DelegatedSource,
  ExtractCSS,
  RstestModulePath,
  RstestMockModuleId,
  RstestHoistMock,
}

impl DependencyType {
  pub fn as_str(&self) -> &'static str {
    match self {
      DependencyType::Unknown => "unknown",
      DependencyType::Entry => "entry",
      DependencyType::EsmImport => "esm import",
      DependencyType::EsmExport => "esm export",
      DependencyType::EsmExportSpecifier => "esm export specifier",
      DependencyType::EsmExportImportedSpecifier => "esm export import specifier",
      DependencyType::EsmImportSpecifier => "esm import specifier",
      DependencyType::EsmExportExpression => "esm export expression",
      DependencyType::EsmExportHeader => "esm export header",
      DependencyType::DynamicImport => "import()",
      DependencyType::CjsRequire => "cjs require",
      DependencyType::CjsFullRequire => "cjs full require",
      DependencyType::CjsExports => "cjs exports",
      DependencyType::ConsumeSharedExports => "consume shared exports",
      DependencyType::CjsExportRequire => "cjs export require",
      DependencyType::CjsSelfReference => "cjs self exports reference",
      DependencyType::AmdDefine => "amd define",
      DependencyType::AmdRequireArray => "amd require array",
      DependencyType::AmdRequireContext => "amd require context",
      DependencyType::AmdRequire => "amd",
      DependencyType::AmdRequireItem => "amd require",
      DependencyType::NewUrl => "new URL()",
      DependencyType::NewWorker => "new Worker()",
      DependencyType::CreateScriptUrl => "create script url",
      DependencyType::ImportMetaHotAccept => "import.meta.webpackHot.accept",
      DependencyType::ImportMetaHotDecline => "import.meta.webpackHot.decline",
      DependencyType::ModuleHotAccept => "module.hot.accept",
      DependencyType::ModuleHotDecline => "module.hot.decline",
      DependencyType::CssUrl => "css url",
      DependencyType::CssImport => "css import",
      DependencyType::CssCompose => "css compose",
      DependencyType::CssExport => "css export",
      DependencyType::CssLocalIdent => "css local ident",
      DependencyType::CssSelfReferenceLocalIdent => "css self reference local ident",
      DependencyType::ContextElement(type_prefix) => match type_prefix {
        ContextTypePrefix::Import => "import() context element",
        ContextTypePrefix::Normal => "context element",
      },
      // TODO: mode
      DependencyType::ImportContext => "import context",
      DependencyType::DynamicImportEager => "import() eager",
      DependencyType::CommonJSRequireContext => "commonjs require context",
      DependencyType::RequireContext => "require.context",
      DependencyType::RequireResolve => "require.resolve",
      DependencyType::RequireResolveContext => "require.resolve context",
      DependencyType::RequireEnsure => "require.ensure",
      DependencyType::RequireEnsureItem => "require.ensure item",
      DependencyType::WasmImport => "wasm import",
      DependencyType::WasmExportImported => "wasm export imported",
      DependencyType::StaticExports => "static exports",
      DependencyType::LoaderImport => "loader import",
      DependencyType::ExportInfoApi => "export info api",
      // TODO: mode
      DependencyType::ImportMetaContext => "import.meta context",
      DependencyType::ContainerExposed => "container exposed",
      DependencyType::ContainerEntry => "container entry",
      DependencyType::DllEntry => "dll entry",
      DependencyType::RemoteToExternal => "remote to external",
      DependencyType::RemoteToFallback => "fallback",
      DependencyType::RemoteToFallbackItem => "fallback item",
      DependencyType::Provided => "provided",
      DependencyType::ProvideSharedModule => "provide shared module",
      DependencyType::ProvideModuleForShared => "provide module for shared",
      DependencyType::ConsumeSharedFallback => "consume shared fallback",
      DependencyType::WebpackIsIncluded => "__webpack_is_included__",
      DependencyType::LazyImport => "lazy import()",
      DependencyType::ModuleDecorator => "module decorator",
      DependencyType::DelegatedSource => "delegated source",
      DependencyType::ExtractCSS => "extract css",
      DependencyType::RstestModulePath => "rstest module path",
      DependencyType::RstestMockModuleId => "rstest mock module id",
      DependencyType::RstestHoistMock => "rstest hoist mock",
    }
  }
}

impl Display for DependencyType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}
