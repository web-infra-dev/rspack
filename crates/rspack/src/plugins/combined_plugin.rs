use async_trait::async_trait;

use crate::{bundler::BundleContext, traits::plugin::Plugin, types::ResolvedId};

// We could use this to dispatch Plugin trait staticly
#[derive(Debug)]
pub struct CombinedPlugin<A: Plugin, B: Plugin> {
    pub left: A,
    pub right: B,
}

impl<A: Plugin, B: Plugin> CombinedPlugin<A, B> {
    pub fn new(left: A, right: B) -> Self {
        Self { left, right }
    }
}

#[async_trait]
impl<A: Plugin, B: Plugin> Plugin for CombinedPlugin<A, B> {
    async fn prepare(&self, ctx: &BundleContext) {
        tokio::join!(self.left.prepare(ctx), self.right.prepare(ctx));
    }

    async fn load(&self, ctx: &BundleContext, id: &str) -> Option<String> {
        let left_res = self.left.load(ctx, id).await;
        if left_res.is_some() {
            left_res
        } else {
            self.right.load(ctx, id).await
        }
    }

    async fn resolve(
        &self,
        ctx: &BundleContext,
        importee: &str,
        importer: Option<&str>,
    ) -> Option<ResolvedId> {
        let left_res = self.left.resolve(ctx, importee, importer).await;
        if left_res.is_some() {
            left_res
        } else {
            self.right.resolve(ctx, importee, importer).await
        }
    }
}
