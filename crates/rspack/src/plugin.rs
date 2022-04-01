use std::{fmt::Debug, path::Path};

use async_trait::async_trait;
use smol_str::SmolStr;

#[derive(Debug, Clone)]
pub struct ResolvedId {
    pub id: SmolStr,
    pub external: bool,
}

impl ResolvedId {
    pub fn new(id: String, external: bool) -> Self {
        Self {
            id: id.into(),
            external,
        }
    }
}

#[async_trait]
pub trait Plugin: Sync + Send + Debug {
    #[inline]
    async fn resolve(&self, importer: Option<&str>, importee: &str) -> Option<ResolvedId> {
        None
    }
    
    #[inline]
    async fn load(&self, id: &str) -> Option<String> {
        None
    }
}

#[derive(Debug)]
pub struct ResolveExtensionPlugin {
    pub extensions: Vec<String>,
}

#[async_trait]
impl Plugin for ResolveExtensionPlugin {
    async fn load(&self, id: &str) -> Option<String> {
        let p = Path::new(id);
        if p.extension().is_none() {
            let mut p = p.to_path_buf();
            for ext in &self.extensions {
                println!("check {:?}", p);
                p.set_extension(ext);
                let source = tokio::fs::read_to_string(&p).await;
                if let Ok(source) = source {
                    return Some(source);
                }
            }
            None
        } else {
            None
        }
    }
}
