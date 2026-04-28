# Deep Dive: CodeSplitter BigUint Available Modules Tracking

**File**: `crates/rspack_core/src/compilation/build_chunk_graph/code_splitter.rs`

---

## How BigUint Is Used

The code splitter tracks which modules are "available" (already loaded) for each chunk group using arbitrary-precision bitmasks (`BigUint`). Each module gets a unique ordinal index, and the bitmask has that bit set if the module is available.

### Data Structures

```rust
pub(crate) ordinal_by_module: IdentifierMap<u64>,     // Module → bit index
pub(crate) mask_by_chunk: UkeyMap<ChunkUkey, BigUint>, // Chunk → modules in this chunk

// Per ChunkGroupInfo:
pub min_available_modules: Arc<BigUint>,               // Modules available from parent chunks
pub available_modules_to_be_merged: Vec<Arc<BigUint>>, // Pending merges
resulting_available_modules: Option<Arc<BigUint>>,     // min_available + this chunk's modules
```

### Hot Operations (called per module per chunk group)

1. **Bit test** — `min_available_modules.bit(*module_ordinal)` — checks if a module is already available
   - Called at line 1139, 1186, 1401, 1895, 1933, 1983
   - **6 call sites**, each hit for every module being processed

2. **Bit set** — `chunk_mask.set_bit(*module_ordinal, true)` — marks a module as part of a chunk
   - Called at line 1152, 1200
   - Hit for every module added to a chunk

3. **Bitwise OR** — `new_resulting_available_modules |= mask` — combines chunk masks
   - Called at line 122 in `calculate_resulting_available_modules`
   - Hit for every chunk in each chunk group

4. **Bitwise AND** — `min_available_modules & modules_to_be_merged` — intersection for merging
   - Called at line 2083
   - Hit during `process_chunk_groups_for_merging`

5. **Arc clone** — `min_available_modules.clone()` — reference count bump
   - Called at line 1367 and others
   - Each clone is an atomic increment

### Memory Layout at Scale

At 10,000 modules:
- Each `BigUint` is `⌈10000/64⌉ = 157` u64 words = **1,256 bytes**
- Each `Arc<BigUint>` adds 16 bytes overhead (strong + weak count + pointer)
- `mask_by_chunk`: One BigUint per chunk (~100-500 chunks) = **125KB - 628KB**
- `min_available_modules`: One per ChunkGroupInfo (~200-1000 groups) = **250KB - 1.25MB**
- `available_modules_to_be_merged`: Multiple per group during merging

### BigUint vs FixedBitSet Performance

| Operation | BigUint (10K bits) | FixedBitSet (10K bits) | Speedup |
|-----------|-------------------|----------------------|---------|
| `bit(i)` | Bounds check + word lookup | Direct word lookup | 1.5-2x |
| `set_bit(i, true)` | Bounds check + potential realloc | Direct write | 2-3x |
| `&` (AND) | Alloc new + word-by-word | In-place word-by-word | 2-3x (no alloc) |
| `\|=` (OR-assign) | Potential realloc + word-by-word | In-place word-by-word | 2-3x (no alloc) |
| `==` (equality) | Length check + word comparison | Fixed-size comparison | 1.5x |
| Clone | Heap alloc + memcpy 1.25KB | memcpy 1.25KB (or stack) | 2x |
| Memory | Heap-allocated Vec<u64> | Stack or heap, no Vec overhead | Less fragmentation |

The key advantages of `FixedBitSet` (or `BitVec`):
1. **No heap allocation for operations**: AND, OR, NOT produce results in pre-allocated storage
2. **No bounds checking**: Size is known at creation time
3. **SIMD-friendly**: Fixed-size arrays enable auto-vectorization
4. **No Arc needed**: Can be stored inline in `ChunkGroupInfo` if small enough

### Specific Hot Path: `process_chunk_groups_for_merging`

```rust
// Line 2072-2084 — THE HOTTEST BIGINT PATH
for modules_to_be_merged in available_modules_to_be_merged {
    if !cgi.min_available_modules_init {
        cgi.min_available_modules_init = true;
        cgi.min_available_modules = modules_to_be_merged;  // Arc clone
        changed = true;
        continue;
    }
    let orig = cgi.min_available_modules.clone();  // Arc<BigUint> clone (atomic inc)
    cgi.min_available_modules =
        Arc::new(cgi.min_available_modules.as_ref() & modules_to_be_merged.as_ref());
        //       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        //       Creates a NEW BigUint allocation for every merge operation!
    changed |= orig != cgi.min_available_modules;
    //         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    //         Full bitwise comparison (157 u64 words at 10K modules)
}
```

For each merge:
1. Clone Arc (atomic increment)
2. **Allocate new BigUint** for AND result
3. Compare old vs new (157-word comparison)
4. **Drop old Arc** (potential deallocation if refcount reaches 0)

With 500 chunk groups × 3 avg merges = 1,500 merge operations. At 10K modules, each merge allocates and compares 1.25KB. Total: ~1.875MB of temporary allocations just for merging.

---

## Recommended Fix

### Option A: Use `fixedbitset::FixedBitSet`

```rust
use fixedbitset::FixedBitSet;

struct ChunkGroupInfo {
    min_available_modules: FixedBitSet,          // No Arc, no heap alloc
    available_modules_to_be_merged: Vec<FixedBitSet>,
    resulting_available_modules: Option<FixedBitSet>,
}

// Pre-allocate with known module count
let module_count = all_modules.len();
let mut min_available = FixedBitSet::with_capacity(module_count);

// Merge operation — in-place, no allocation!
min_available.intersect_with(&modules_to_be_merged);
```

**Benefits**:
- `intersect_with` is **in-place** — no allocation
- `union_with` is **in-place** — no allocation
- No `Arc` overhead
- Auto-vectorizable inner loop

### Option B: Use `bitvec::BitVec` with SIMD

```rust
use bitvec::prelude::*;

type AvailableModules = BitVec<u64, Lsb0>;  // SIMD-friendly u64 backing

// Operations are similar but with explicit SIMD support
```

### Option C: Custom SIMD bitset

For maximum performance, a custom bitset using `std::simd` (nightly) or `packed_simd2`:

```rust
struct SimdBitSet {
    words: Vec<u64>,  // Aligned for SIMD
}

impl SimdBitSet {
    fn intersect_assign(&mut self, other: &Self) {
        // Process 4 u64s at a time with AVX2 (256-bit)
        for (a, b) in self.words.chunks_exact_mut(4).zip(other.words.chunks_exact(4)) {
            // This auto-vectorizes to VPAND (AVX2)
            a[0] &= b[0];
            a[1] &= b[1];
            a[2] &= b[2];
            a[3] &= b[3];
        }
    }
}
```

---

## Impact Projection

At 10K modules with FixedBitSet:
- Merge operations: 0 allocations (vs ~1,500 allocations × 1.25KB = 1.875MB)
- Bit test: ~2x faster (no bounds check)
- Total BuildChunkGraphPass: **20-40% faster**

This compounds with the fact that code splitting is single-threaded — every cycle saved directly reduces wall-clock time.
