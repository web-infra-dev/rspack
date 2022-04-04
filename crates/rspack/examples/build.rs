use std::sync::Arc;

use rspack::{
    bundle_context::BundleContext,
    graph_container::{GraphContainer, InputOptions},
    plugin::ResolveExtensionPlugin,
    plugin_driver::PluginDriver,
};

#[tokio::main]
async fn main() {
    let ctx = Arc::new(BundleContext {
        assets: Default::default(),
    });
    let mut g = GraphContainer {
        plugin_driver: Arc::new(PluginDriver {
            ctx,
            plugins: vec![Box::new(ResolveExtensionPlugin {
                extensions: vec!["js".to_string()],
            })],
        }),
        resolved_entries: Default::default(),
        module_by_id: Default::default(),
        input: InputOptions {
            entries: vec![
                "./crates/rspack/fixtures/basic/entry-a.js".to_string(),
                "./crates/rspack/fixtures/basic/entry-b.js".to_string(),
            ],
        },
    };
    g.generate_module_graph().await;
}
