use std::fmt::Debug;

use async_trait::async_trait;

use crate::{bundler::BundleContext, types::ResolvedId};

pub type LoadHookOutput = Option<String>;
pub type ResolveHookOutput = Option<ResolvedId>;

#[async_trait]
pub trait Plugin: Sync + Send + Debug {
    #[inline]
    async fn prepare(&self, _ctx: &BundleContext) {}

    #[inline]
    async fn resolve(
        &self,
        _ctx: &BundleContext,
        _importee: &str,
        _importer: Option<&str>,
    ) -> ResolveHookOutput {
        None
    }

    #[inline]
    async fn load(&self, _ctx: &BundleContext, _id: &str) -> LoadHookOutput {
        None
    }
}

// We could use this to dispatch Plugin trait staticly
#[derive(Debug)]
pub struct PipedPlugin<A: Plugin, B: Plugin> {
    pub left: A,
    pub right: B,
}

impl<A: Plugin, B: Plugin> PipedPlugin<A, B> {
    pub fn new(left: A, right: B) -> Self {
        Self { left, right }
    }
}

#[async_trait]
impl<A: Plugin, B: Plugin> Plugin for PipedPlugin<A, B> {
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
