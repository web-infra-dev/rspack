# react-10k Benchmark — Detailed Analysis Against Our Findings

**Source**: https://github.com/rspack-contrib/build-tools-performance/tree/main/cases/react-10k

---

## Benchmark Structure

The react-10k case contains **10,001 JSX files** organized in a nested tree:

```
src/
├── index.css
├── index.jsx         (imports f0-f8.jsx + renders them)
├── f0.jsx - f8.jsx   (9 root components, each imports 9 children from d0/)
├── d0/ - d9/         (10 directories, each with the same structure)
│   ├── f0.jsx - f8.jsx
│   └── d0/ - d9/
│       └── ... (4 levels deep)
```

Each component file:
```jsx
import React from 'react';
import I from '@iconify-icons/material-symbols/phone-android-sharp.js';
import { Icon } from '@iconify/react/dist/offline';
import C0 from './d0/f0.jsx';
// ... 8 more imports
function Component() {
  return (
    <div className="">
      <C0 /><C1 />...<C8 />
    </div>
  );
}
export default Component;
```

### Key Characteristics

| Property | Value |
|----------|-------|
| Total JSX files | 10,001 |
| External dependencies | `react`, `@iconify-icons/*`, `@iconify/react` |
| Imports per file | 12 (React + 2 iconify + 9 child components) |
| Total import statements | ~120,000 |
| Module type | JSX (requires SWC transform) |
| Async chunks | 0 (no dynamic imports) |
| CSS files | 1 (index.css) |
| Tree depth | 4 levels (d0/d0/d0/f*.jsx) |

### Rspack Configuration

```javascript
{
  mode: production/development,
  devtool: isProd ? false : undefined,
  target: ['web', 'es2022'],
  entry: './src/index.jsx',
  cache: { type: 'persistent' },
  module: {
    rules: [{
      test: /\.(js|ts|tsx|jsx)$/,
      use: { loader: 'builtin:swc-loader', options: { jsc: { target: 'es2022', ... } } }
    }]
  },
  plugins: [HtmlRspackPlugin, ReactRefreshPlugin (dev only)]
}
```

---

## How Our Findings Map to react-10k

### Finding #1: SideEffects O(n²) — **CRITICAL for react-10k**

At 10,001 modules, our confirmed O(n²) formula predicts:
```
T_side_effects = 2.92e-4 × 10001² = 29,206ms (debug)
T_side_effects_release ≈ 29,206 / 7 ≈ 4,172ms
```

**4.2 seconds** just for SideEffects in production mode. This is likely the single biggest bottleneck in the react-10k benchmark.

The react-10k benchmark is particularly susceptible because:
- All 10K modules are JSX components with `export default`
- Each module imports React (which has `sideEffects: false` in package.json)
- The @iconify packages also likely have `sideEffects` configuration
- The deep import tree means `get_side_effects_connection_state` recursion goes 4 levels deep

### Finding #2: BuildChunkGraph BigUint — **LOW impact for react-10k**

The react-10k case has **0 async chunks** (no dynamic imports). BuildChunkGraph just does a simple BFS with 1 chunk group. The BigUint operations are minimal.

However, in a real React SPA with route-based code splitting (200+ routes), this would be critical. The benchmark is not representative of real-world React apps in this regard.

### Finding #3: SplitChunks — **LOW impact for react-10k**

With 0 async chunks and default `splitChunks` config, there's little work for the SplitChunksPlugin. Only vendor chunk splitting (React, iconify) is relevant.

### Finding #4: Make Phase (Module Building) — **HIGH impact for react-10k**

At 10K modules:
```
T_make_release ≈ 0.107 × 10001 ≈ 1,070ms
T_make_wall (8 cores) ≈ 1,070/8 + 214 (main thread) ≈ 348ms
```

But each module requires:
1. **SWC JSX transform** — more expensive than plain JS due to JSX → createElement conversion
2. **React runtime: automatic** — additional transform overhead
3. **12 dependency scans per module** — 120K total dependencies to process

The main-thread bottleneck processes 120K dependencies:
- 120K HashMap inserts for dependencies
- 120K connection creations
- 10K module additions

### Finding #5: Module Concatenation — **MODERATE impact for react-10k**

With 10K ESM modules, ModuleConcatenationPlugin attempts to concatenate them. Most will fail because they import external dependencies (React, iconify) that can't be concatenated.

The analysis overhead (evaluating 10K modules for concatenation eligibility) is significant even if few succeed. Our profiling showed 26ms for 471 modules → projected ~550ms for 10K modules.

### Finding #6: FlagDependencyExports/Usage — **HIGH impact for react-10k**

At 10K modules with 12 imports each:
- FlagDependencyExports processes 120K dependency exports
- FlagDependencyUsage traces usage across 120K connections
- Both plugins clone ExportsInfoData for parallel processing

### Finding #7: Memory — **HIGH for react-10k**

Projected memory at 10K modules:
- Module data: ~7.5MB
- Dependencies (120K): ~22.8MB
- Connections (120K): ~11.5MB
- ExportInfoData: ~22MB
- Source code: ~48MB (each JSX file ~5KB)
- **Total: ~112MB+ just for module graph data**

---

## Projected react-10k Build Times

### Production Cold Build (Release Mode, 8 Cores)

| Phase | Projected Time | Notes |
|-------|---------------|-------|
| build module graph | ~350ms wall | 10K modules, parallelized, main-thread bottleneck ~214ms |
| finish modules | ~120ms | FlagDependencyExports |
| **optimize dependencies** | **~4,200ms** | **SideEffects O(n²) — THE BOTTLENECK** |
| build chunk graph | ~15ms | Only 1 chunk group, no async |
| optimize chunks | ~20ms | Basic vendor splitting |
| optimize chunk modules | ~550ms | ModuleConcatenation analysis |
| module hashing | ~15ms | Parallelized |
| code generation | ~50ms | Parallelized |
| hashing | ~10ms | 1-2 chunks |
| chunk assets | ~10ms | 1-2 chunks |
| minification | ~500ms | If enabled, ~50KB output |
| **Total (no minify)** | **~5,400ms** | Dominated by SideEffects |
| **Total (with minify)** | **~5,900ms** | |

### After SideEffects O(n²) Fix

| Phase | Before | After |
|-------|--------|-------|
| optimize dependencies | 4,200ms | ~430ms |
| **Total** | ~5,400ms | **~1,630ms** |

A **3.3x speedup** from fixing just the SideEffects issue!

### Development Mode

| Phase | Projected Time | Notes |
|-------|---------------|-------|
| build module graph | ~350ms wall | Same as production |
| build chunk graph | ~15ms | |
| code generation | ~100ms | eval wrapping overhead |
| **Total** | **~600ms** | No optimize dependencies, no concat |

---

## Recommendations Specific to react-10k

### Priority 1: Fix SideEffects O(n²)
**Impact**: 4,200ms → 430ms. This alone would make the react-10k benchmark ~3.3x faster.

### Priority 2: Optimize FlagDependencyExports/Usage Cloning
**Impact**: With 120K dependencies and 10K ExportsInfoData clones, this adds ~200-400ms.

### Priority 3: Merge SWC JSX Transform Passes
**Impact**: With JSX, there are even more SWC passes (JSX transform + the standard 3). Merging saves ~15% of make phase.

### Priority 4: Persistent Cache
The benchmark config already enables `cache: { type: 'persistent' }`. Ensuring this cache works well for the react-10k case would provide dramatic rebuild speedups.

---

## What the react-10k Benchmark Does NOT Test

1. **Route-based code splitting** — no dynamic imports (would reveal BuildChunkGraph issues)
2. **CSS processing** — only 1 CSS file (CSS scaling not tested)
3. **TypeScript** — uses JSX with TS parser config but no actual TS features
4. **Source maps** — disabled in production mode
5. **Real-world module sizes** — each component is ~20 lines (real components are 100-1000 lines)
6. **Third-party dependencies** — only React + iconify (real apps have 100+ npm packages)
7. **Watch mode rebuild** — benchmarks measure cold build only

A more comprehensive benchmark would add:
- 200 route-based async chunks
- Mixed TypeScript/JavaScript
- CSS modules
- Source maps in production
- Larger per-module source size
