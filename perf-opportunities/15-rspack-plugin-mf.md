# rspack_plugin_mf — Performance Opportunities

**Size**: 7,630 lines of Rust across 45 files  
**Role**: Module Federation — enables runtime sharing of code between independently built applications  
**Impact**: Low for standard builds (not active unless configured), Medium-High when Module Federation is used

---

## Table of Contents

1. [Container Module Factory](#1-container-module-factory)
2. [Shared Module Resolution](#2-shared-module-resolution)
3. [Runtime Module Generation](#3-runtime-module-generation)
4. [Manifest Generation](#4-manifest-generation)

---

## 1. Container Module Factory

Module Federation creates container entry modules that expose specific modules. Each exposed module creates a dependency chain:
- `ContainerEntryDependency` → `ContainerEntryModule`
- `ContainerExposedDependency` for each exposed module
- `RemoteModule` for each consumed remote

**Opportunity**: For large federations with many exposed modules, batch the dependency creation instead of creating them one-by-one.

**Impact**: Low unless federation configuration is very large (100+ exposed modules).

**Estimated Gain**: 1-5% of federation overhead

---

## 2. Shared Module Resolution

The `ConsumeSharedPlugin` checks every module against shared configuration:

```rust
// For each module, check if it matches any shared configuration
// This involves string matching against shared module names
```

With many shared modules and 10K total modules, this creates N×M comparisons.

**Opportunity**:
1. **Pre-compute shared module set**: Build a HashSet of shared module names for O(1) lookup instead of O(n) iteration
2. **Skip non-shared modules early**: Most modules are not shared; add a quick rejection check

**Impact**: Low for typical configurations with few shared modules.

**Estimated Gain**: 1-3% of federation overhead

---

## 3. Runtime Module Generation

Module Federation generates several runtime modules:
- `RemoteRuntimeModule` — remote loading logic
- `ConsumeSharedRuntimeModule` — shared module consumption
- `ShareRuntimeModule` — shared module provision
- `FederationDataRuntimeModule` — federation metadata
- Various EJS templates for loading logic

These runtime modules use EJS templates that are rendered during code generation.

**Opportunity**: Same as the general runtime plugin — pre-render static portions, cache results.

**Impact**: Low. Runtime modules are few per compilation.

**Estimated Gain**: <1%

---

## 4. Manifest Generation

When `ManifestPlugin` is configured, the plugin generates a JSON manifest describing the federation configuration. This involves serializing module information.

**Opportunity**: Use streaming JSON serialization instead of building the entire manifest in memory.

**Impact**: Negligible unless the manifest is very large.

**Estimated Gain**: <1%

---

## Summary

Module Federation overhead is primarily relevant when the feature is actively used. For the react-10k benchmark without federation, this crate has zero impact.

| Rank | Opportunity | Estimated Gain | Effort |
|------|-----------|----------------|--------|
| 1 | Pre-compute shared module lookup set | 1-3% of federation overhead | Low |
| 2 | Batch container dependency creation | 1-5% of federation overhead | Low |
