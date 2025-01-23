use rspack_hook::define_hook;

define_hook!(RealContentHashPluginUpdateHash: AsyncSeriesBail(assets: &mut Vec<Vec<u8>>, old_hash: &mut String) -> String);

#[derive(Debug, Default)]
pub struct RealContentHashPluginHooks {
  pub update_hash: RealContentHashPluginUpdateHashHook,
}
