# Module Federation Runtime Modules Analysis - Complete Documentation

**Generated**: October 27, 2025  
**Branch**: feature/async-startup-runtime-promise  
**Crate**: rspack_plugin_mf

---

## Overview

This directory contains comprehensive analysis of how Module Federation runtime modules work in rspack, with detailed recommendations for implementing Promise.all wrapping for federation startup.

## Documentation Files

### 1. MF_ANALYSIS_SUMMARY.txt (13 KB) - START HERE
**Best for**: Executive summary, quick reference, all answers at a glance

**Contents**:
- Key findings for all 6 analysis questions
- Architecture summary with hierarchy diagram
- Recommendations for Promise.all wrapping
- Implementation checklist
- Critical files and locations

**Read this first for**: Quick understanding of the complete system

---

### 2. MF_RUNTIME_ANALYSIS.md (34 KB) - DETAILED REFERENCE
**Best for**: Deep dive, complete architecture understanding, code examples

**Contents**:
- Executive summary
- 8 major sections:
  1. Runtime Requirements Added by Federation (4 mechanisms)
  2. How Consumes and Remotes Chunk Handlers Are Registered
  3. Federation Startup Integration with Async Mode
  4. All Runtime Modules in the MF Crate (6 modules)
  5. No Custom "federation-entry-startup" Requirement
  6. How EmbedFederationRuntimeModule Interacts with Startup
  7. Can We Add Custom Runtime Requirement for Federation Startup?
  8. Recommendations for Promise.all Wrapping Logic

**Code Locations**:
- Actual file paths and line numbers
- Function references
- Code snippets from codebase
- Generated JavaScript examples

**Read this for**: Complete understanding of architecture and implementation

---

### 3. MF_RUNTIME_QUICK_REFERENCE.md (10 KB) - DEVELOPER CHECKLIST
**Best for**: Implementation guide, quick lookups, handler details

**Contents**:
- Quick answers to key questions
- Implementation checklist (3 phases)
- Key files and line references (table)
- Current vs. proposed code flow (JavaScript)
- Handler registration details with process flows
- Critical interactions diagram
- Runtime requirements summary
- Next steps for implementation

**Read this for**: Implementing Promise.all wrapping, understanding handlers

---

## Quick Navigation

### If you want to understand...

**What are the 6 runtime modules in federation?**
- Start with: MF_ANALYSIS_SUMMARY.txt (QUESTION 6)
- Then read: MF_RUNTIME_ANALYSIS.md (Section 4)

**How do remotes/consumes handlers work?**
- Start with: MF_RUNTIME_QUICK_REFERENCE.md (Handler Registration Details)
- Then read: MF_RUNTIME_ANALYSIS.md (Section 2)
- Code files: 
  - `/crates/rspack_plugin_mf/src/container/remote_runtime_module.rs`
  - `/crates/rspack_plugin_mf/src/sharing/consume_shared_runtime_module.rs`

**How to add Promise.all wrapping?**
- Start with: MF_RUNTIME_QUICK_REFERENCE.md (Implementation Checklist)
- Then read: MF_RUNTIME_ANALYSIS.md (Section 8)
- Code file: `/crates/rspack_plugin_mf/src/container/embed_federation_runtime_module.rs`

**What are all the runtime requirements?**
- MF_ANALYSIS_SUMMARY.txt (QUESTION 1)
- MF_RUNTIME_QUICK_REFERENCE.md (Runtime Requirements Summary)

**How does EmbedFederationRuntimeModule work?**
- MF_ANALYSIS_SUMMARY.txt (QUESTION 4)
- MF_RUNTIME_ANALYSIS.md (Section 6)

---

## Key Findings Summary

### Question 1: What runtime requirements does federation add?
Federation adds requirements through 4 plugins:
- ModuleFederationRuntimePlugin: FederationDataRuntimeModule + STARTUP_ENTRYPOINT (if async)
- EmbedFederationRuntimePlugin: STARTUP or STARTUP_ENTRYPOINT
- ConsumeSharedPlugin: MODULE, SHARE_SCOPE_MAP, INITIALIZE_SHARING, etc.
- ContainerReferencePlugin: MODULE, RemoteRuntimeModule (if needed)

### Question 2: How are consumes/remotes handlers registered?
Through runtime module generation:
- RemoteRuntimeModule creates `__webpack_require__.f.remotes` handler
- ConsumeSharedRuntimeModule creates `__webpack_require__.f.consumes` handler
- Both called during `__webpack_require__.e(chunkId)` with promises array

### Question 3: Is there a "federation-entry-startup" requirement?
NO. Federation reuses existing:
- STARTUP_ENTRYPOINT (when mf_async_startup = true)
- STARTUP (when mf_async_startup = false)

### Question 4: How does EmbedFederationRuntimeModule work?
Creates "prevStartup wrapper" pattern:
1. Saves original `__webpack_require__.x` (or `.X`)
2. Replaces with wrapper that executes federation modules
3. Calls prevStartup()
4. Uses hasRun flag to prevent double execution

**Limitation**: Synchronous even when async capable

### Question 5: Can we add Promise.all wrapping?
YES - Recommended approach:
- Modify EmbedFederationRuntimeModule::generate()
- Detect federation deps needing chunks
- Wrap in Promise.all when mf_async_startup=true
- Minimal changes, uses existing async capability

### Question 6: What runtime modules exist?
6 main modules:
1. FederationDataRuntimeModule (federation config data)
2. RemoteRuntimeModule (remotes loading handler)
3. ConsumeSharedRuntimeModule (consumes handler)
4. ShareRuntimeModule (sharing initialization)
5. ExposeRuntimeModule (container initialization)
6. EmbedFederationRuntimeModule (startup wrapping)

---

## Critical File Locations

### Primary Implementation Files
| File | Purpose | Key Lines |
|------|---------|-----------|
| `embed_federation_runtime_module.rs` | Federation startup wrapper | 45-113 |
| `module_federation_runtime_plugin.rs` | Federation requirements | 39-67 |

### Reference Files
| File | Purpose | Key Lines |
|------|---------|-----------|
| `embed_federation_runtime_plugin.rs` | Plugin orchestration | 56-90 |
| `remote_runtime_module.rs` | Remotes handler | 40-108 |
| `consume_shared_runtime_module.rs` | Consumes handler | 39-155 |
| `share_runtime_module.rs` | Sharing initialization | 32-122 |
| `federation_data_runtime_module.rs` | Federation data | 51-105 |

### Generated JavaScript
| File | Contains |
|------|----------|
| `remotesLoading.js` | Remotes handler implementation |
| `consumesLoading.js` | Consumes handler implementation |
| `consumesCommon.js` | Shared utilities |
| `consumesInitial.js` | Initial setup |
| `initializeSharing.js` | Sharing logic |

---

## Implementation Roadmap

### Phase 1: Understanding
- [ ] Read MF_ANALYSIS_SUMMARY.txt (5 min)
- [ ] Review MF_RUNTIME_ANALYSIS.md Sections 4, 6 (15 min)
- [ ] Examine embed_federation_runtime_module.rs code (10 min)

### Phase 2: Detection
- [ ] Analyze federation dependency collection
- [ ] Check chunk dependencies (ENSURE_CHUNK)
- [ ] Determine async wrapping needs

### Phase 3: Implementation
- [ ] Modify EmbedFederationRuntimeModule::generate()
- [ ] Add Promise.all wrapper detection
- [ ] Generate appropriate code (sync or async)

### Phase 4: Testing
- [ ] Test mf_async_startup=true with chunk deps
- [ ] Test mf_async_startup=false (sync only)
- [ ] Test error handling
- [ ] Use examples/basic for validation

---

## Architecture at a Glance

```
Entry Chunk Startup
     ↓
EmbedFederationRuntimeModule (Stage 11)
     ├─ Wraps __webpack_require__.x or .X
     ├─ Executes federation runtime dependencies
     └─ Calls prevStartup()
         ↓
    Chunk Handler System
         ├─ RemoteRuntimeModule (.f.remotes)
         ├─ ConsumeSharedRuntimeModule (.f.consumes)
         ├─ ShareRuntimeModule (.I initializer)
         └─ FederationDataRuntimeModule (config)
```

---

## Current vs. Proposed Implementation

### Current (Synchronous)
```javascript
var prevStartup = __webpack_require__.x;
__webpack_require__.x = function() {
  if (!hasRun) {
    __webpack_require__(123);  // Blocking
    __webpack_require__(124);  // Blocking
  }
  return prevStartup();
};
```

### Proposed (Async with Promise.all)
```javascript
var prevStartup = __webpack_require__.X;
var fedPromise;
__webpack_require__.X = function() {
  if (!hasRun) {
    fedPromise = Promise.all([
      __webpack_require__.e(1),  // Parallel
      __webpack_require__.e(2)   // Parallel
    ]).then(function() {
      __webpack_require__(123);  // After deps ready
      return typeof prevStartup === 'function' ? prevStartup() : undefined;
    });
    return fedPromise;
  }
  return fedPromise || prevStartup();
};
```

---

## How to Use This Documentation

### For Quick Answers
Use **MF_ANALYSIS_SUMMARY.txt**
- All 6 questions answered directly
- Architecture overview
- Implementation checklist
- Take: 5-10 minutes to read

### For Implementation
Use **MF_RUNTIME_QUICK_REFERENCE.md**
- Implementation checklist (3 phases)
- Handler registration details
- Code generation examples
- Next steps
- Take: 15-20 minutes to understand and implement

### For Complete Understanding
Read all three documents in order:
1. MF_ANALYSIS_SUMMARY.txt (5 min)
2. MF_RUNTIME_QUICK_REFERENCE.md (10 min)
3. MF_RUNTIME_ANALYSIS.md (30 min)

Total: ~45 minutes for complete understanding

---

## Recommendations

### Best Approach for Promise.all Wrapping
**Option A (RECOMMENDED)**: Extend EmbedFederationRuntimeModule
- Modify one file: `embed_federation_runtime_module.rs`
- Detect if federation deps need async wrapping
- Generate Promise.all wrapper when mf_async_startup=true
- Benefits: Minimal changes, uses existing async capability

**Why not Option B**: Creating new FEDERATION_STARTUP requirement
- Would require changes to multiple files
- New RuntimeGlobal constant
- More infrastructure overhead
- Redundant unless adding more special handling

### Immediate Next Steps
1. Read MF_ANALYSIS_SUMMARY.txt (complete overview)
2. Read MF_RUNTIME_QUICK_REFERENCE.md Implementation Checklist
3. Review embed_federation_runtime_module.rs code
4. Implement Phase 1 detection
5. Test with examples/basic

---

## Files in This Analysis

```
rspack/
├── README_MF_ANALYSIS.md (this file)
├── MF_ANALYSIS_SUMMARY.txt (13 KB) - Executive summary
├── MF_RUNTIME_ANALYSIS.md (34 KB) - Complete technical analysis
└── MF_RUNTIME_QUICK_REFERENCE.md (10 KB) - Implementation guide

Total: ~57 KB of comprehensive documentation
```

---

## Questions Answered

This analysis comprehensively answers:
1. What runtime requirements does federation add? ✓
2. How are consumes and remotes chunk handlers registered? ✓
3. Is there a "federation-entry-startup" custom requirement? ✓
4. How does EmbedFederationRuntimeModule interact with startup? ✓
5. Can we add a custom runtime requirement for federation startup wrapping? ✓
6. What are all runtime modules in the mf crate? ✓

Plus comprehensive recommendations for Promise.all wrapping logic.

---

## Contact & References

**Branch**: feature/async-startup-runtime-promise  
**Crate**: rspack_plugin_mf  
**Analysis Date**: October 27, 2025

All code references point to actual rspack source files in:
`/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/`

---

**Start Reading**: Open `MF_ANALYSIS_SUMMARY.txt` for quick overview!
