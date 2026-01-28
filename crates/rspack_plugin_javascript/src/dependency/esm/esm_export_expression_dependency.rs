use itertools::Itertools;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  AsContextDependency, AsModuleDependency, DEFAULT_EXPORT, Dependency, DependencyCodeGeneration,
  DependencyId, DependencyLocation, DependencyRange, DependencyTemplate, DependencyTemplateType,
  DependencyType, ESMExportInitFragment, ExportNameOrSpec, ExportsInfoGetter, ExportsOfExportsSpec,
  ExportsSpec, ForwardId, GetUsedNameParam, ModuleGraph, ModuleGraphCacheArtifact,
  PrefetchExportsInfoMode, SharedSourceMap, TemplateContext, TemplateReplaceSource, UsedName,
  property_access, rspack_sources::ReplacementEnforce,
};
use swc_core::atoms::Atom;

use crate::parser_plugin::JS_DEFAULT_KEYWORD;

#[cacheable]
#[derive(Debug, Clone)]
pub enum DeclarationId {
  Id(String),
  Func(DeclarationInfo),
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct DeclarationInfo {
  range: DependencyRange,
  prefix: String,
  suffix: String,
}

impl DeclarationInfo {
  pub fn new(range: DependencyRange, prefix: String, suffix: String) -> Self {
    Self {
      range,
      prefix,
      suffix,
    }
  }
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct ESMExportExpressionDependency {
  id: DependencyId,
  range: DependencyRange,
  range_stmt: DependencyRange,
  prefix: String,
  declaration: Option<DeclarationId>,
  loc: Option<DependencyLocation>,
}

impl ESMExportExpressionDependency {
  pub fn new(
    range: DependencyRange,
    range_stmt: DependencyRange,
    prefix: String,
    declaration: Option<DeclarationId>,
    source_map: Option<SharedSourceMap>,
  ) -> Self {
    let loc = range.to_loc(source_map.as_ref());
    Self {
      id: DependencyId::default(),
      range,
      range_stmt,
      declaration,
      prefix,
      loc,
    }
  }
}

#[cacheable_dyn]
impl Dependency for ESMExportExpressionDependency {
  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::EsmExportExpression
  }

  fn id(&self) -> &rspack_core::DependencyId {
    &self.id
  }

  fn loc(&self) -> Option<DependencyLocation> {
    self.loc.clone()
  }

  fn get_exports(
    &self,
    _mg: &ModuleGraph,
    _mg_cache: &ModuleGraphCacheArtifact,
  ) -> Option<ExportsSpec> {
    Some(ExportsSpec {
      exports: ExportsOfExportsSpec::Names(vec![ExportNameOrSpec::String(
        JS_DEFAULT_KEYWORD.clone(),
      )]),
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
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _module_chain: &mut IdentifierSet,
    _connection_state_cache: &mut IdentifierMap<rspack_core::ConnectionState>,
  ) -> rspack_core::ConnectionState {
    rspack_core::ConnectionState::Active(false)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
  }

  fn forward_id(&self) -> ForwardId {
    ForwardId::Id(JS_DEFAULT_KEYWORD.clone())
  }
}

impl AsModuleDependency for ESMExportExpressionDependency {}
impl AsContextDependency for ESMExportExpressionDependency {}

#[cacheable_dyn]
impl DependencyCodeGeneration for ESMExportExpressionDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ESMExportExpressionDependencyTemplate::template_type())
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ESMExportExpressionDependencyTemplate;

impl ESMExportExpressionDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::EsmExportExpression)
  }
}

impl DependencyTemplate for ESMExportExpressionDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ESMExportExpressionDependency>()
      .expect(
        "ESMExportExpressionDependencyTemplate should only be used for ESMExportExpressionDependency",
      );
    let TemplateContext {
      compilation,
      runtime,
      module,
      init_fragments,
      concatenation_scope,
      runtime_template,
      ..
    } = code_generatable_context;

    let mg = compilation.get_module_graph();
    let module_identifier = module.identifier();

    if let Some(declaration) = &dep.declaration {
      let name = match declaration {
        DeclarationId::Id(id) => id,
        DeclarationId::Func(func) => {
          source.replace(
            func.range.start,
            func.range.end,
            &format!("{}{}{}", func.prefix, DEFAULT_EXPORT, func.suffix),
            None,
          );
          DEFAULT_EXPORT
        }
      };

      if let Some(scope) = concatenation_scope {
        scope.register_export(JS_DEFAULT_KEYWORD.clone(), name.to_string());
      } else if let Some(used) = ExportsInfoGetter::get_used_name(
        GetUsedNameParam::WithNames(
          &mg.get_prefetched_exports_info(&module_identifier, PrefetchExportsInfoMode::Default),
        ),
        *runtime,
        std::slice::from_ref(&JS_DEFAULT_KEYWORD),
      ) && let UsedName::Normal(used) = used
      {
        init_fragments.push(Box::new(ESMExportInitFragment::new(
          module.get_exports_argument(),
          vec![(
            used
              .iter()
              .map(|i| i.to_string())
              .collect_vec()
              .join("")
              .into(),
            Atom::from(format!("/* export default binding */ {name}")),
          )],
        )));
      } else {
        // do nothing for unused or inlined
      }

      source.replace(
        dep.range_stmt.start,
        dep.range.start,
        format!("/* export default */ {}", dep.prefix).as_str(),
        None,
      );
    } else {
      // 'var' is a little bit incorrect as TDZ is not correct, but we can't use 'const'
      let supports_const = compilation.options.output.environment.supports_const();
      let content = if let Some(scope) = concatenation_scope {
        scope.register_export(JS_DEFAULT_KEYWORD.clone(), DEFAULT_EXPORT.to_string());
        format!(
          "/* export default */ {} {DEFAULT_EXPORT} = ",
          if supports_const { "const" } else { "var" }
        )
      } else if let Some(used) = ExportsInfoGetter::get_used_name(
        GetUsedNameParam::WithNames(
          &mg.get_prefetched_exports_info(&module_identifier, PrefetchExportsInfoMode::Default),
        ),
        *runtime,
        std::slice::from_ref(&JS_DEFAULT_KEYWORD),
      ) {
        if let UsedName::Normal(used) = used {
          if supports_const {
            init_fragments.push(Box::new(ESMExportInitFragment::new(
              module.get_exports_argument(),
              vec![(
                used
                  .iter()
                  .map(|i| i.to_string())
                  .collect_vec()
                  .join("")
                  .into(),
                DEFAULT_EXPORT.into(),
              )],
            )));
            format!("/* export default */ const {DEFAULT_EXPORT} = ")
          } else {
            format!(
              r#"/* export default */ {}{} = "#,
              runtime_template.render_exports_argument(module.get_exports_argument()),
              property_access(used, 0)
            )
          }
        } else {
          format!("/* inlined export default */ var {DEFAULT_EXPORT} = ")
        }
      } else {
        format!("/* unused export default */ var {DEFAULT_EXPORT} = ")
      };

      source.replace(
        dep.range_stmt.start,
        dep.range.start,
        &format!("{}({}", content, dep.prefix),
        None,
      );
      source.replace_with_enforce(
        dep.range.end,
        dep.range_stmt.end,
        ");",
        None,
        ReplacementEnforce::Post,
      );
    }
  }
}
