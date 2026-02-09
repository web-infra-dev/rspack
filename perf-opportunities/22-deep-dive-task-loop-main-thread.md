# Deep Dive: Task Loop Main-Thread Bottleneck

**File**: `crates/rspack_core/src/utils/task_loop.rs` and `crates/rspack_core/src/compilation/build_module_graph/graph_updater/repair/`

---

## The Problem

The module building pipeline uses a task loop with Main and Background task types:

```
[Background] FactorizeTask → FactorizeResultTask [Main] → AddTask [Main] → BuildTask [Background]
→ BuildResultTask [Main] → ProcessDependenciesTask [Main] → FactorizeTask [Background] → ...
```

**Main tasks** (sequential on main thread):
- `FactorizeResultTask`: Records factorize results, creates AddTask
- `AddTask`: Adds module to graph, creates BuildTask (or skips if already exists)
- `BuildResultTask`: Records build results, adds dependencies to graph, creates ProcessDependenciesTask
- `ProcessDependenciesTask`: Groups dependencies, creates FactorizeTask per unique resource

**Background tasks** (parallel on tokio workers):
- `FactorizeTask`: Calls module factory to create module
- `BuildTask`: Calls module.build() (includes SWC parsing/transforms)

### Task Distribution at 10K Modules

For each module:
1. 1 × `FactorizeTask` (background) — ~1-5ms
2. 1 × `FactorizeResultTask` (main) — ~0.01ms
3. 1 × `AddTask` (main) — ~0.01ms
4. 1 × `BuildTask` (background) — ~1-10ms
5. 1 × `BuildResultTask` (main) — ~0.05-0.1ms (adds deps to graph, handles file dependencies)
6. 1 × `ProcessDependenciesTask` (main) — ~0.02ms (groups deps, creates factorize tasks)

**Per module main-thread time**: ~0.1-0.2ms
**Total main-thread time at 10K modules**: ~1-2 seconds

**Per module background time**: ~2-15ms (but parallelized across N cores)
**Total background time at 10K modules**: ~2-15s / N_cores

With 8 cores: background parallelism = ~2.5-19s / 8 = 0.3-2.4s
Main thread: 1-2s (SEQUENTIAL, can't be parallelized)

**The main thread becomes the bottleneck when background work finishes faster than main thread processing can keep up.**

---

## Detailed Main Thread Work Analysis

### `BuildResultTask::main_run` — The Heaviest Main Task

```rust
// crates/rspack_core/src/compilation/build_module_graph/graph_updater/repair/build.rs

async fn main_run(self: Box<Self>, context: &mut TaskContext) -> TaskResult<TaskContext> {
    let BuildResultTask { mut module, build_result, plugin_driver, mut forwarded_ids } = *self;

    // 1. Call succeed_module hook
    plugin_driver.compilation_hooks.succeed_module.call(...).await?;

    // 2. Record diagnostics
    if !module.diagnostics().is_empty() {
        context.artifact.make_failed_module.insert(module.identifier());
    }

    // 3. Record optimization bailouts
    context.artifact.module_graph.get_optimization_bailout_mut(...).extend(...);

    // 4. Record file dependencies (multiple HashSet operations)
    let resource_id = ResourceId::from(module.identifier());
    context.artifact.file_dependencies.add_files(&resource_id, &build_info.file_dependencies);
    context.artifact.context_dependencies.add_files(&resource_id, &build_info.context_dependencies);
    context.artifact.missing_dependencies.add_files(&resource_id, &build_info.missing_dependencies);
    context.artifact.build_dependencies.add_files(&resource_id, &build_info.build_dependencies);

    // 5. Process all dependencies and blocks — THE EXPENSIVE PART
    let mut lazy_dependencies = LazyDependencies::default();
    let mut queue = VecDeque::new();
    let mut all_dependencies = vec![];
    let mut handle_block = |dependencies, blocks, current_block| {
        for dependency in dependencies {
            module_graph.set_parents(dependency_id, DependencyParents { ... });
            module_graph.add_dependency(dependency);  // HashMap insert
        }
        if let Some(current_block) = current_block {
            module.add_block_id(current_block.identifier());
            module_graph.add_block(current_block);  // HashMap insert
        }
        blocks
    };
    let blocks = handle_block(build_result.dependencies, build_result.blocks, None);
    queue.extend(blocks);
    while let Some(mut block) = queue.pop_front() {
        let dependencies = block.take_dependencies();
        let blocks = handle_block(dependencies, block.take_blocks(), Some(block));
        queue.extend(blocks);
    }

    // 6. Set all_dependencies on module graph module
    mgm.all_dependencies = all_dependencies.clone();

    // 7. Add module to module graph
    module_graph.add_module(module);  // HashMap insert

    // 8. Create ProcessDependenciesTask
    Ok(tasks)
}
```

At 10K modules with ~50 dependencies each:
- Step 4: 40K HashSet insertions (file deps)
- Step 5: 500K HashMap insertions (dependencies) + 10K HashMap insertions (blocks)
- Step 6: 500K Vec items cloned
- Step 7: 10K HashMap insertions (modules)

Total: ~950K HashMap/HashSet operations on the main thread = **significant memory allocation and cache pressure**.

### `ProcessDependenciesTask::main_run` — Dependency Grouping

```rust
async fn main_run(self: Box<Self>, context: &mut TaskContext) -> TaskResult<TaskContext> {
    let mut sorted_dependencies = HashMap::default();
    
    for dependency_id in dependencies {
        context.artifact.affected_dependencies.mark_as_add(dependency_id);
        
        let dependency = module_graph.dependency_by_id(&dependency_id);
        let resource_identifier = ...; // Compute resource identifier
        
        sorted_dependencies.entry(resource_identifier)
            .or_insert(vec![])
            .push(dependency.clone());  // Clone BoxDependency!
    }
    
    // Create one FactorizeTask per unique resource
    for dependencies in sorted_dependencies.into_values() {
        res.push(Box::new(FactorizeTask { ... }));
    }
    Ok(res)
}
```

Key issues:
1. `dependency.clone()` — clones trait objects, involves heap allocation
2. `HashMap::default()` — allocates per task
3. Grouping logic runs for every module's dependencies

---

## Optimization Opportunities

### Opportunity 1: Move dependency graph updates to background tasks

The heaviest main-thread work is in `BuildResultTask::main_run` step 5 — adding dependencies to the module graph. If the module graph used a concurrent data structure, this work could stay in the background task.

```rust
// Current: BuildTask (background) → BuildResultTask (main) → adds deps to graph
// Proposed: BuildTask (background) → directly adds deps to concurrent graph
```

**Challenge**: Module graph uses `HashMap` which isn't thread-safe. Would need:
- `DashMap` for dependencies/connections/blocks
- Or lock-free append-only structures
- Or per-module write buffers merged in batches

**Estimated improvement**: 30-50% of main-thread time (eliminates ~500K sequential HashMap inserts)

### Opportunity 2: Batch main-thread processing

Instead of processing one result at a time, batch multiple results:

```rust
// Current:
loop {
    let task = self.main_task_queue.pop_front();
    if let Some(task) = task {
        self.handle_task_result(task.main_run(ctx).await)?;
    }
    // Check for background results
}

// Proposed:
loop {
    // Process up to N main tasks
    for _ in 0..BATCH_SIZE {
        let task = self.main_task_queue.pop_front();
        if let Some(task) = task {
            self.handle_task_result(task.main_run(ctx).await)?;
        }
    }
    // Drain all available background results at once
    while let Ok(result) = self.task_result_receiver.try_recv() {
        self.handle_task_result(result)?;
    }
}
```

This reduces channel contention and improves cache locality.

### Opportunity 3: Reduce dependency cloning

In `ProcessDependenciesTask`, dependencies are cloned for grouping:
```rust
sorted_dependencies.entry(resource_identifier).or_insert(vec![]).push(dependency.clone());
```

Instead, group by dependency ID and only clone when creating the FactorizeTask:
```rust
// Group IDs, not full objects
let mut grouped_ids: HashMap<Cow<str>, Vec<DependencyId>> = HashMap::default();
for dep_id in dependencies {
    let dep = module_graph.dependency_by_id(&dep_id);
    let resource_id = compute_resource_id(dep);
    grouped_ids.entry(resource_id).or_default().push(dep_id);
}
```

### Opportunity 4: Pre-allocate dependency containers

```rust
// BuildResultTask currently uses:
let mut all_dependencies = vec![];  // Grows dynamically

// Could pre-allocate based on build_result hint:
let mut all_dependencies = Vec::with_capacity(
    build_result.dependencies.len() + estimated_block_deps
);
```

---

## Impact Projection

| Optimization | Main Thread Savings | Complexity |
|-------------|-------------------|------------|
| Move dep graph updates to background | 30-50% of main thread | Very High |
| Batch main-thread processing | 10-20% of main thread | Medium |
| Reduce dependency cloning | 5-10% of main thread | Low |
| Pre-allocate containers | 2-5% of main thread | Low |

Combined at 10K modules:
- Current: ~1-2s main-thread time
- After optimizations: ~0.3-0.8s main-thread time
- **Net build time improvement: 5-15% of total make phase**
