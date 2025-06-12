use rspack_core::{BoxPlugin, Plugin};
use rspack_node_builder_macros::register_plugin;

#[derive(Debug)]
struct P {}

impl Plugin for P {
  fn name(&self) -> &'static str {
    "unknown"
  }

  fn apply(
    &self,
    _ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
    _options: &rspack_core::CompilerOptions,
  ) -> rspack_error::Result<()> {
    Ok(())
  }

  fn clear_cache(&self, _id: rspack_core::CompilationId) {}
}

fn test() {
  register_plugin!("haha", |env, options, can_inherent_from_parent| {
    Ok(Box::new(P {}) as BoxPlugin)
  });
}
