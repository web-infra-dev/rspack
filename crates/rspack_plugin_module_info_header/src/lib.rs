use rspack_core::Plugin;

#[derive(Debug)]
struct ModuleInfoHeaderPlugin {}

#[async_trait::async_trait]
impl Plugin for ModuleInfoHeaderPlugin {}
