use std::{collections::HashSet, sync::Arc};

use dashmap::DashMap;
use linked_hash_map::LinkedHashMap;
use smol_str::SmolStr;
use swc_atoms::JsWord;
use swc_common::util::take::Take;

use crate::{
    plugin_driver::PluginDriver,
    structs::{DynImportDesc, ResolvedId},
    utils::resolve_id,
};

#[derive(Debug)]
pub struct DependencyIdResolver {
    pub module_id: SmolStr,
    pub resolved_ids: DashMap<JsWord, ResolvedId>,
    pub plugin_driver: Arc<PluginDriver>,
}

impl DependencyIdResolver {
    pub async fn resolve_id(&self, dep_src: &JsWord) -> ResolvedId {
        let resolved_id;
        if let Some(cached) = self.resolved_ids.get(dep_src) {
            resolved_id = cached.clone();
        } else {
            resolved_id =
                resolve_id(dep_src, Some(&self.module_id), false, &self.plugin_driver).await;
            self.resolved_ids
                .insert(dep_src.clone(), resolved_id.clone());
        }
        resolved_id
    }
}

pub struct JsModule {
    pub exec_order: usize,
    pub id: SmolStr,
    pub ast: swc_ecma_ast::Program,
    pub dependencies: LinkedHashMap<JsWord, ()>,
    pub dyn_dependencies: HashSet<DynImportDesc>,
    pub is_user_defined_entry_point: bool,
    pub dependency_resolver: DependencyIdResolver,
}

impl std::fmt::Debug for JsModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsModule")
            .field("exec_order", &self.exec_order)
            .field("id", &self.id)
            // .field("ast", &self.ast)
            .field("dependencies", &self.dependencies)
            .field("dyn_dependencies", &self.dyn_dependencies)
            .field(
                "is_user_defined_entry_point",
                &self.is_user_defined_entry_point,
            )
            .field("dependency_resolver", &self.dependency_resolver)
            .finish()
    }
}

impl JsModule {
    pub fn new(dependency_resolver: DependencyIdResolver) -> Self {
        Self {
            exec_order: Default::default(),
            id: Default::default(),
            ast: swc_ecma_ast::Program::Module(Take::dummy()),
            dependencies: Default::default(),
            dyn_dependencies: Default::default(),
            is_user_defined_entry_point: Default::default(),
            dependency_resolver,
        }
    }

    pub async fn resolve_id(&self, dep_src: &JsWord) -> ResolvedId {
        self.dependency_resolver.resolve_id(dep_src).await
    }

    pub fn resolved_ids(&self) -> &DashMap<JsWord, ResolvedId> {
        &self.dependency_resolver.resolved_ids
    }
}
