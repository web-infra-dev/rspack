# Rspack Line-by-Line Performance Review

This report summarizes a line-by-line CPU sampling review of Rspack using the
existing `tests/bench/fixtures/ts-react` benchmark fixture. The analysis is
based on the generated `line-report.txt` from the profiling script. It covers
three build variants to maximize coverage:

- **Web (default browser target)**
- **Node target**
- **ESM output (output.module)**

## Method

- Benchmark fixture: `tests/bench/fixtures/ts-react`
- Build: `build:binding:profiling` + `build:js`
- Commands:
  ```sh
  # Web target
  pnpm run profile:line-report -- \
    --config scripts/profile/bench-ts-react.config.cjs \
    --outDir ./.rspack-profile-ts-react-web \
    --traceFilter OVERVIEW \
    --perf /usr/lib/linux-tools-6.8.0-100/perf \
    --addr2line /usr/bin/llvm-addr2line

  # Node target
  pnpm run profile:line-report -- \
    --config scripts/profile/bench-ts-react.node.config.cjs \
    --outDir ./.rspack-profile-ts-react-node \
    --traceFilter OVERVIEW \
    --perf /usr/lib/linux-tools-6.8.0-100/perf \
    --addr2line /usr/bin/llvm-addr2line

  # ESM output
  pnpm run profile:line-report -- \
    --config scripts/profile/bench-ts-react.esm.config.cjs \
    --outDir ./.rspack-profile-ts-react-esm \
    --traceFilter OVERVIEW \
    --perf /usr/lib/linux-tools-6.8.0-100/perf \
    --addr2line /usr/bin/llvm-addr2line
  ```

> **Note:** Because the run is short, the sample count is low and each entry
> has equal weight. For higher confidence, rerun with `--repeat` and/or a higher
> sample rate.

## Line-by-line Findings (Web target)

Sample count: **21** (each entry ≈ 4.76%).

| Overhead | Source:Line | Symbol | Analysis |
| --- | --- | --- | --- |
| 4.76% | ?? | Builtin:RecordWriteIgnoreFP (V8) | Write barrier for GC; expected VM overhead during compilation. |
| 4.76% | ?? | malloc (libc) | Allocation hot path; likely dominated by AST/minifier structures. |
| 4.76% | ?? | v8::Utf16CharacterStream::AdvanceUntil (SkipWhiteSpace) | Parser whitespace scanning in V8 while loading JS/CLI. |
| 4.76% | ?? | v8::ObjectLiteral::CalculateEmitStore | V8 AST emission for object literals; VM compile overhead. |
| 4.76% | ?? | v8::ParserBase::ParseVariableStatement | V8 parsing overhead for JS modules. |
| 4.76% | ?? | v8::Serializer::ObjectSerializer::OutputRawData | V8 snapshot serialization cost. |
| 4.76% | ?? | v8::String::ComputeAndSetRawHash | String hashing overhead. |
| 4.76% | ?? | v8::StringsStorage::Release | String storage churn. |
| 4.76% | `triomphe/src/thin_arc.rs:248` | `swc_ecma_minifier::optimize` | SWC minifier entry; dominant Rust-side cost. |
| 4.76% | `swc_ecma_visit/generated.rs:0` | `swc_ecma_utils::cast_to_bool` | AST traversal / type coercion; frequent during transforms. |
| 4.76% | `mimalloc-rspack/src/lib.rs:0` | `swc_ecma_parser::parse_bin_op_recursively_inner` | Parser recursion + allocator overhead. |
| 4.76% | `crates/rspack_plugin_javascript/.../walk.rs:1355` | `JavascriptParser::walk_statements` | Dependency scan traversal in Rspack plugin. |
| 4.76% | `swc_ecma_minifier/src/compress/optimize/mod.rs:484` | `Optimizer::handle_stmts` | Minifier statement optimization pass. |
| 4.76% | `better_scoped_tls/src/lib.rs:74` | `Resolver::mark_for_ref_inner` | Resolver metadata marking; symbol resolution overhead. |
| 4.76% | `lightningcss/.../border_image.rs` | `BorderImageHandler::flush` | CSS minification/output cost. |
| 4.76% | `mimalloc-rspack/src/lib.rs:51` | `Optimizer::visit_mut_fn_decl` | Minifier visitor on function declarations. |
| 4.76% | `hashbrown/src/util.rs:13` | `simplify::Analyzer::visit_children_with` | HashMap utility hot path in DCE analyzer. |
| 4.76% | `swc_atoms/src/lib.rs:39` | `VarWithOutInitCounter::visit_children_with` | Atom usage during minifier pure-vars pass. |
| 4.76% | `core/src/slice/iter/macros.rs` | `write_punct` (codegen) | Codegen writer overhead. |
| 4.76% | `swc_ecma_visit/generated.rs:62468` | `Fixer::visit_mut_children_with` | Fixer traversal; post-transform cleanup. |
| 4.76% | `swc_ecma_parser/src/lexer/mod.rs:1093` | lexer table SEM0 | Lexer dispatch hot path. |

## Line-by-line Findings (Node target)

Sample count: **19** (each entry ≈ 5.26%).

| Overhead | Source:Line | Symbol | Analysis |
| --- | --- | --- | --- |
| 5.26% | ?? | Builtin:KeyedLoadIC_Megamorphic (V8) | Property access IC overhead; VM hot path. |
| 5.26% | ?? | ld-linux resolver | Loader relocation cost. |
| 5.26% | ?? | pthread_mutex_unlock | libc sync; allocator/GC related. |
| 5.26% | ?? | v8::StringShape::DispatchToSpecificType | String lookup; VM runtime work. |
| 5.26% | ?? | PagedSpaceObjectIterator::Next | GC heap walk. |
| 5.26% | ?? | v8::Scanner::Next | Parser tokenization in V8. |
| 5.26% | ?? | v8::Scanner::ScanString | V8 string literal scanning. |
| 5.26% | ?? | v8::String::ToCString | String conversion cost. |
| 5.26% | ?? | BytecodeRegisterOptimizer::Flush | Bytecode optimizer overhead. |
| 5.26% | `core/src/ptr/mod.rs:547` | `swc_common::Globals::with` | Scoped TLS for syntax context; SWC infra cost. |
| 5.26% | `core/src/mem/mod.rs:893` | `JavascriptParser::call_hooks_info` | Hook evaluation during dependency scan. |
| 5.26% | `indexmap/src/map.rs:739` | `FnEnvHoister::to_decl` | Environment hoisting in SWC utils. |
| 5.26% | `alloc/src/vec/mod.rs:2659` | `rename::Analyzer::add_usage` | Usage tracking; vector growth. |
| 5.26% | `swc_ecma_minifier/src/compress/optimize/if_return.rs:496` | `Optimizer::merge_sequential_expr` | Minifier sequence merge optimization. |
| 5.26% | `alloc/src/vec/mod.rs:1639` | `rspack_core::parse_resolve` | Resolve option merge; configuration processing. |
| 5.26% | `hstr/src/wtf8/mod.rs:0` | `Atom::eq` | Atom string comparisons. |
| 5.26% | `hstr/src/macros.rs:55` | `Resolver::visit_mut_children_with` | Resolver traversal in transforms. |
| 5.26% | `core/src/fmt/builders.rs:330` | lexer whitespace table | Lexer whitespace table dispatch. |
| 5.26% | `blake3_avx512_x86-64_unix.S:4471` | `mi_malloc_aligned` | SIMD hash/alloc; hashing overhead. |

## Line-by-line Findings (ESM output)

Sample count: **19** (each entry ≈ 5.26%).

| Overhead | Source:Line | Symbol | Analysis |
| --- | --- | --- | --- |
| 5.26% | ?? | Builtin:Call_ReceiverIsAny (V8) | JS call dispatch overhead. |
| 5.26% | ?? | malloc (libc) | Allocation pressure during compilation. |
| 5.26% | ?? | pthread_mutex_lock | Sync in allocator/VM runtime. |
| 5.26% | ?? | V8 `ScanIdentifierOrKeywordInner` | Token scanning for identifiers/keywords. |
| 5.26% | ?? | ClassBoilerplate::New | Class literal compilation overhead. |
| 5.26% | ?? | Utf8DecoderBase | UTF-8 decoding overhead. |
| 5.26% | ?? | BytecodeGenerator::VisitPropertyLoad | VM bytecode generation. |
| 5.26% | ?? | CalculateLineEndsImpl (two entries) | Script line map generation overhead. |
| 5.26% | `core/src/num/uint_macros.rs:355` | `Globals::with` (marks) | Syntax context scope setup. |
| 5.26% | `core/src/ptr/mod.rs:805` | `drop_in_place<Result<RawAliasOptionItem>>` | Raw resolve option cleanup in binding layer. |
| 5.26% | `alloc/src/vec/mod.rs:2659` | `rename::Analyzer::add_usage` | Usage tracking cost. |
| 5.26% | `swc_ecma_parser/src/lexer/token.rs:373` | `parse_subscript` | Parser subscript processing. |
| 5.26% | `core/src/ptr/mod.rs:1944` | `optimize_expr_in_bool_ctx` | Minifier bool context optimization. |
| 5.26% | `core/src/ptr/mod.rs:0` | `hashbrown::HashMap::insert` | Map insertion during transforms. |
| 5.26% | `core/src/ptr/mod.rs:547` | `spec_from_iter_nested` | Iterator materialization in minifier passes. |
| 5.26% | `swc_atoms/src/lib.rs:39` | `visit_mut_expr_stmt` | Minifier visitor on expr statements. |
| 5.26% | `core/src/ptr/non_null.rs:1692` | `petgraph::GraphMap::to_index` | Graph map indexing in DCE. |
| 5.26% | `swc_ecma_visit/generated.rs:60104` | `ExplicitResourceManagement::visit_mut_children_with` | Proposal transform traversal. |

## Consolidated Biggest Impacts

1. **SWC minifier + usage analyzer** — repeated optimizer passes and AST traversal.
2. **HashMap churn** — `hashbrown` insert/remove in SWC and Rspack plugin logic.
3. **Parser + lexer** — SWC parser + V8 parser overhead during build.
4. **Codegen + writer allocations** — `swc_ecma_codegen` text writer hotspots.
5. **Runtime/allocator overhead** — libc + V8 GC/IC costs.

## Rspack-specific Hotspots (Non-SWC)

These lines are **inside Rspack crates** or closely related subsystems (not
pure SWC internals). They are prime candidates for optimization work beyond
SWC tuning.

### Web target

| Source:Line | Symbol | Analysis |
| --- | --- | --- |
| `crates/rspack_plugin_javascript/src/visitors/dependency/parser/walk.rs:1355` | `JavascriptParser::walk_statements` | Dependency scanning traversal; repeated AST walks amplify cost across modules. |
| `crates/rspack_plugin_javascript/src/dependency/esm/esm_import_specifier_dependency.rs:362` | `VariableInfoId HashMap insert` | Import specifier tracking; map churn appears during ESM import analysis. |
| `lightningcss/.../border_image.rs` | `BorderImageHandler::flush` | CSS minification output path; indicates CSS pipeline cost in production. |

### Node target

| Source:Line | Symbol | Analysis |
| --- | --- | --- |
| `crates/rspack_core/options/resolve/clever_merge` (alloc vec growth) | `parse_resolve` | Resolve option normalization; may be sensitive to large config objects. |
| `crates/rspack_plugin_javascript/.../call_hooks_info` | `JavascriptParser::call_hooks_info` | Hook evaluation during dependency scan; synchronous JS hook overhead is visible. |

### ESM output

| Source:Line | Symbol | Analysis |
| --- | --- | --- |
| `rspack_binding_api::options::raw_resolve` (drop_in_place) | `RawAliasOptionItem` cleanup | N-API option conversion cleanup; suggests overhead in option marshaling. |

## Additional Non-SWC Libraries with Hot Lines

| Library | Example line | Notes |
| --- | --- | --- |
| `lightningcss` | `BorderImageHandler::flush` | CSS pipeline cost in production builds. |
| `hstr` | `Atom::eq`, macros.rs | Atom comparisons during transforms and resolver passes. |
| `mimalloc-rspack` | allocator entry points | Indicates allocation pressure in parser/minifier. |
| `blake3` | `mi_malloc_aligned` (AVX512) | Hashing / allocation interaction in node target. |

## Next Steps to Improve Coverage

- Add `--repeat` for more samples once perf report timeouts are addressed.
- Add more fixtures (e.g., external `rspack-benchcases`) to diversify workloads.
- Use the `rspack.pftrace` output to correlate phase-level timing with the
  line-level hotspots above.
