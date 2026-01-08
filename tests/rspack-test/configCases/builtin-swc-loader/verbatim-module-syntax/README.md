# verbatimModuleSyntax Test Case

This test case verifies that `verbatimModuleSyntax: true` is enabled by default in builtin:swc-loader.

## What is verbatimModuleSyntax?

`verbatimModuleSyntax` is a TypeScript compiler option that:
- Requires explicit `type` modifier for type-only imports/exports
- Ensures type-only imports are completely erased at runtime
- Provides better compatibility with ES modules and modern bundlers

## Expected Behavior

When `verbatimModuleSyntax: true` is enabled:
- `import type { User } from './types'` - Type import is completely removed from output
- `import { DEFAULT_ROLE } from './types'` - Value import is preserved in output
- Type-only exports are also removed from the bundle

This improves tree-shaking and reduces bundle size by ensuring TypeScript types don't accidentally end up in the runtime code.

