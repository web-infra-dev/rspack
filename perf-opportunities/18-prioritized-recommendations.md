# Prioritized Recommendations — All Performance Opportunities

This document consolidates all findings from the per-crate analyses and ranks them by estimated impact and implementation effort.

---

## Scoring Methodology

- **Impact**: Estimated percentage improvement to total build time for a react-10k (10,000 React components) cold build
- **Effort**: Low (days), Medium (1-2 weeks), High (weeks-months)
- **Phase**: Which compilation phase is affected

---

## Tier 1 — High Impact, Worth Immediate Investigation

> **Updated with profiling data** — SideEffects O(n²) confirmed as #1 bottleneck.

| # | Opportunity | Crate | Phase | Est. Impact | Effort | Details |
|---|-----------|-------|-------|-------------|--------|---------|
| **★** | **Fix SideEffectsFlagPlugin O(n²) scaling** | rspack_plugin_javascript | OptimizeDeps | **10-50% of optimize deps** | Medium | **PROFILING-CONFIRMED**: Fresh per-task caches cause O(n²) traversal. 292ms at 1K modules, projected 29s at 10K. Use shared cache or topological ordering. See `20-deep-dive-sideeffects-bottleneck.md`, `23-deep-dive-exports-info-system.md` |
| 1 | **Replace BigUint with FixedBitSet in CodeSplitter** | rspack_core | BuildChunkGraph | 3-8% total | Medium | BigUint operations for available module tracking are O(n/64) per operation. FixedBitSet with SIMD would be O(n/256) or better. See `21-deep-dive-codesplitter-biguint.md` |
| 2 | **Reduce main-thread bottleneck in task loop** | rspack_core | BuildModuleGraph | 5-12% total | High | 950K sequential HashMap ops at 10K modules. See `22-deep-dive-task-loop-main-thread.md` |
| 3 | **Merge SWC AST 7-pass pipeline** | rspack_plugin_javascript | BuildModuleGraph | 3-6% total | Medium | 7 passes over AST = 140M node visits at 10K. Merge SWC transforms + pre-walks. See `24-deep-dive-parsing-scanning.md` |
| 4 | **Skip JS↔Rust boundary for hooks with no JS taps** | rspack_binding_api | All phases | 2-10% total | Medium | When no JavaScript plugins are listening, skip the boundary crossing entirely. Most Rspack configurations use only builtin plugins. See `12-rspack-binding-api.md` §1 |
| 5 | **Eliminate ExportsInfoData clone in FlagDependencyExports/Usage** | rspack_plugin_javascript | OptimizeDeps | 2-5% total | High | Both plugins clone ExportsInfoData for parallel processing. ~20MB cloned at 10K modules. See `23-deep-dive-exports-info-system.md` |

---

## Tier 2 — Medium Impact, Good ROI

| # | Opportunity | Crate | Phase | Est. Impact | Effort | Details |
|---|-----------|-------|-------|-------------|--------|---------|
| 6 | **Avoid `.modules()` collecting into new HashMap** | rspack_core | Multiple | 1-3% total | Low | Return iterators instead of collecting into new IdentifierMap. Called dozens of times per compilation. See `01-rspack-core.md` §2, §12 |
| 7 | **Batch tokio task spawns in rspack_futures::scope** | rspack_futures | CodeGen, Hashing | 1-3% total | Medium | Batch 50-100 modules per task instead of one task per module. Reduces spawn overhead from 20K+ to 200+. See `05-rspack-futures.md` §1 |
| 8 | **Pipeline independent compilation passes** | rspack_core | Seal | 1-3% total | High | ModuleIdsPass and ChunkIdsPass are independent and could run concurrently. See `01-rspack-core.md` §1 |
| 9 | **Pre-compute plugin interest masks for parser dispatch** | rspack_plugin_javascript | BuildModuleGraph | 1-3% total | Medium | Skip plugin dispatch for AST node types no plugin is interested in. Saves ~350M unnecessary virtual dispatch calls. See `02-rspack-plugin-javascript.md` §9 |
| 10 | **Optimize module concatenation analysis** | rspack_plugin_javascript | OptimizeChunkModules | 1-2% total | Medium | Pre-compute ESM connected components, improve failure propagation. See `02-rspack-plugin-javascript.md` §6 |
| 11 | **Optimize hashing pipeline** | rspack_core | CreateHash | 1-3% total | High | Merge module hashing into code generation, reduce sequential runtime chunk hashing. See `01-rspack-core.md` §4 |
| 12 | **Priority queue for split chunks module group selection** | rspack_plugin_split_chunks | OptimizeChunks | 0.5-1% total | Medium | Replace O(n²) greedy loop with O(n log n) priority queue. See `03-rspack-plugin-split-chunks.md` §2 |

---

## Tier 3 — Lower Impact but Low Effort (Quick Wins)

| # | Opportunity | Crate | Phase | Est. Impact | Effort | Details |
|---|-----------|-------|-------|-------------|--------|---------|
| 13 | **Cache module-to-runtimes mapping** | rspack_core | Multiple | 0.5-1% total | Low | Compute once, reuse across CreateModuleHashes, CodeGeneration, RuntimeRequirements. See `01-rspack-core.md` §15 |
| 14 | **Unbox Xxhash64 in RspackHash enum** | rspack_hash | All hashing | <0.5% total | Low | Embed 48-byte Xxh64 state directly instead of boxing. See `04-rspack-hash.md` §4 |
| 15 | **Pre-compute salted hash seed** | rspack_hash | All hashing | <0.5% total | Low | Clone pre-salted hasher instead of re-hashing salt each time. See `04-rspack-hash.md` §5 |
| 16 | **Fast path for single builtin loader** | rspack_loader_runner | BuildModuleGraph | 0.5-1% total | Medium | Skip full loader framework for the common case. See `07-rspack-loader-runner.md` §1 |
| 17 | **Cache normalized glob patterns in SideEffectsPlugin** | rspack_plugin_javascript | OptimizeDeps | <0.5% total | Low | Avoid re-normalizing identical patterns for each module. See `02-rspack-plugin-javascript.md` §5 |
| 18 | **Early bail for empty hooks** | rspack_hook | All | 0.5-1% total | Low | Skip hook machinery when no plugins are tapped. See `17-remaining-crates.md` |

---

## Tier 4 — Rebuild/Watch Mode Specific

| # | Opportunity | Crate | Phase | Est. Impact | Effort | Details |
|---|-----------|-------|-------|-------------|--------|---------|
| 19 | **Content-addressed persistent cache** | rspack_storage | Cache | 10-30% of rebuild | High | Key by content hash for automatic cache hits. See `11-rspack-storage.md` §4 |
| 20 | **Memory-mapped I/O for pack files** | rspack_storage | Cache | 20-50% of cache I/O | Medium | Use mmap for reading. See `11-rspack-storage.md` §1 |
| 21 | **Incremental MangleExports** | rspack_plugin_javascript | Rebuild | Avoids full re-hash | High | Track changed exports, only re-mangle those. See `02-rspack-plugin-javascript.md` §7 |
| 22 | **Zero-copy deserialization with rkyv** | rspack_cacheable | Cache | 10-30% of cache load | Medium | Ensure archived types are used directly. See `16-rspack-cacheable.md` §1 |

---

## Tier 5 — Architectural (Long-term)

| # | Opportunity | Crate | Phase | Est. Impact | Effort | Details |
|---|-----------|-------|-------|-------------|--------|---------|
| 23 | **Arena allocation for per-module data** | rspack_allocator | BuildModuleGraph | 5-15% of make | High | Use bumpalo/arena for AST and dependency objects. See `10-rspack-allocator.md` §3 |
| 24 | **Split Compilation struct into focused sub-structs** | rspack_core | All | 1-5% total | High | Improve cache locality. See `01-rspack-core.md` §11 |
| 25 | **Streaming/lazy AST parser** | rspack_plugin_javascript | BuildModuleGraph | 5-10% of make | Very High | Only materialize AST nodes that dependency scanner inspects. See `02-rspack-plugin-javascript.md` §1 |
| 26 | **Lock-free module graph for concurrent writes** | rspack_core | BuildModuleGraph | 5-15% of make | Very High | Enable background tasks to write directly. See `01-rspack-core.md` §6 |

---

## Impact Distribution by Phase (react-10k cold build)

Based on typical profiling data for a react-10k build:

| Phase | % of Total Time | Top Opportunities | Combined Potential |
|-------|----------------|-------------------|-------------------|
| BuildModuleGraph (make) | ~40-50% | §2, §3, §4, §9, §16, §23, §25, §26 | 15-30% improvement |
| OptimizeDependencies | ~5-10% | §5, §17 | 10-30% improvement |
| BuildChunkGraph | ~5-10% | §1, §8 | 10-20% improvement |
| OptimizeChunks | ~3-5% | §10, §12 | 5-15% improvement |
| Hashing + CodeGen | ~15-20% | §6, §7, §11, §13, §14, §15 | 5-15% improvement |
| CreateChunkAssets | ~5-10% | §7 | 5-10% improvement |
| Emit | ~5% | — | Limited |

---

## Recommended Implementation Order

### Sprint 1 (Quick Wins — 1-2 weeks)
1. ✅ **§6**: Return iterators from `modules()` / `module_graph_modules()` (Low effort, immediate impact)
2. ✅ **§13**: Cache module-to-runtimes mapping (Low effort)
3. ✅ **§14-15**: RspackHash optimizations (Low effort)
4. ✅ **§18**: Early bail for empty hooks (Low effort)

### Sprint 2 (Medium Effort — 2-4 weeks)
5. ✅ **§1**: Replace BigUint with FixedBitSet (Medium effort, high impact)
6. ✅ **§3**: Merge SWC AST transform passes (Medium effort)
7. ✅ **§4**: Skip JS boundary for hookless taps (Medium effort)
8. ✅ **§7**: Batch tokio task spawns (Medium effort)

### Sprint 3 (High Effort — 1-2 months)
9. ✅ **§2**: Reduce task loop main-thread bottleneck (High effort, highest potential)
10. ✅ **§5**: Eliminate ExportsInfoData clones (High effort)
11. ✅ **§11**: Optimize hashing pipeline (High effort)

### Sprint 4 (Architectural — Ongoing)
12. ✅ **§23**: Arena allocation research
13. ✅ **§25**: Lazy/streaming parser research
14. ✅ **§26**: Lock-free module graph research

---

## Estimated Total Impact

If all Tier 1-3 opportunities were implemented:
- **Cold build**: 15-25% faster
- **Rebuild (watch mode)**: 20-40% faster (with Tier 4 additions)

The react-10k benchmark would see the most benefit from:
1. Make phase improvements (§2, §3, §9) — the make phase dominates at 10K modules
2. Code splitting improvements (§1) — BigUint scaling at 10K modules
3. Hashing pipeline optimization (§11) — 10K modules means 10K hash computations
