
#[derive(Debug)]

pub struct ResolveExtPlugin {
    pub extensions: Vec<String>,
}

#[async_trait]
impl Plugin for ResolveExtensionPlugin {
    async fn load(&self, _ctx: &BundleContext, id: &str) -> Option<String> {
        let p = Path::new(id);
        if p.extension().is_none() {
            let mut p = p.to_path_buf();
            for ext in &self.extensions {
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