mod entry;
mod span;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;

pub use entry::*;
use once_cell::sync::Lazy;
use rspack_util::ext::AsAny;
use serde::Serialize;
pub use span::SpanExt;
mod runtime_template;
pub use runtime_template::*;
mod runtime_requirements_dependency;
pub use runtime_requirements_dependency::RuntimeRequirementsDependency;
mod context_element_dependency;
mod dependency_macro;
pub use context_element_dependency::*;
mod const_dependency;
use std::{
  any::Any,
  borrow::Cow,
  fmt::{Debug, Display},
  hash::Hash,
};

pub use const_dependency::ConstDependency;
mod dependency_template;
pub use dependency_template::*;
use dyn_clone::{clone_trait_object, DynClone};

use crate::{
  ChunkGroupOptions, ConnectionState, Context, ContextMode, ContextOptions, ErrorSpan, ModuleGraph,
  ModuleGraphConnection, ReferencedExport, RuntimeSpec,
};

// Used to describe dependencies' types, see webpack's `type` getter in `Dependency`
// Note: This is almost the same with the old `ResolveKind`
#[derive(Default, Clone, PartialEq, Eq, Hash, Debug)]
pub enum DependencyType {
  #[default]
  Unknown,
  ExportInfoApi,
  Entry,
  // Harmony import
  EsmImport,
  EsmImportSpecifier,
  // Harmony export
  EsmExport,
  EsmExportImportedSpecifier,
  // import()
  DynamicImport,
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
  Custom(Cow<'static, str>),
}

impl Display for DependencyType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DependencyType::Unknown => write!(f, "unknown"),
      DependencyType::Entry => write!(f, "entry"),
      DependencyType::EsmImport => write!(f, "esm import"),
      DependencyType::EsmExport => write!(f, "esm export"),
      DependencyType::EsmExportImportedSpecifier => write!(f, "esm export import specifier"),
      DependencyType::EsmImportSpecifier => write!(f, "esm import specifier"),
      DependencyType::DynamicImport => write!(f, "dynamic import"),
      DependencyType::CjsRequire => write!(f, "cjs require"),
      DependencyType::NewUrl => write!(f, "new URL()"),
      DependencyType::NewWorker => write!(f, "new Worker()"),
      DependencyType::ImportMetaHotAccept => write!(f, "import.meta.webpackHot.accept"),
      DependencyType::ImportMetaHotDecline => write!(f, "import.meta.webpackHot.decline"),
      DependencyType::ModuleHotAccept => write!(f, "module.hot.accept"),
      DependencyType::ModuleHotDecline => write!(f, "module.hot.decline"),
      DependencyType::CssUrl => write!(f, "css url"),
      DependencyType::CssImport => write!(f, "css import"),
      DependencyType::CssCompose => write!(f, "css compose"),
      DependencyType::ContextElement => write!(f, "context element"),
      DependencyType::ImportContext => write!(f, "import context"),
      DependencyType::CommonJSRequireContext => write!(f, "commonjs require context"),
      DependencyType::RequireContext => write!(f, "require.context"),
      DependencyType::RequireResolve => write!(f, "require.resolve"),
      DependencyType::WasmImport => write!(f, "wasm import"),
      DependencyType::WasmExportImported => write!(f, "wasm export imported"),
      DependencyType::StaticExports => write!(f, "static exports"),
      DependencyType::Custom(ty) => write!(f, "custom {ty}"),
      DependencyType::ExportInfoApi => write!(f, "export info api"),
    }
  }
}

#[derive(Default, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum DependencyCategory {
  #[default]
  Unknown,
  Esm,
  CommonJS,
  Url,
  CssImport,
  CssCompose,
  Wasm,
  Worker,
}

impl From<&str> for DependencyCategory {
  fn from(value: &str) -> Self {
    match value {
      "esm" => Self::Esm,
      "commonjs" => Self::CommonJS,
      "url" => Self::Url,
      "wasm" => Self::Wasm,
      "css-import" => Self::CssImport,
      "css-compose" => Self::CssCompose,
      "worker" => Self::Worker,
      "unknown" => Self::Unknown,
      _ => unimplemented!("DependencyCategory {}", value),
    }
  }
}

impl Display for DependencyCategory {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DependencyCategory::Unknown => write!(f, "unknown"),
      DependencyCategory::Esm => write!(f, "esm"),
      DependencyCategory::CommonJS => write!(f, "commonjs"),
      DependencyCategory::Url => write!(f, "url"),
      DependencyCategory::CssImport => write!(f, "css-import"),
      DependencyCategory::CssCompose => write!(f, "css-compose"),
      DependencyCategory::Wasm => write!(f, "wasm"),
      DependencyCategory::Worker => write!(f, "worker"),
    }
  }
}

pub trait Dependency: AsAny + DynClone + Send + Sync + Debug {
  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Unknown
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::Unknown
  }

  fn get_context(&self) -> Option<&Context> {
    None
  }
}

pub enum ExportsReferencedType {
  No,     // NO_EXPORTS_REFERENCED
  Object, // EXPORTS_OBJECT_REFERENCED
  Value(Vec<ReferencedExport>),
}

impl Dependency for Box<dyn Dependency> {
  fn category(&self) -> &DependencyCategory {
    (**self).category()
  }

  fn dependency_type(&self) -> &DependencyType {
    (**self).dependency_type()
  }
}

pub trait AsModuleDependency {
  fn as_module_dependency(&self) -> Option<&dyn ModuleDependency> {
    None
  }
}

impl<T: ModuleDependency> AsModuleDependency for T {
  fn as_module_dependency(&self) -> Option<&dyn ModuleDependency> {
    Some(self)
  }
}

pub type DependencyConditionFn =
  Box<dyn Fn(&ModuleGraphConnection, &RuntimeSpec, &ModuleGraph) -> ConnectionState>;
pub enum DependencyCondition {
  Nil,
  False,
  Fn(DependencyConditionFn),
}

pub trait ModuleDependency: Dependency {
  fn id(&self) -> &DependencyId;
  fn request(&self) -> &str;
  fn user_request(&self) -> &str;
  fn span(&self) -> Option<&ErrorSpan>;
  fn weak(&self) -> bool {
    false
  }
  fn set_request(&mut self, request: String);

  // TODO should split to `ModuleDependency` and `ContextDependency`
  fn options(&self) -> Option<&ContextOptions> {
    None
  }
  fn get_optional(&self) -> bool {
    false
  }

  fn as_code_generatable_dependency(&self) -> Option<&dyn DependencyTemplate> {
    None
  }

  // TODO: wired to place ChunkGroupOptions on dependency, should place on AsyncDependenciesBlock
  fn group_options(&self) -> Option<&ChunkGroupOptions> {
    None
  }

  fn get_condition(&self, _module_graph: &ModuleGraph) -> DependencyCondition {
    DependencyCondition::Nil
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _runtime: &RuntimeSpec,
  ) -> ExportsReferencedType {
    ExportsReferencedType::Object
  }

  // an identifier to merge equal requests
  fn resource_identifier(&self) -> Option<&str> {
    None
  }
}

impl ModuleDependency for Box<dyn ModuleDependency> {
  fn id(&self) -> &DependencyId {
    (**self).id()
  }

  fn request(&self) -> &str {
    (**self).request()
  }

  fn user_request(&self) -> &str {
    (**self).user_request()
  }

  fn span(&self) -> Option<&ErrorSpan> {
    (**self).span()
  }

  fn weak(&self) -> bool {
    (**self).weak()
  }

  fn options(&self) -> Option<&ContextOptions> {
    (**self).options()
  }

  fn get_optional(&self) -> bool {
    (**self).get_optional()
  }

  fn group_options(&self) -> Option<&ChunkGroupOptions> {
    (**self).group_options()
  }

  fn set_request(&mut self, request: String) {
    (**self).set_request(request);
  }

  fn as_code_generatable_dependency(&self) -> Option<&dyn DependencyTemplate> {
    (**self).as_code_generatable_dependency()
  }
}

impl Dependency for Box<dyn ModuleDependency> {
  fn category(&self) -> &DependencyCategory {
    (**self).category()
  }

  fn dependency_type(&self) -> &DependencyType {
    (**self).dependency_type()
  }

  fn get_context(&self) -> Option<&Context> {
    (**self).get_context()
  }
}

impl dyn Dependency + '_ {
  pub fn downcast_ref<D: Any>(&self) -> Option<&D> {
    self.as_any().downcast_ref::<D>()
  }

  pub fn downcast_mut<D: Any>(&mut self) -> Option<&mut D> {
    self.as_any_mut().downcast_mut::<D>()
  }
}

clone_trait_object!(Dependency);
clone_trait_object!(ModuleDependency);

pub type BoxModuleDependency = Box<dyn ModuleDependency>;
pub type BoxDependency = Box<dyn Dependency>;

pub fn is_async_dependency(dep: &BoxModuleDependency) -> bool {
  if matches!(dep.dependency_type(), DependencyType::DynamicImport) {
    return true;
  }
  if matches!(dep.dependency_type(), DependencyType::NewWorker) {
    return true;
  }
  if matches!(dep.dependency_type(), DependencyType::ContextElement) {
    if let Some(options) = dep.options() {
      return matches!(options.mode, ContextMode::Lazy | ContextMode::LazyOnce);
    }
  }
  false
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct DependencyId(usize);

pub static DEPENDENCY_ID: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(0));

impl DependencyId {
  pub fn new() -> Self {
    Self(DEPENDENCY_ID.fetch_add(1, Relaxed))
  }
}
impl Default for DependencyId {
  fn default() -> Self {
    Self::new()
  }
}

impl std::ops::Deref for DependencyId {
  type Target = usize;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<usize> for DependencyId {
  fn from(id: usize) -> Self {
    Self(id)
  }
}

// should move to rspack_plugin_javascript
pub mod needs_refactor {
  use once_cell::sync::Lazy;
  use regex::Regex;
  use swc_core::{
    common::{EqIgnoreSpan, Spanned, SyntaxContext, DUMMY_SP},
    ecma::{
      ast::{
        Expr, ExprOrSpread, Id, Ident, ImportDecl, Lit, MemberExpr, MemberProp, MetaPropExpr,
        MetaPropKind, ModuleExportName, NewExpr,
      },
      atoms::{js_word, JsWord},
      visit::Visit,
    },
  };

  use crate::SpanExt;

  pub fn match_new_url(new_expr: &NewExpr) -> Option<(u32, u32, String)> {
    fn is_import_meta_url(expr: &Expr) -> bool {
      static IMPORT_META: Lazy<Expr> = Lazy::new(|| {
        Expr::Member(MemberExpr {
          span: DUMMY_SP,
          obj: Box::new(Expr::MetaProp(MetaPropExpr {
            span: DUMMY_SP,
            kind: MetaPropKind::ImportMeta,
          })),
          prop: MemberProp::Ident(Ident {
            span: DUMMY_SP,
            sym: "url".into(),
            optional: false,
          }),
        })
      });
      Ident::within_ignored_ctxt(|| expr.eq_ignore_span(&IMPORT_META))
    }

    if matches!(&*new_expr.callee, Expr::Ident(Ident { sym: js_word!("URL"), .. }))
    && let Some(args) = &new_expr.args
    && let (Some(first), Some(second)) = (args.first(), args.get(1))
    && let (
      ExprOrSpread { spread: None, expr: box Expr::Lit(Lit::Str(path)) },
      ExprOrSpread { spread: None, expr: box expr },
    ) = (first, second) && is_import_meta_url(expr) {
      return Some((path.span.real_lo(), expr.span().real_hi(), path.value.to_string()))
    }
    None
  }

  static WORKER_FROM_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(.+?)(\(\))?\s+from\s+(.+)$").expect("invalid regex"));

  #[derive(Debug, Default)]
  pub struct WorkerSyntaxList {
    variables: Vec<WorkerSyntax>,
    globals: Vec<WorkerSyntax>,
  }

  impl WorkerSyntaxList {
    pub fn push(&mut self, syntax: WorkerSyntax) {
      if syntax.ctxt.is_some() {
        self.variables.push(syntax);
      } else {
        self.globals.push(syntax);
      }
    }

    fn find_worker_syntax(&self, ident: &Ident) -> Option<&WorkerSyntax> {
      (self.variables.iter().chain(self.globals.iter())).find(|s| s.matches(ident))
    }

    pub fn match_new_worker(&self, new_expr: &NewExpr) -> bool {
      matches!(&*new_expr.callee, Expr::Ident(ident) if self.find_worker_syntax(ident).is_some())
    }
  }

  impl Extend<WorkerSyntax> for WorkerSyntaxList {
    fn extend<T: IntoIterator<Item = WorkerSyntax>>(&mut self, iter: T) {
      for i in iter {
        self.push(i);
      }
    }
  }

  impl From<WorkerSyntaxScanner<'_>> for WorkerSyntaxList {
    fn from(value: WorkerSyntaxScanner) -> Self {
      value.result
    }
  }

  #[derive(Debug, PartialEq, Eq)]
  pub struct WorkerSyntax {
    word: JsWord,
    ctxt: Option<SyntaxContext>,
  }

  impl WorkerSyntax {
    pub fn new(word: JsWord, ctxt: Option<SyntaxContext>) -> Self {
      Self { word, ctxt }
    }

    pub fn matches(&self, ident: &Ident) -> bool {
      if let Some(ctxt) = self.ctxt {
        let (word, id_ctxt) = ident.to_id();
        word == self.word && id_ctxt == ctxt
      } else {
        self.word == ident.sym
      }
    }
  }

  pub struct WorkerSyntaxScanner<'a> {
    result: WorkerSyntaxList,
    caps: Vec<(&'a str, &'a str)>,
  }

  pub const DEFAULT_WORKER_SYNTAX: &[&str] =
    &["Worker", "SharedWorker", "Worker from worker_threads"];

  impl<'a> WorkerSyntaxScanner<'a> {
    pub fn new(syntax: &'a [&'a str]) -> Self {
      let mut result = WorkerSyntaxList::default();
      let mut caps = Vec::new();
      for s in syntax {
        if let Some(captures) = WORKER_FROM_REGEX.captures(s)
        && let Some(ids) = captures.get(1)
        && let Some(source) = captures.get(3) {
          caps.push((ids.as_str(), source.as_str()));
        } else {
          result.push(WorkerSyntax::new(JsWord::from(*s), None))
        }
      }
      Self { result, caps }
    }
  }

  impl Visit for WorkerSyntaxScanner<'_> {
    fn visit_import_decl(&mut self, decl: &ImportDecl) {
      let source = &*decl.src.value;
      let found = self
        .caps
        .iter()
        .filter(|cap| cap.1 == source)
        .flat_map(|cap| {
          if cap.0 == "default" {
            decl
              .specifiers
              .iter()
              .filter_map(|spec| spec.as_default())
              .map(|spec| spec.local.to_id())
              .collect::<Vec<Id>>()
          } else {
            decl
              .specifiers
              .iter()
              .filter_map(|spec| {
                spec.as_named().filter(|named| {
                  if let Some(imported) = &named.imported {
                    let s = match imported {
                      ModuleExportName::Ident(s) => &s.sym,
                      ModuleExportName::Str(s) => &s.value,
                    };
                    s == cap.0
                  } else {
                    &*named.local.sym == cap.0
                  }
                })
              })
              .map(|spec| spec.local.to_id())
              .collect::<Vec<Id>>()
          }
        })
        .map(|pair| WorkerSyntax::new(pair.0, Some(pair.1)));
      self.result.extend(found);
    }
  }
}
