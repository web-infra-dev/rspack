// ESM bridge that re-exports from a CJS module
// This module is ESM but its dependency is CJS, so the CJS module
// and its dependencies (including fs) won't be scope-hoisted
export { cjsResult } from './cjs-consumer.cjs'
