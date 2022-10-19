use swc_ecma_transforms::proposals::decorators;
use swc_ecma_visit::Fold;

pub fn decorator() -> impl Fold {
  decorators(decorators::Config {
    legacy: false,
    emit_metadata: false,
    use_define_for_class_fields: true,
  })
}
