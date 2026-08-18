use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsPreset, AsVec},
};
use rspack_core::{
  AsContextDependency, AsModuleDependency, CachedConstDependency, ConstDependency, Dependency,
  DependencyCategory, DependencyCodeGeneration, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, ExportNameOrSpec, ExportSpec, ExportsInfoArtifact,
  ExportsOfExportsSpec, ExportsSpec, InitFragmentExt, InitFragmentKey, InitFragmentStage, Module,
  ModuleGraph, ModuleGraphCacheArtifact, ModuleInitFragments, NormalInitFragment, TemplateContext,
  TemplateReplaceSource, UsedName, property_access, to_identifier,
};
use rspack_util::json_stringify_str;
use swc_atoms::Atom;

use crate::dependency::commonjs::OBJECT_PROTOTYPE_METHODS;

#[cacheable]
#[derive(Debug, Clone, Copy)]
pub enum ExportsBase {
  Exports,
  ModuleExports,
  This,
  DefinePropertyExports,
  DefinePropertyModuleExports,
  DefinePropertyThis,
}

impl ExportsBase {
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Exports => "exports",
      Self::ModuleExports => "module.exports",
      Self::This => "this",
      Self::DefinePropertyExports => "Object.defineProperty(exports)",
      Self::DefinePropertyModuleExports => "Object.defineProperty(module.exports)",
      Self::DefinePropertyThis => "Object.defineProperty(this)",
    }
  }

  pub const fn is_exports(&self) -> bool {
    matches!(self, Self::Exports | Self::DefinePropertyExports)
  }

  pub const fn is_module_exports(&self) -> bool {
    matches!(
      self,
      Self::ModuleExports | Self::DefinePropertyModuleExports
    )
  }

  pub const fn is_this(&self) -> bool {
    matches!(self, Self::This | Self::DefinePropertyThis)
  }

  pub const fn is_expression(&self) -> bool {
    matches!(self, Self::Exports | Self::ModuleExports | Self::This)
  }

  pub const fn is_define_property(&self) -> bool {
    matches!(
      self,
      Self::DefinePropertyExports | Self::DefinePropertyModuleExports | Self::DefinePropertyThis
    )
  }
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct CommonJsExportsDependency {
  id: DependencyId,
  range: DependencyRange,
  value_range: Option<DependencyRange>,
  prevent_name_inference: bool,
  base: ExportsBase,
  #[cacheable(with=AsVec<AsPreset>)]
  names: Vec<Atom>,
}

impl CommonJsExportsDependency {
  pub fn new(
    range: DependencyRange,
    value_range: Option<DependencyRange>,
    prevent_name_inference: bool,
    base: ExportsBase,
    names: Vec<Atom>,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      range,
      value_range,
      prevent_name_inference,
      base,
      names,
    }
  }

  pub fn base(&self) -> ExportsBase {
    self.base
  }

  pub fn names(&self) -> &[Atom] {
    &self.names
  }
}

fn prevent_name_inference(
  source: &mut TemplateReplaceSource,
  module: &dyn Module,
  dependency: &CommonJsExportsDependency,
) {
  let value_range = dependency
    .value_range
    .expect("concatenated CommonJS export should have a value range");
  // DefinePlugin and similar presentational dependencies replace source after
  // parsing, so an anonymous value may not have been visible in the AST above.
  let has_anonymous_value_replacement = dependency.names.len() == 1
    && module
      .get_presentational_dependencies()
      .is_some_and(|dependencies| {
        dependencies.iter().any(|presentational_dependency| {
          presentational_dependency
            .as_any()
            .downcast_ref::<ConstDependency>()
            .is_some_and(|dependency| {
              dependency.range == value_range
                && (dependency.content.contains("function")
                  || dependency.content.contains("class")
                  || dependency.content.contains("=>"))
            })
        })
      });
  if !dependency.prevent_name_inference && !has_anonymous_value_replacement {
    return;
  }
  // An assignment infers a name for an anonymous function or class from its
  // target. Keep the value behind an identity call so the generated export
  // binding does not become its inferred name. The value is passed as an
  // argument (rather than evaluated inside the helper) to preserve lexical
  // `this` and `arguments` for arrow functions. The annotation is required to
  // prevent the minimizer from inlining the helper and reintroducing name
  // inference.
  source.insert_static(
    value_range.start,
    "/*#__NOINLINE__*/ (function(value) { return value; })(",
    None,
  );
  source.insert_static(value_range.end, ")", None);
}

pub(super) fn get_concatenated_export_access(
  module: &dyn Module,
  concatenation_scope: &mut rspack_core::ConcatenationScope,
  init_fragments: &mut ModuleInitFragments<'_>,
  names: &[Atom],
  property_access_suffix: String,
) -> String {
  let name = names.first().expect("should have a CommonJS export name");
  let symbol = concatenation_scope
    .current_module
    .export_map
    .as_ref()
    .and_then(|export_map| export_map.get(name))
    .cloned()
    .unwrap_or_else(|| {
      let identifier = to_identifier(name);
      let base = if identifier == name.as_str() {
        format!("__RSPACK_CJS_EXPORT_{name}__")
      } else {
        format!(
          "__RSPACK_CJS_EXPORT_{}_{}__",
          identifier,
          hex::encode(name.as_bytes())
        )
      };
      let symbol = get_concatenated_name(module, concatenation_scope, &base);
      concatenation_scope.register_export(name.clone(), symbol.clone());
      symbol
    });

  init_fragments.push(
    NormalInitFragment::new(
      format!("var {symbol};\n"),
      InitFragmentStage::StageConstants,
      0,
      InitFragmentKey::CommonJsExports(symbol.clone()),
      None,
    )
    .boxed(),
  );
  format!("{symbol}{property_access_suffix}")
}

fn get_concatenated_name(
  module: &dyn Module,
  concatenation_scope: &mut rspack_core::ConcatenationScope,
  base: &str,
) -> String {
  if concatenation_scope.is_faster_module_concatenation() {
    concatenation_scope
      .ensure_generated_top_level_symbol(base)
      .to_string()
  } else {
    get_unique_concatenated_name(module, concatenation_scope, base)
  }
}

fn get_unique_concatenated_name(
  module: &dyn Module,
  concatenation_scope: &rspack_core::ConcatenationScope,
  base: &str,
) -> String {
  let is_used = |candidate: &str| {
    module
      .build_info()
      .top_level_declarations
      .as_ref()
      .is_some_and(|declarations| {
        declarations
          .iter()
          .any(|declaration| declaration.as_str() == candidate)
      })
      || module
        .build_info()
        .unresolved_identifier_names
        .iter()
        .any(|name| name.as_str() == candidate)
      || module
        .get_presentational_dependencies()
        .is_some_and(|dependencies| {
          dependencies.iter().any(|dependency| {
            dependency
              .as_any()
              .downcast_ref::<ConstDependency>()
              .is_some_and(|dependency| dependency.content.contains(candidate))
              || dependency
                .as_any()
                .downcast_ref::<CachedConstDependency>()
                .is_some_and(|dependency| {
                  dependency.identifier.contains(candidate)
                    || dependency.content.contains(candidate)
                })
          })
        })
      || concatenation_scope
        .current_module
        .export_map
        .as_ref()
        .is_some_and(|export_map| export_map.values().any(|name| name == candidate))
  };

  if !is_used(base) {
    return base.to_string();
  }

  for index in 0.. {
    let candidate = format!("{base}_{index}");
    if !is_used(&candidate) {
      return candidate;
    }
  }

  unreachable!("a unique CommonJS export name should always be available")
}

#[cacheable_dyn]
impl Dependency for CommonJsExportsDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CjsExports
  }

  fn get_exports(
    &self,
    _mg: &ModuleGraph,
    _mg_cache: &ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
  ) -> Option<ExportsSpec> {
    let name = self.names[0].clone();
    if self.base.is_expression() && name.as_str() == "__proto__" {
      return None;
    }
    let vec = vec![ExportNameOrSpec::ExportSpec(ExportSpec {
      // We can't mangle names that are in an empty object because one could access the prototype property
      // when export isn't set yet. It's different for different targets. so here we only list common properties.
      // Check out test case `configCases/mangle/mangle-with-object-prop`
      can_mangle: Some(!OBJECT_PROTOTYPE_METHODS.contains(&name.as_str())),
      name,
      ..Default::default()
    })];
    Some(ExportsSpec {
      exports: ExportsOfExportsSpec::Names(vec),
      ..Default::default()
    })
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
  }
}

impl AsModuleDependency for CommonJsExportsDependency {}

#[cacheable_dyn]
impl DependencyCodeGeneration for CommonJsExportsDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(CommonJsExportsDependencyTemplate::template_type())
  }
}

impl AsContextDependency for CommonJsExportsDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct CommonJsExportsDependencyTemplate;

impl CommonJsExportsDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::CjsExports)
  }
}

impl DependencyTemplate for CommonJsExportsDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<CommonJsExportsDependency>()
      .expect(
        "CommonJsExportsDependencyTemplate should only be used for CommonJsExportsDependency",
      );

    let TemplateContext {
      compilation,
      module,
      runtime,
      init_fragments,
      runtime_template,
      ..
    } = code_generatable_context;

    let module_graph = compilation.get_module_graph();
    let module = module_graph
      .module_by_identifier(&module.identifier())
      .expect("should have mgm");

    if dep.base.is_expression()
      && dep
        .names
        .first()
        .is_some_and(|name| name.as_str() == "__proto__")
    {
      debug_assert!(
        source.concatenation_scope().is_none(),
        "CommonJS __proto__ assignment should prevent concatenation"
      );
      return;
    }

    let exports_info = compilation
      .exports_info_artifact
      .get_exports_info_data(&module.identifier());
    let used = exports_info.get_used_name(&compilation.exports_info_artifact, *runtime, &dep.names);

    if source.concatenation_scope().is_some() {
      debug_assert!(
        matches!(dep.base, ExportsBase::Exports | ExportsBase::ModuleExports),
        "unsupported CommonJS exports base in a concatenated module"
      );
      if let Some(UsedName::Normal(_)) = used {
        let replacement = get_concatenated_export_access(
          module.as_ref(),
          source
            .concatenation_scope()
            .expect("concatenated CommonJS export should have a concatenation scope"),
          init_fragments,
          &dep.names,
          property_access(dep.names[1..].iter(), 0),
        );
        source.replace(dep.range.start, dep.range.end, replacement, None);
      } else {
        let placeholder_var = get_concatenated_name(
          module.as_ref(),
          source
            .concatenation_scope()
            .expect("concatenated CommonJS export should have a concatenation scope"),
          "__rspack_unused_export",
        );
        source.replace(
          dep.range.start,
          dep.range.end,
          placeholder_var.clone(),
          None,
        );
        init_fragments.push(
          NormalInitFragment::new(
            format!("var {placeholder_var};\n"),
            InitFragmentStage::StageConstants,
            0,
            InitFragmentKey::CommonJsExports(placeholder_var),
            None,
          )
          .boxed(),
        );
      }
      prevent_name_inference(source, module.as_ref(), dep);
      return;
    }

    let exports_argument = module.get_exports_argument();
    let module_argument = module.get_module_argument();

    let base = if dep.base.is_exports() {
      runtime_template.render_exports_argument(exports_argument)
    } else if dep.base.is_module_exports() {
      format!(
        "{}.exports",
        runtime_template.render_module_argument(module_argument)
      )
    } else if dep.base.is_this() {
      runtime_template.render_this_exports()
    } else {
      panic!("Unexpected base type");
    };

    if dep.base.is_expression() {
      if let Some(UsedName::Normal(used)) = used {
        source.replace(
          dep.range.start,
          dep.range.end,
          format!("{}{}", base, property_access(used, 0)),
          None,
        );
      } else {
        // Export a inlinable const from cjs is not possible for now but we compat it here
        let is_inlined = matches!(used, Some(UsedName::Inlined(_)));
        let placeholder_var = format!(
          "__rspack_{}_export",
          if is_inlined { "inlined" } else { "unused" }
        );
        source.replace(
          dep.range.start,
          dep.range.end,
          placeholder_var.clone(),
          None,
        );
        init_fragments.push(
          NormalInitFragment::new(
            format!("var {placeholder_var};\n"),
            InitFragmentStage::StageConstants,
            0,
            InitFragmentKey::CommonJsExports(placeholder_var),
            None,
          )
          .boxed(),
        );
      }
    } else if dep.base.is_define_property() {
      if let Some(value_range) = &dep.value_range {
        if let Some(UsedName::Normal(used)) = used {
          if !used.is_empty() {
            source.replace(
              dep.range.start,
              value_range.start,
              format!(
                "Object.defineProperty({}{}, {}, (",
                base,
                property_access(used[0..used.len() - 1].iter(), 0),
                json_stringify_str(used.last().expect("Unexpected render define property base"))
              ),
              None,
            );
            source.replace_static(value_range.end, dep.range.end, "))", None);
          } else {
            panic!("Unexpected base type");
          }
        } else {
          init_fragments.push(
            NormalInitFragment::new(
              "var __rspack_unused_export;\n".to_string(),
              InitFragmentStage::StageConstants,
              0,
              InitFragmentKey::CommonJsExports("__rspack_unused_export".to_owned()),
              None,
            )
            .boxed(),
          );
          source.replace_static(
            dep.range.start,
            value_range.start,
            "__rspack_unused_export = (",
            None,
          );
          source.replace_static(value_range.end, dep.range.end, ")", None);
        }
      } else {
        panic!("Define property need value range");
      }
    } else {
      panic!("Unexpected base type");
    }
  }
}
