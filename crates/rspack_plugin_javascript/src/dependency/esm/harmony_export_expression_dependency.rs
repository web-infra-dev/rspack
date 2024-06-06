use itertools::Itertools;
use rspack_core::tree_shaking::symbol::DEFAULT_JS_WORD;
use rspack_core::{
  property_access, AsContextDependency, AsModuleDependency, Compilation, Dependency,
  DependencyLocation, DependencyType, ExportNameOrSpec, ExportsOfExportsSpec, ExportsSpec,
  HarmonyExportInitFragment, ModuleGraph, RuntimeGlobals, RuntimeSpec, UsedName, DEFAULT_EXPORT,
};
use rspack_core::{DependencyId, DependencyTemplate};
use rspack_core::{TemplateContext, TemplateReplaceSource};
use rspack_identifier::Identifier;
use swc_core::atoms::Atom;

#[derive(Debug, Clone)]
pub enum DeclarationId {
  Id(String),
  Func(DeclarationInfo),
}

#[derive(Debug, Clone)]
pub struct DeclarationInfo {
  pub range: DependencyLocation,
  pub prefix: String,
  pub suffix: String,
}

#[derive(Debug, Clone)]
pub struct HarmonyExportExpressionDependency {
  pub range: DependencyLocation,
  pub range_stmt: DependencyLocation,
  pub declaration: Option<DeclarationId>,
  pub id: DependencyId,
}

impl HarmonyExportExpressionDependency {
  pub fn new(
    range: DependencyLocation,
    range_stmt: DependencyLocation,
    declaration: Option<DeclarationId>,
  ) -> Self {
    Self {
      range,
      range_stmt,
      declaration,
      id: DependencyId::default(),
    }
  }
}

impl Dependency for HarmonyExportExpressionDependency {
  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::EsmExportExpression
  }

  fn id(&self) -> &rspack_core::DependencyId {
    &self.id
  }

  fn get_exports(&self, _mg: &ModuleGraph) -> Option<ExportsSpec> {
    Some(ExportsSpec {
      exports: ExportsOfExportsSpec::Array(vec![ExportNameOrSpec::String(DEFAULT_JS_WORD.clone())]),
      priority: Some(1),
      can_mangle: None,
      terminal_binding: Some(true),
      from: None,
      dependencies: None,
      hide_export: None,
      exclude_exports: None,
    })
  }
  fn get_module_evaluation_side_effects_state(
    &self,
    _module_graph: &rspack_core::ModuleGraph,
    _module_chain: &mut rustc_hash::FxHashSet<rspack_core::ModuleIdentifier>,
  ) -> rspack_core::ConnectionState {
    rspack_core::ConnectionState::Bool(false)
  }
}

impl AsModuleDependency for HarmonyExportExpressionDependency {}

impl DependencyTemplate for HarmonyExportExpressionDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext {
      compilation,
      runtime,
      runtime_requirements,
      module,
      init_fragments,
      concatenation_scope,
      ..
    } = code_generatable_context;

    fn get_used_name(
      name: &str,
      compilation: &Compilation,
      runtime: &Option<&RuntimeSpec>,
      module_identifier: &Identifier,
    ) -> Option<UsedName> {
      let module_graph = compilation.get_module_graph();
      module_graph
        .get_exports_info(module_identifier)
        .id
        .get_used_name(&module_graph, *runtime, UsedName::Str(name.into()))
    }

    if let Some(declaration) = &self.declaration {
      let name = match declaration {
        DeclarationId::Id(id) => id,
        DeclarationId::Func(func) => {
          source.replace(
            func.range.start(),
            func.range.end(),
            &format!("{}{}{}", func.prefix, DEFAULT_EXPORT, func.suffix),
            None,
          );
          DEFAULT_EXPORT
        }
      };

      // skip this lint, will make it easy to align with webpack in the future
      #[allow(clippy::collapsible_else_if)]
      if let Some(scope) = concatenation_scope {
        scope.register_export(DEFAULT_JS_WORD.clone(), name.to_string());
      } else {
        if let Some(used) = get_used_name(
          DEFAULT_JS_WORD.as_str(),
          compilation,
          runtime,
          &module.identifier(),
        ) {
          init_fragments.push(Box::new(HarmonyExportInitFragment::new(
            module.get_exports_argument(),
            vec![(
              match used {
                UsedName::Str(s) => s,
                UsedName::Vec(v) => v
                  .iter()
                  .map(|i| i.to_string())
                  .collect_vec()
                  .join("")
                  .into(),
              },
              Atom::from(format!("/* export default binding */ {name}")),
            )],
          )));
        }
      }

      source.replace(
        self.range_stmt.start(),
        self.range.start(),
        "/* harmony default export */ ",
        None,
      );
    } else {
      let content = if let Some(ref mut scope) = concatenation_scope {
        scope.register_export(DEFAULT_JS_WORD.clone(), DEFAULT_EXPORT.to_string());
        // TODO: support const inspect
        format!("/* harmony default export */ var {DEFAULT_EXPORT} = ")
      } else if let Some(used) = get_used_name(
        DEFAULT_JS_WORD.as_str(),
        compilation,
        runtime,
        &module.identifier(),
      ) {
        runtime_requirements.insert(RuntimeGlobals::EXPORTS);
        format!(
          r#"/* harmony default export */ {}{} = "#,
          module.get_exports_argument(),
          property_access(
            match used {
              UsedName::Str(name) => vec![name].into_iter(),
              UsedName::Vec(names) => names.into_iter(),
            },
            0
          )
        )
      } else {
        format!("/* unused harmony default export */ var {DEFAULT_EXPORT} = ")
      };

      source.replace(
        self.range_stmt.start(),
        self.range.start(),
        &format!("{}(", content),
        None,
      );
      source.replace(self.range.end(), self.range_stmt.end(), ");", None);
    }
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }
}

impl AsContextDependency for HarmonyExportExpressionDependency {}
