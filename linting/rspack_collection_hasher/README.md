# rspack_collection_hasher

### What it does

Warns when `HashMap`/`HashSet`-like collections in Rspack use Rust's default hasher, and
when `Ustr` or `Identifier` keys do not use identity-hasher-based collections.

### Why is this bad?

Rspack is a toolchain workload. The collision resistance of Rust's default hasher is not needed
for these internal hot paths, while the extra hashing cost is measurable. `Ustr` and Rspack's
`Identifier` already store precomputed hashes, so they should use `IdentityHasher`-based
collections instead of generic hashers such as `FxHasher`.

### Known problems

This lint intentionally focuses on the collection families used in the Rspack workspace:
`HashMap`/`HashSet`, `IndexMap`/`IndexSet`, `DashMap`/`DashSet`, and
`LinkedHashMap`/`LinkedHashSet`.

### Example

```rust
use std::collections::HashMap;

let map = HashMap::<String, usize>::new();
```

Use instead:

```rust
use rspack_util::fx_hash::FxHashMap;

let map = FxHashMap::<String, usize>::default();
```
