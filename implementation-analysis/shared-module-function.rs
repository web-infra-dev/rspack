fn is_consume_shared_descendant(module_graph: &ModuleGraph, module_id: &ModuleIdentifier) -> bool {
  // Quick check: if the module itself has shared metadata or is a shared module type
  if let Some(module) = module_graph.module_by_identifier(module_id) {
    if module.build_meta().shared_key.is_some()
      || module.build_meta().consume_shared_key.is_some()
      || module.module_type() == &ModuleType::ConsumeShared
      || module.module_type() == &ModuleType::ProvideShared
    {
      return true;
    }
  }

  // Check if any issuer (module that imports this one) is a shared module
  // This uses a breadth-first search to find shared modules in the dependency chain
  let mut visited = HashSet::default();
  let mut queue = vec![*module_id];

  while let Some(current_id) = queue.pop() {
    if !visited.insert(current_id) {
      continue;
    }

    for connection in module_graph.get_incoming_connections(&current_id) {
      if let Some(issuer_id) = connection.original_module_identifier {
        if let Some(issuer_module) = module_graph.module_by_identifier(&issuer_id) {
          // If we find a shared module in the chain, this module should get PURE annotations
          if issuer_module.build_meta().shared_key.is_some()
            || issuer_module.build_meta().consume_shared_key.is_some()
            || issuer_module.module_type() == &ModuleType::ConsumeShared
            || issuer_module.module_type() == &ModuleType::ProvideShared
          {
            return true;
          }

          // Continue searching up the chain
          queue.push(issuer_id);
        }
      }
    }
  }

  false
}
