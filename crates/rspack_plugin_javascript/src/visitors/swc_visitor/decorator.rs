use either::Either;
use rspack_core::DecoratorOptions;
use swc_core::ecma::transforms::base::pass::noop;
use swc_core::ecma::transforms::base::Assumptions;
use swc_core::ecma::transforms::proposal::decorators;
use swc_core::ecma::visit::Fold;

pub fn decorator(assumptions: Assumptions, option: &Option<DecoratorOptions>) -> impl Fold {
  if let Some(option) = option {
    Either::Left(decorators(decorators::Config {
      legacy: option.legacy,
      emit_metadata: option.emit_metadata,
      use_define_for_class_fields: !assumptions.set_public_class_fields,
    }))
  } else {
    Either::Right(noop())
  }
}
