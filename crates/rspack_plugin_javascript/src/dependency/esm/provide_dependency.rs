use itertools::Itertools;
use rspack_core::{module_raw, NormalInitFragment, UsedName};
use rspack_core::{AsContextDependency, Dependency, InitFragmentKey, InitFragmentStage};
use rspack_core::{DependencyCategory, DependencyId, DependencyTemplate};
use rspack_core::{DependencyType, ErrorSpan};
use rspack_core::{ModuleDependency, TemplateContext, TemplateReplaceSource};
use swc_core::atoms::Atom;

#[derive(Debug, Clone)]
pub struct ProvideDependency {
  start: u32,
  end: u32,
  id: DependencyId,
  request: Atom,
  identifier: String,
  ids: Vec<Atom>,
}

impl ProvideDependency {
  pub fn new(start: u32, end: u32, request: Atom, identifier: String, ids: Vec<Atom>) -> Self {
    Self {
      start,
      end,
      request,
      identifier,
      ids,
      id: DependencyId::new(),
    }
  }
}

impl Dependency for ProvideDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::Provided
  }

  fn span(&self) -> Option<ErrorSpan> {
    None
  }

  fn dependency_debug_name(&self) -> &'static str {
    "ProvideDependency"
  }
}

impl ModuleDependency for ProvideDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }
}

impl DependencyTemplate for ProvideDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext {
      compilation,
      runtime,
      runtime_requirements,
      init_fragments,
      ..
    } = code_generatable_context;
    let Some(con) = compilation
      .get_module_graph()
      .connection_by_dependency(&self.id)
    else {
      unreachable!();
    };
    let exports_info = compilation
      .get_module_graph()
      .get_exports_info(&con.module_identifier);
    let used_name = exports_info.id.get_used_name(
      compilation.get_module_graph(),
      *runtime,
      UsedName::Vec(self.ids.clone()),
    );
    init_fragments.push(Box::new(NormalInitFragment::new(
      format!(
        "/* provided dependency */ var {} = {}{};\n",
        self.identifier,
        module_raw(
          compilation,
          runtime_requirements,
          self.id(),
          self.request(),
          self.weak()
        ),
        path_to_string(used_name.as_ref())
      ),
      InitFragmentStage::StageProvides,
      1,
      InitFragmentKey::ExternalModule(format!("provided {}", self.identifier)),
      None,
    )));
    source.replace(self.start, self.end, &self.identifier, None);
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }
}

fn path_to_string(path: Option<&UsedName>) -> String {
  match path {
    Some(p) => match p {
      UsedName::Str(str) => format!("[\"{}\"]", str.as_str()),
      UsedName::Vec(vec) if !vec.is_empty() => vec
        .iter()
        .map(|part| format!("[\"{}\"]", part.as_str()))
        .join(""),
      _ => String::new(),
    },
    None => String::new(),
  }
}

impl AsContextDependency for ProvideDependency {}
