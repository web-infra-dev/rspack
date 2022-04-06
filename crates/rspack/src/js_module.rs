use std::{collections::HashSet, fmt::Debug};

use dashmap::DashMap;
use linked_hash_map::LinkedHashMap;
use smol_str::SmolStr;
use sugar_path::PathSugar;
use swc_atoms::JsWord;

use crate::{plugin::ResolvedId, PluginDriver};

pub struct JsModule {
    pub exec_order: usize,
    pub id: SmolStr,
    pub source: String,
    pub resolved_ids: DashMap<SmolStr, ResolvedId>,
    pub dependencies: LinkedHashMap<JsWord, ()>,
    pub dynamic_dependencies: HashSet<DynImportDesc>,
}

impl Debug for JsModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cwd = std::env::current_dir().unwrap();
        f.debug_struct("JsModule")
            .field("id", &cwd.relative(self.id.as_str()))
            .field("source", &self.source)
            .field("resolved_ids", &self.resolved_ids)
            .finish()
    }
}

impl JsModule {
    pub fn new(id: SmolStr) -> Self {
        Self {
            id,
            resolved_ids: Default::default(),
            source: "".to_string(),
            dependencies: Default::default(),
            dynamic_dependencies: Default::default(),
            exec_order: Default::default(),
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RelationInfo {
    pub source: JsWord,
    // Empty HashSet represents `import './side-effect'` or `import {} from './foo'`
    pub names: HashSet<Specifier>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Specifier {
    /// The original defined name
    pub original: JsWord,
    /// The name importer used
    pub used: JsWord,
    // pub mark: Mark,
}

// import('./foo')
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct DynImportDesc {
    pub argument: JsWord,
    // pub id: Option<JsWord>,
}
