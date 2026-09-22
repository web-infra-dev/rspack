#[cfg(allocative)]
use rspack_util::allocative;

pub mod commonjs_external;
pub mod dyn_import;

use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  DependencyCodeGeneration, DependencyTemplate, DependencyTemplateType, Module, TemplateContext,
  TemplateReplaceSource,
};

/// Keep generated CommonJS external bindings outside issuers using Rspack's identifier
/// namespace. Persist this parser result with the module, including nested scopes.
#[cacheable]
#[derive(Debug)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub(crate) struct ExternalBindingBailout;

impl ExternalBindingBailout {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("CommonJsExternalBindingBailout")
  }

  pub fn is_present(module: &dyn Module) -> bool {
    module
      .get_presentational_dependencies()
      .is_some_and(|deps| deps.iter().any(|dep| dep.as_any().is::<Self>()))
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for ExternalBindingBailout {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(Self::template_type())
  }
}

impl DependencyTemplate for ExternalBindingBailout {
  fn render(
    &self,
    _dependency: &dyn DependencyCodeGeneration,
    _source: &mut TemplateReplaceSource,
    _context: &mut TemplateContext,
  ) {
    // Parser metadata only; the original dependency templates render the issuer.
  }
}
