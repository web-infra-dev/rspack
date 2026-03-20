use std::{
  collections::hash_map::Entry,
  sync::{Arc, LazyLock, Mutex},
};

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  BoxDependency, DependencyId, Module as ModuleTrait, ModuleGraph, ModuleIdentifier, ParseContext,
  ParseResult, ParserAndGenerator, SourceType,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
};
use rspack_error::{IntoTWithDiagnosticArray, Result, TWithDiagnosticArray, error};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use swc_core::{
  common::{DUMMY_SP, FileName, SourceMap, comments::SingleThreadedComments, sync::Lrc},
  ecma::{
    ast::*,
    codegen::{Config, Emitter, text_writer::JsWriter},
    parser::{Parser, StringInput, Syntax, TsSyntax, lexer::Lexer},
    visit::{Visit, VisitMut, VisitMutWith, VisitWith},
  },
};

use crate::dependency::{DtsDependency, DtsDependencyKind};

pub const DTS_MODULE_TYPE: &str = "dts";
pub const DTS_SOURCE_TYPE: &str = "dts";

pub fn to_dts_request(request: &str) -> String {
  if request.contains(".rspack[dts]!=!") {
    request.to_string()
  } else {
    format!("{request}.rspack[{DTS_MODULE_TYPE}]!=!{request}")
  }
}

#[derive(Debug, Clone)]
pub struct DtsRenderDecl {
  pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DtsDeclSpace {
  Type,
  Value,
  Both,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DtsDeclKey {
  pub module_identifier: String,
  pub local_name: String,
  pub space: DtsDeclSpace,
}

impl DtsDeclKey {
  pub fn id(&self) -> String {
    format!(
      "{}::{}::{:?}",
      self.module_identifier, self.local_name, self.space
    )
  }
}

#[derive(Debug, Clone)]
pub struct DtsImportBinding {
  pub local: String,
  pub imported: String,
  pub dependency_id: Option<DependencyId>,
  pub request: String,
  pub is_external: bool,
}

#[derive(Debug, Clone)]
pub struct DtsNamedReexport {
  pub exported: String,
  pub imported: String,
  pub dependency_id: Option<DependencyId>,
  pub request: String,
  pub is_external: bool,
  pub item: ModuleItem,
}

#[derive(Debug, Clone)]
pub struct DtsStarReexport {
  pub dependency_id: Option<DependencyId>,
  pub request: String,
  pub is_external: bool,
  pub item: ModuleItem,
}

#[derive(Debug, Clone)]
pub struct DtsDeclSummary {
  pub key: DtsDeclKey,
  pub export_names: Vec<String>,
  pub item: ModuleItem,
  pub references: FxHashSet<String>,
}

#[derive(Debug, Clone)]
pub struct DtsModuleSummary {
  pub module_identifier: ModuleIdentifier,
  pub declarations: FxHashMap<String, DtsDeclSummary>,
  pub imports: FxHashMap<String, DtsImportBinding>,
  pub named_reexports: Vec<DtsNamedReexport>,
  pub star_reexports: Vec<DtsStarReexport>,
  pub external_import_items: Vec<ModuleItem>,
  pub external_reexport_items: Vec<ModuleItem>,
}

#[derive(Debug, Clone)]
pub struct DtsModuleData {
  pub summary: DtsModuleSummary,
}

pub type DtsModuleStore = Arc<Mutex<FxHashMap<ModuleIdentifier, Arc<DtsModuleData>>>>;

#[cacheable]
#[derive(Debug, Clone)]
pub struct DtsParserAndGenerator {
  #[cacheable(with=rspack_cacheable::with::Skip)]
  store: DtsModuleStore,
  #[cacheable(with=rspack_cacheable::with::Skip)]
  external_requests: Arc<FxHashSet<String>>,
}

impl DtsParserAndGenerator {
  pub fn new(store: DtsModuleStore, external_requests: Arc<FxHashSet<String>>) -> Self {
    Self {
      store,
      external_requests,
    }
  }
}

pub fn parse_dts_source(
  module_identifier: ModuleIdentifier,
  source_text: String,
  external_requests: &FxHashSet<String>,
) -> Result<DtsModuleSummary> {
  let cm: Lrc<SourceMap> = Default::default();
  let comments = SingleThreadedComments::default();
  let fm = cm.new_source_file(
    FileName::Custom(module_identifier.to_string()).into(),
    source_text,
  );
  let lexer = Lexer::new(
    Syntax::Typescript(TsSyntax {
      dts: true,
      ..Default::default()
    }),
    Default::default(),
    StringInput::from(&*fm),
    Some(&comments),
  );
  let mut parser = Parser::new_from(lexer);
  let module = parser
    .parse_module()
    .map_err(|err| error!("Failed to parse dts module {}: {err:?}", module_identifier))?;

  if has_unsupported_syntax(&module) {
    return Err(error!(
      "Unsupported syntax found while parsing {}. MVP-1 does not support declaration merging, namespaces, augmentations, export =, import = require, or inline import() types.",
      module_identifier
    ));
  }

  let mut collector = SummaryCollector::new(module_identifier, external_requests);
  module.visit_with(&mut collector);
  collector.finalize()?;
  Ok(collector.into_summary(module_identifier))
}

#[cacheable_dyn]
#[async_trait]
impl ParserAndGenerator for DtsParserAndGenerator {
  fn source_types(&self, _module: &dyn ModuleTrait, _module_graph: &ModuleGraph) -> &[SourceType] {
    static SOURCE_TYPES: LazyLock<[SourceType; 1]> =
      LazyLock::new(|| [SourceType::Custom(DTS_SOURCE_TYPE.into())]);
    &SOURCE_TYPES[..]
  }

  async fn parse<'a>(
    &mut self,
    parse_context: ParseContext<'a>,
  ) -> Result<TWithDiagnosticArray<ParseResult>> {
    let ParseContext {
      source,
      module_identifier,
      module_type,
      build_info,
      build_meta,
      ..
    } = parse_context;

    if module_type.as_str() != DTS_MODULE_TYPE {
      return Err(error!(
        "DtsParserAndGenerator received unexpected module type {}",
        module_type
      ));
    }

    let source_text = source.source().into_string_lossy().into_owned();
    let mut summary = parse_dts_source(
      module_identifier,
      source_text,
      self.external_requests.as_ref(),
    )?;
    let mut dependencies = vec![];
    for binding in summary.imports.values_mut() {
      if !binding.is_external {
        let dependency = Box::new(DtsDependency::new(
          to_dts_request(&binding.request),
          DtsDependencyKind::Import,
        )) as BoxDependency;
        binding.dependency_id = Some(*dependency.id());
        dependencies.push(dependency);
      }
    }
    for reexport in &mut summary.named_reexports {
      if !reexport.is_external {
        let dependency = Box::new(DtsDependency::new(
          to_dts_request(&reexport.request),
          DtsDependencyKind::Reexport,
        )) as BoxDependency;
        reexport.dependency_id = Some(*dependency.id());
        dependencies.push(dependency);
      }
    }
    for reexport in &mut summary.star_reexports {
      if !reexport.is_external {
        let dependency = Box::new(DtsDependency::new(
          to_dts_request(&reexport.request),
          DtsDependencyKind::Reexport,
        )) as BoxDependency;
        reexport.dependency_id = Some(*dependency.id());
        dependencies.push(dependency);
      }
    }

    build_info.cacheable = true;
    build_info.module = true;
    build_meta.exports_type = rspack_core::BuildMetaExportsType::Namespace;

    self
      .store
      .lock()
      .expect("dts module store lock poisoned")
      .insert(module_identifier, Arc::new(DtsModuleData { summary }));

    Ok(
      ParseResult {
        dependencies,
        blocks: vec![],
        presentational_dependencies: vec![],
        code_generation_dependencies: vec![],
        source,
        side_effects_bailout: None,
      }
      .with_diagnostic(vec![]),
    )
  }

  fn size(&self, _module: &dyn ModuleTrait, _source_type: Option<&SourceType>) -> f64 {
    1.0
  }

  async fn generate(
    &self,
    _source: &BoxSource,
    _module: &dyn ModuleTrait,
    _generate_context: &mut rspack_core::GenerateContext,
  ) -> Result<BoxSource> {
    Ok(RawStringSource::from(String::new()).boxed())
  }

  fn get_concatenation_bailout_reason(
    &self,
    _module: &dyn ModuleTrait,
    _mg: &ModuleGraph,
    _cg: &rspack_core::ChunkGraph,
  ) -> Option<std::borrow::Cow<'static, str>> {
    Some("dts module does not participate in JS concatenation".into())
  }
}

#[derive(Debug)]
struct SummaryCollector<'a> {
  module_identifier: ModuleIdentifier,
  external_requests: &'a FxHashSet<String>,
  declarations: FxHashMap<String, DtsDeclSummary>,
  local_exports: Vec<(String, String)>,
  imports: Vec<(String, String, String, bool, ModuleItem)>,
  named_reexports: Vec<(String, String, String, bool, ModuleItem)>,
  star_reexports: Vec<(String, bool, ModuleItem)>,
  errors: Vec<String>,
}

impl<'a> SummaryCollector<'a> {
  fn new(module_identifier: ModuleIdentifier, external_requests: &'a FxHashSet<String>) -> Self {
    Self {
      module_identifier,
      external_requests,
      declarations: Default::default(),
      local_exports: vec![],
      imports: vec![],
      named_reexports: vec![],
      star_reexports: vec![],
      errors: vec![],
    }
  }

  fn finalize(&self) -> Result<()> {
    if let Some(message) = self.errors.first() {
      return Err(error!("{}", message));
    }
    Ok(())
  }

  fn into_summary(self, module_identifier: ModuleIdentifier) -> DtsModuleSummary {
    let mut declarations = self.declarations;
    for (local, exported) in self.local_exports {
      if let Some(summary) = declarations.get_mut(&local)
        && !summary.export_names.iter().any(|name| name == &exported)
      {
        summary.export_names.push(exported);
      }
    }

    let mut imports = FxHashMap::default();
    let mut external_import_items = vec![];
    for (local, imported, request, is_external, item) in self.imports {
      if is_external {
        external_import_items.push(item.clone());
      }
      imports.insert(
        local.clone(),
        DtsImportBinding {
          local,
          imported,
          dependency_id: None,
          request,
          is_external,
        },
      );
    }

    let mut named_reexports = vec![];
    let mut star_reexports = vec![];
    let mut external_reexport_items = vec![];

    for (exported, imported, request, is_external, item) in self.named_reexports {
      if is_external {
        external_reexport_items.push(item.clone());
      }
      named_reexports.push(DtsNamedReexport {
        exported,
        imported,
        dependency_id: None,
        request,
        is_external,
        item,
      });
    }

    for (request, is_external, item) in self.star_reexports {
      if is_external {
        external_reexport_items.push(item.clone());
      }
      star_reexports.push(DtsStarReexport {
        dependency_id: None,
        request,
        is_external,
        item,
      });
    }

    DtsModuleSummary {
      module_identifier,
      declarations,
      imports,
      named_reexports,
      star_reexports,
      external_import_items,
      external_reexport_items,
    }
  }

  fn add_decl(&mut self, local_name: String, space: DtsDeclSpace, item: ModuleItem) {
    let key = DtsDeclKey {
      module_identifier: self.module_identifier.to_string(),
      local_name: local_name.clone(),
      space,
    };
    let references = collect_references_from_item(&item, &local_name);
    match self.declarations.entry(local_name.clone()) {
      Entry::Occupied(_) => self.errors.push(format!(
        "Unsupported declaration merging in {} for symbol {}",
        self.module_identifier, local_name
      )),
      Entry::Vacant(vacant) => {
        vacant.insert(DtsDeclSummary {
          key,
          export_names: vec![],
          item,
          references,
        });
      }
    }
  }
}

impl Visit for SummaryCollector<'_> {
  fn visit_module_item(&mut self, item: &ModuleItem) {
    match item {
      ModuleItem::Stmt(Stmt::Decl(decl)) => {
        if let Some((name, space)) = decl_name_and_space(decl) {
          self.add_decl(name, space, item.clone());
        }
      }
      ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export_decl)) => {
        if let Some((name, space)) = decl_name_and_space(&export_decl.decl) {
          self.add_decl(name.clone(), space, item.clone());
          self.local_exports.push((name.clone(), name));
        }
      }
      ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) => {
        let request = import_decl
          .src
          .value
          .to_atom_lossy()
          .into_owned()
          .to_string();
        let is_external = self.external_requests.contains(&request) || !request.starts_with('.');
        for specifier in &import_decl.specifiers {
          match specifier {
            ImportSpecifier::Named(named) => {
              self.imports.push((
                named.local.sym.to_string(),
                named
                  .imported
                  .as_ref()
                  .map(imported_atom)
                  .unwrap_or_else(|| named.local.sym.to_string()),
                request.clone(),
                is_external,
                item.clone(),
              ));
            }
            ImportSpecifier::Default(default) => {
              self.imports.push((
                default.local.sym.to_string(),
                "default".to_string(),
                request.clone(),
                is_external,
                item.clone(),
              ));
            }
            ImportSpecifier::Namespace(_) => {
              self.errors.push(format!(
                "Unsupported namespace import in {}: {}",
                self.module_identifier, request
              ));
            }
          }
        }
      }
      ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(named)) => {
        if let Some(src) = &named.src {
          let request = src.value.to_atom_lossy().into_owned().to_string();
          let is_external = self.external_requests.contains(&request) || !request.starts_with('.');
          for specifier in &named.specifiers {
            match specifier {
              ExportSpecifier::Named(named_specifier) => {
                self.named_reexports.push((
                  named_specifier
                    .exported
                    .as_ref()
                    .map(export_name_atom)
                    .unwrap_or_else(|| export_name_atom(&named_specifier.orig)),
                  export_name_atom(&named_specifier.orig),
                  request.clone(),
                  is_external,
                  item.clone(),
                ));
              }
              _ => self.errors.push(format!(
                "Unsupported export specifier in {}",
                self.module_identifier
              )),
            }
          }
        } else {
          for specifier in &named.specifiers {
            match specifier {
              ExportSpecifier::Named(named_specifier) => {
                self.local_exports.push((
                  export_name_atom(&named_specifier.orig),
                  named_specifier
                    .exported
                    .as_ref()
                    .map(export_name_atom)
                    .unwrap_or_else(|| export_name_atom(&named_specifier.orig)),
                ));
              }
              _ => self.errors.push(format!(
                "Unsupported export specifier in {}",
                self.module_identifier
              )),
            }
          }
        }
      }
      ModuleItem::ModuleDecl(ModuleDecl::ExportAll(export_all)) => {
        let request = export_all
          .src
          .value
          .to_atom_lossy()
          .into_owned()
          .to_string();
        let is_external = self.external_requests.contains(&request) || !request.starts_with('.');
        self
          .star_reexports
          .push((request, is_external, item.clone()));
      }
      ModuleItem::ModuleDecl(ModuleDecl::TsImportEquals(_))
      | ModuleItem::ModuleDecl(ModuleDecl::TsExportAssignment(_))
      | ModuleItem::ModuleDecl(ModuleDecl::TsNamespaceExport(_))
      | ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(_))
      | ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(_)) => {
        self.errors.push(format!(
          "Unsupported declaration syntax in {}",
          self.module_identifier
        ));
      }
      _ => {}
    }
  }
}

fn has_unsupported_syntax(module: &swc_core::ecma::ast::Module) -> bool {
  struct UnsupportedSyntaxDetector {
    unsupported: bool,
  }

  impl Visit for UnsupportedSyntaxDetector {
    fn visit_ts_module_decl(&mut self, _: &TsModuleDecl) {
      self.unsupported = true;
    }

    fn visit_ts_import_type(&mut self, _: &TsImportType) {
      self.unsupported = true;
    }

    fn visit_export_default_decl(&mut self, _: &ExportDefaultDecl) {
      self.unsupported = true;
    }

    fn visit_export_default_expr(&mut self, _: &ExportDefaultExpr) {
      self.unsupported = true;
    }

    fn visit_ts_export_assignment(&mut self, _: &TsExportAssignment) {
      self.unsupported = true;
    }

    fn visit_ts_import_equals_decl(&mut self, _: &TsImportEqualsDecl) {
      self.unsupported = true;
    }
  }

  let mut detector = UnsupportedSyntaxDetector { unsupported: false };
  module.visit_with(&mut detector);
  detector.unsupported
}

fn decl_name_and_space(decl: &Decl) -> Option<(String, DtsDeclSpace)> {
  match decl {
    Decl::TsInterface(interface_decl) => {
      Some((interface_decl.id.sym.to_string(), DtsDeclSpace::Type))
    }
    Decl::TsTypeAlias(type_alias_decl) => {
      Some((type_alias_decl.id.sym.to_string(), DtsDeclSpace::Type))
    }
    Decl::Class(class_decl) => Some((class_decl.ident.sym.to_string(), DtsDeclSpace::Both)),
    Decl::Fn(fn_decl) => Some((fn_decl.ident.sym.to_string(), DtsDeclSpace::Value)),
    Decl::TsEnum(enum_decl) => Some((enum_decl.id.sym.to_string(), DtsDeclSpace::Both)),
    Decl::Var(var_decl) => var_decl
      .decls
      .first()
      .and_then(|decl| decl.name.as_ident())
      .map(|ident| (ident.sym.to_string(), DtsDeclSpace::Value)),
    _ => None,
  }
}

fn imported_atom(imported: &ModuleExportName) -> String {
  export_name_atom(imported)
}

fn export_name_atom(name: &ModuleExportName) -> String {
  match name {
    ModuleExportName::Ident(ident) => ident.sym.to_string(),
    ModuleExportName::Str(str_) => str_.value.to_atom_lossy().into_owned().to_string(),
  }
}

fn collect_references_from_item(item: &ModuleItem, own_name: &str) -> FxHashSet<String> {
  #[derive(Default)]
  struct RefCollector {
    refs: FxHashSet<String>,
    declared: Vec<String>,
  }

  impl Visit for RefCollector {
    fn visit_ident(&mut self, ident: &Ident) {
      if !self.declared.iter().any(|name| name == ident.sym.as_ref()) {
        self.refs.insert(ident.sym.to_string());
      }
    }

    fn visit_binding_ident(&mut self, ident: &BindingIdent) {
      self.declared.push(ident.id.sym.to_string());
    }

    fn visit_class_decl(&mut self, decl: &ClassDecl) {
      self.declared.push(decl.ident.sym.to_string());
      decl.class.visit_children_with(self);
    }

    fn visit_fn_decl(&mut self, decl: &FnDecl) {
      self.declared.push(decl.ident.sym.to_string());
      decl.function.visit_children_with(self);
    }

    fn visit_ts_interface_decl(&mut self, decl: &TsInterfaceDecl) {
      self.declared.push(decl.id.sym.to_string());
      decl.visit_children_with(self);
    }

    fn visit_ts_type_alias_decl(&mut self, decl: &TsTypeAliasDecl) {
      self.declared.push(decl.id.sym.to_string());
      decl.type_ann.visit_children_with(self);
    }

    fn visit_ts_enum_decl(&mut self, decl: &TsEnumDecl) {
      self.declared.push(decl.id.sym.to_string());
      decl.visit_children_with(self);
    }
  }

  let mut collector = RefCollector::default();
  item.visit_with(&mut collector);
  collector.refs.remove(own_name);
  collector.refs
}

pub fn render_module_item(item: &ModuleItem) -> Result<String> {
  let cm: Lrc<SourceMap> = Default::default();
  let mut buf = vec![];
  {
    let wr = JsWriter::new(cm.clone(), "\n", &mut buf, None);
    let mut emitter = Emitter {
      cfg: Config::default(),
      comments: None,
      cm,
      wr,
    };
    let module = swc_core::ecma::ast::Module {
      span: DUMMY_SP,
      body: vec![item.clone()],
      shebang: None,
    };
    emitter
      .emit_module(&module)
      .map_err(|err| error!("Failed to render dts module item: {err:?}"))?;
  }
  String::from_utf8(buf).map_err(|err| error!("Invalid utf8 while rendering dts item: {err}"))
}

pub fn strip_export_from_item(item: &ModuleItem) -> ModuleItem {
  match item {
    ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export_decl)) => {
      ModuleItem::Stmt(Stmt::Decl(export_decl.decl.clone()))
    }
    _ => item.clone(),
  }
}

pub fn apply_name_mapping(item: &mut ModuleItem, mapping: &FxHashMap<String, String>) {
  if mapping.is_empty() {
    return;
  }

  struct RenameVisitor<'a> {
    mapping: &'a FxHashMap<String, String>,
  }

  impl VisitMut for RenameVisitor<'_> {
    fn visit_mut_ident(&mut self, ident: &mut Ident) {
      if let Some(next) = self.mapping.get(ident.sym.as_ref()) {
        ident.sym = next.clone().into();
      }
    }
  }

  item.visit_mut_with(&mut RenameVisitor { mapping });
}
