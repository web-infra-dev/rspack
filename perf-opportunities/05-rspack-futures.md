# rspack_futures — Performance Opportunities

**Size**: ~200 lines of Rust across 2 files  
**Role**: Provides `scope()` — an async scoped task spawning primitive used for parallelism in code generation, hashing, chunk asset creation, and emit  
**Impact**: High — this is the parallelism backbone for most seal-phase operations

---

## Table of Contents

1. [Tokio Task Spawn Overhead](#1-tokio-task-spawn-overhead)
2. [JoinHandle Collection Pattern](#2-joinhandle-collection-pattern)
3. [Abort-on-Drop Safety Guard](#3-abort-on-drop-safety-guard)
4. [MaybeUninit Output Buffer](#4-maybeuninit-output-buffer)
5. [No Work-Stealing Across Scopes](#5-no-work-stealing-across-scopes)

---

## 1. Tokio Task Spawn Overhead

**File**: `crates/rspack_futures/src/scope.rs`

Every task spawned via `scope()` creates a new tokio task:

```rust
impl<'scope, 'spawner, T, O> Spawner<'scope, 'spawner, T, O> {
    pub fn spawn<F, Fut>(self, f: F) where F: FnOnce(T) -> Fut, Fut: Future<Output = O> {
        let future = f(self.used);
        let handle = tokio::task::spawn(async move { future.await });
        self.list.borrow_mut().push(handle);
    }
}
```

Each `tokio::task::spawn` involves:
- Allocating task state on the heap
- Scheduling on the tokio runtime
- Creating a `JoinHandle`

At 10K modules, `code_generation_modules` spawns 10K tasks. `create_module_hashes` spawns another 10K. `create_chunk_assets` spawns one per chunk (hundreds). That's 20K+ tokio task spawns per compilation.

**Opportunity**:
1. **Batch small tasks**: Instead of one task per module, batch modules into groups (e.g., 100 modules per task) to reduce spawn overhead. Each batch processes its modules sequentially.
2. **Use rayon for CPU-bound work**: Code generation and hashing are CPU-bound. Rayon's work-stealing threadpool is more efficient than tokio's multi-threaded runtime for CPU work.
3. **Thread-local buffers**: Pre-allocate per-thread buffers to reduce heap allocation within tasks.

**Impact**: Medium. Task spawn overhead is ~1μs per task, but at 20K+ tasks that's ~20ms. The real cost is scheduling overhead and cache misses from task migration between threads.

**Estimated Gain**: 5-15% of parallel phases (code gen, hashing)

---

## 2. JoinHandle Collection Pattern

```rust
pub async fn scope<'scope, F, O>(f: F) -> Vec<Result<O, JoinError>> {
    let list = RefCell::new(Vec::new());
    let token = Token { list: &list, ... };
    f(token);
    let handles = list.into_inner();
    let results = join_all(handles).await;
    results
}
```

All task handles are collected into a `Vec` and then `join_all` is awaited. This means:
- All tasks must complete before any results are processed
- The `Vec<JoinHandle>` grows dynamically as tasks are spawned
- `join_all` creates a `FuturesUnordered` internally, adding overhead

**Opportunity**:
1. **Pre-allocate handle vector**: The number of tasks is often known in advance (e.g., number of modules). Pre-allocate the Vec.
2. **Stream results as they complete**: Use `FuturesUnordered` directly and process results as they arrive, enabling pipelining.
3. **Bounded concurrency**: Instead of spawning all tasks at once, use a semaphore to limit concurrent tasks and reduce memory pressure.

**Impact**: Low-Medium. The main cost is the all-or-nothing completion pattern.

**Estimated Gain**: 2-5% of parallel phases

---

## 3. Abort-on-Drop Safety Guard

```rust
impl Drop for ScopeGuard {
    fn drop(&mut self) {
        std::process::abort();  // avoid unsound caused by poll interruption
    }
}
```

The `ScopeGuard` aborts the process if the scope future is dropped before completion. This is a safety mechanism to prevent use-after-free, but it means:
- The scope can never be cancelled
- If any task panics, the entire process aborts
- No graceful shutdown possible

**Opportunity**: This is a correctness concern more than performance, but the `mem::forget` trick used to disarm the guard has a small overhead.

**Estimated Gain**: Negligible

---

## 4. MaybeUninit Output Buffer

**File**: `crates/rspack_futures/src/lib.rs`

The `par_iter_then_collect` function uses `MaybeUninit` for the output buffer:

```rust
pub async unsafe fn par_iter_then_collect<I, F, O>(iter: I) -> Vec<O> {
    let output: Box<[MaybeUninit<SyncUnsafeCell<O>>]> = Box::new_uninit_slice(iter.len());
    scope(|token| {
        for (i, f) in iter.enumerate() {
            let spawner = unsafe { token.used((f, &output)) };
            spawner.spawn(move |(f, output)| async move {
                let result = f.await;
                let slot = &output[i];
                unsafe { UnsafeCell::raw_get(slot.as_ptr().cast::<UnsafeCell<O>>()).write(result); }
            });
        }
    }).await;
    let output = unsafe { output.assume_init() };
    // ... transmute to Vec<O>
}
```

This is already well-optimized:
- Pre-allocates exact output size
- Uses `MaybeUninit` to avoid initialization cost
- Direct slot writes avoid synchronization
- `SyncUnsafeCell` avoids overhead of `Mutex` or `RwLock`

**Opportunity**: The implementation is already efficient. The main improvement would be in the caller patterns (see item #1).

**Estimated Gain**: Negligible

---

## 5. No Work-Stealing Across Scopes

The `scope` function creates isolated task groups. Tasks from different scopes cannot steal work from each other. If one scope finishes early (e.g., fewer modules in the second code generation batch), those threads sit idle until the scope completes.

**Opportunity**:
1. **Global task pool**: Use a global work-stealing pool (rayon) instead of per-scope tokio tasks
2. **Merge scopes**: When multiple scopes run sequentially (e.g., the two code generation batches), merge them into a single scope with priority scheduling

**Impact**: Low. The load imbalance is typically small within a scope.

**Estimated Gain**: 1-3%

---

## Summary

| Rank | Opportunity | Estimated Gain | Effort |
|------|-----------|----------------|--------|
| 1 | Batch small tasks to reduce spawn overhead | 5-15% of parallel phases | Medium |
| 2 | Use rayon for CPU-bound work instead of tokio | 5-15% | High |
| 3 | Stream results instead of join_all | 2-5% | Medium |
