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

Sample count: **31** (each entry ≈ 3.23%).

| Overhead | Source:Line | Symbol | Analysis |
| --- | --- | --- | --- |
| 3.23% | `swc_ecma_codegen/src/lib.rs:666` | `Emitter::emit_list` (VarDeclarator) | Codegen loop over declarators; writer allocation churn. |
| 3.23% | `crates/rspack_plugin_javascript/.../call_hooks_name.rs:126` | `JavascriptParser::call_hooks_info` | Hook evaluation during dependency scan; synchronous hook overhead. |
| 3.23% | `lightningcss/.../values/easing` (`raw_vec/mod.rs:564`) | `EasingFunction::to_css` | CSS serialization cost; indicates CSS pipeline overhead. |
| 3.23% | `preset_env_base/src/query.rs:67` | `targets_to_versions` | Browserslist target resolution; cacheable between builds. |
| 3.23% | `swc_ecma_parser/src/parser/expr.rs:1796` | `parse_member_expr_or_new_expr_inner` | Parser recursion; parsing cost visible for TS/JS modules. |
| 3.23% | `swc_ecma_minifier/.../remove_invalid_bin` | `Optimizer::remove_invalid_bin` | Minifier validity cleanup pass. |
| 3.23% | `swc_ecma_minifier/.../can_absorb_negate` | `compress::util` | Boolean algebra simplification pass. |
| 3.23% | `hashbrown/control/bitmask.rs` | `Optimizer::visit_mut_expr` | HashMap bitmask operations inside SWC passes. |
| 3.23% | `swc_ecma_visit/generated.rs:16513` | `Preserver::visit_children_with` | Mangle-name preserver traversal. |
| 3.23% | `swc_ecma_visit/generated.rs:50375` | `optional_chaining` transform | Optional chaining transform traversal. |
| 3.23% | `napi_register_module_v1` | N-API module init | Binding initialization overhead (short runs). |

## Line-by-line Findings (Node target)

Sample count: **23** (each entry ≈ 4.35%).

| Overhead | Source:Line | Symbol | Analysis |
| --- | --- | --- | --- |
| 4.35% | `swc_ecma_parser/src/parser/stmt.rs:1302` | `parse_ident_name` | Parser identifier handling; TS/JS parsing cost. |
| 4.35% | `crates/rspack_plugin_javascript/.../walk_pre` | `pre_walk_statements` | Pre-walk dependency traversal in plugin parser. |
| 4.35% | `serde_json/src/lib.rs:412` | `MapDeserializer::end` | JSON parse cost (config/metadata) in build pipeline. |
| 4.35% | `swc_common/comments` | `take_leading` | Comment extraction overhead; can be gated if comments unused. |
| 4.35% | `swc_ecma_ast/eq_ignore_span` | Span comparison; AST equality checks (potentially prune). |
| 4.35% | `InsertedSemicolons` visitor | Semicolon insertion traversal; extra pass cost. |

## Line-by-line Findings (ESM output)

Sample count: **19** (each entry ≈ 5.26%).

| Overhead | Source:Line | Symbol | Analysis |
| --- | --- | --- | --- |
| 5.26% | `core/src/option.rs` | `JavascriptParser::walk_statements` | Dependency traversal still visible in ESM output. |
| 5.26% | `swc_ecma_codegen/text_writer/basic_impl.rs:98` | `emit_leading_comments` | Comment emission; could be disabled if no comments needed. |
| 5.26% | `swc_ecma_parser/lexer/jsx.rs:9` | `read_keyword_with` | JSX lexer keyword scanning. |
| 5.26% | `lightningcss/media_query` | `parse_with_options` | Media query parse cost (CSS pipeline). |
| 5.26% | `blake3` | `mi_free` | Hashing / allocation cleanup overhead. |

## Consolidated Biggest Impacts

1. **SWC minifier + usage analyzer** — repeated optimizer passes and AST traversal.
2. **HashMap churn** — `hashbrown` insert/remove in SWC and Rspack plugin logic.
3. **Parser + lexer** — SWC parser + V8 parser overhead during build.
4. **Codegen + writer allocations** — `swc_ecma_codegen` text writer hotspots.
5. **Runtime/allocator overhead** — libc + V8 GC/IC costs.
6. **Config/target resolution overhead** — `preset_env_base` and JSON parsing.

## Rspack-specific Hotspots (Non-SWC)

These lines are **inside Rspack crates** or closely related subsystems (not
pure SWC internals). They are prime candidates for optimization work beyond
SWC tuning.

### Web target

| Source:Line | Symbol | Analysis |
| --- | --- | --- |
| `crates/rspack_plugin_javascript/src/visitors/dependency/parser/call_hooks_name.rs:126` | `JavascriptParser::call_hooks_info` | Hook evaluation during dependency scan; synchronous hook overhead. |
| `crates/rspack_plugin_javascript/src/dependency/esm/esm_import_specifier_dependency.rs:362` | `VariableInfoId HashMap insert` | Import specifier tracking; map churn appears during ESM import analysis. |
| `preset_env_base/src/query.rs:67` | `targets_to_versions` | Browserslist target computation; cacheable per build. |
| `lightningcss/.../values/easing` | `EasingFunction::to_css` | CSS serialization in output stage. |

### Node target

| Source:Line | Symbol | Analysis |
| --- | --- | --- |
| `crates/rspack_plugin_javascript/.../walk_pre` | `pre_walk_statements` | Pre-walk dependency traversal in plugin parser. |
| `serde_json/src/lib.rs:412` | `MapDeserializer::end` | JSON parsing overhead; suggests caching parsed configs or pre-serialization. |
| `swc_common/comments` | `take_leading` | Comment extraction; could be gated when not required. |
| `InsertedSemicolons` visitor | `visit_children_with` | Semicolon insertion traversal overhead. |

### ESM output

| Source:Line | Symbol | Analysis |
| --- | --- | --- |
| `core/src/option.rs` | `JavascriptParser::walk_statements` | Parser dependency traversal in ESM mode. |
| `swc_ecma_codegen/text_writer/basic_impl.rs:98` | `emit_leading_comments` | Comment emission in ESM output. |
| `lightningcss/media_query` | `parse_with_options` | CSS media query parse cost. |

## Additional Non-SWC Libraries with Hot Lines

| Library | Example line | Notes |
| --- | --- | --- |
| `lightningcss` | `BorderImageHandler::flush` | CSS pipeline cost in production builds. |
| `hstr` | `Atom::eq`, macros.rs | Atom comparisons during transforms and resolver passes. |
| `mimalloc-rspack` | allocator entry points | Indicates allocation pressure in parser/minifier. |
| `blake3` | `mi_malloc_aligned` (AVX512) | Hashing / allocation interaction in node target. |
| `preset_env_base` | `targets_to_versions` | Browserslist target computation cost; cacheable. |
| `serde_json` | `MapDeserializer::end` | JSON parse overhead for config/metadata. |

## Next Steps to Improve Coverage

- Add `--repeat` for more samples once perf report timeouts are addressed.
- Add more fixtures (e.g., external `rspack-benchcases`) to diversify workloads.
- Use the `rspack.pftrace` output to correlate phase-level timing with the
  line-level hotspots above.
