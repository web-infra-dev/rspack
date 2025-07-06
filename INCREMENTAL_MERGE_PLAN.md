# Incremental Merge Plan: treeshake-fix → swc-macro

## Overview
Merging 74+ commits from treeshake-fix branch into swc-macro branch incrementally to ensure stability.

## Strategy
Merge commits in logical groups to maintain build stability and make rollback easier if issues arise.

## Progress Status

### ✅ Group 1: Foundation & Infrastructure (COMPLETED)
- **Status**: ✅ COMPLETED
- **Commits**: `ad6425a4f` - `308e2cb9c`: Initial tree-shaking macro infrastructure  
- **Focus**: Basic export tracking and PURE annotations
- **Build**: ✅ PASSED (with fixes for compilation errors)
- **Tests**: ⏱️ TIMEOUT (expected for full CI, build working)
- **Issues Fixed**: 
  - ExportInfoSetter API changes
  - Queue parameter mismatches
  - Missing module file references
  - PrefetchExportsInfoMode enum variants

### ☐ Group 2: Core ConsumeShared Implementation (NEXT)
- **Status**: ⏳ PENDING
- **Commits**: `abb97ba83` - `60ddf9aff`: Core ConsumeShared functionality
- **Focus**: Export usage analysis and metadata handling

### ☐ Group 4: Macro Handling Enhancements 
- **Status**: ⏳ PENDING  
- **Commits**: `2448db114` - `129abb699`: Enhanced macro processing
- **Focus**: Conditional pure annotations and macro refinements

### ☐ Group 5: Bug Fixes & Refinements
- **Status**: ⏳ PENDING
- **Commits**: `ed9ed2d20` - `0b48a4f44`: Various bug fixes and improvements
- **Focus**: Compilation errors, borrow checker fixes, cleanup

### ☐ Group 6: Final Integration & Testing
- **Status**: ⏳ PENDING
- **Commits**: Latest commits including merge conflict resolutions
- **Focus**: Integration with main branch and final testing

### ☐ Group 3: Testing & Configuration (LAST)
- **Status**: ⏳ PENDING - MOVED TO END
- **Commits**: `a388b70dc` - `dd8e6b865`: Testing infrastructure and examples
- **Focus**: Test setup and configuration files
- **Note**: Moved to end to migrate implementation first, then adapt tests

## Execution Plan
1. ✅ Group 1: Foundation & Infrastructure - COMPLETED
2. ⏳ Group 2: Core ConsumeShared Implementation - IN PROGRESS  
3. ☐ Group 4: Macro Handling Enhancements
4. ☐ Group 5: Bug Fixes & Refinements
5. ☐ Group 6: Final Integration & Testing
6. ☐ Group 3: Testing & Configuration - MOVED TO END

## Rollback Strategy
- Each group merge is a separate commit point
- Can rollback to previous group if critical issues arise
- Maintain clean commit history for easier debugging