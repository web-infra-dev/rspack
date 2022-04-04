use std::{collections::HashMap, path::Path};

use crate::plugin::Plugin;
use async_trait::async_trait;
use sugar_path::PathSugar;
use swc_ecma_ast::BindingIdent;
use swc_ecma_visit::VisitMut;

use crate::bundle_context::BundleContext;

#[derive(Debug)]
pub struct DefinePlugin {
    replaced: HashMap<String, String>,
}

#[async_trait]
impl Plugin for DefinePlugin {
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
