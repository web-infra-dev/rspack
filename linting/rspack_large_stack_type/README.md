# rspack_large_stack_type

### What it does

Denies by-value use of known large Rspack artifact/result types in type positions.
These types should be returned or owned through `Box<T>`, and passed by `&T` / `&mut T`
when ownership is not needed.

### Why is this bad?

Large compilation artifacts such as `BuildModuleGraphArtifact`, `ExportsInfoArtifact`,
`BuildResult`, `ParseResult`, and `CodeGenerationResult` should not be moved through function
or async boundaries by value. Boxing keeps the owning allocation stable and avoids inflating
async state machines with large values.

### Known problems

This lint is intentionally project-specific. Update `LARGE_RSPACK_TYPES` in `src/lib.rs` when
another artifact/result type should be boxed by policy. The lint treats `Box`, references, raw
pointers, `Arc`, `Rc`, collection containers, `BindingCell`, `WeakBindingCell`, `StealCell`,
and `MemoryGCStorage` as allowed boundaries.

### Example

```rust
struct BuildResult;

fn build() -> Result<BuildResult, ()> {
  todo!()
}
```

Use instead:

```rust
struct BuildResult;

fn build() -> Result<Box<BuildResult>, ()> {
  todo!()
}
```
