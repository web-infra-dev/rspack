use std::path::Path;

use async_trait::async_trait;
use sugar_path::PathSugar;

use crate::{bundle_context::BundleContext, plugin::Plugin};

#[derive(Debug)]
pub struct FileLoaderPlugin {}

#[async_trait]
impl Plugin for FileLoaderPlugin {
    async fn load(&self, ctx: &BundleContext, id: &str) -> Option<String> {
        // ctx.emit_asset(crate::bundle_context::Asset { source:
        //   tokioLL, filename: () }
        // );
        Some(format!(
            "export default '{}'",
            Path::new(id)
                .relative(&std::env::current_dir().unwrap())
                .to_string_lossy()
        ))
    }
}
