use std::{collections::HashMap, ffi::OsString, sync::Arc};

use dashmap::DashMap;
use smol_str::SmolStr;

use crate::{plugin::ResolvedId, PluginDriver};

#[derive(Debug)]
pub struct JsModule {
    pub id: SmolStr,
    pub source: String,
    pub resolved_ids: DashMap<SmolStr, ResolvedId>,
}

impl JsModule {
    pub fn new(id: SmolStr) -> Self {
        Self {
            id,
            resolved_ids: Default::default(),
            source: "".to_string(),
        }
    }
    pub(crate) async fn resolve_id(
        &self,
        plugin_driver: &PluginDriver,
        importee: &str,
    ) -> ResolvedId {
        if let Some(resolved_id) = self.resolved_ids.get(importee) {
            resolved_id.clone()
        } else {
            let resolved_id = plugin_driver
                .resolve_id(Some(self.id.as_ref()), &importee)
                .await;
            self.resolved_ids
                .insert(importee.to_string().into(), resolved_id.clone());
            resolved_id
        }
    }
}
