// Export the `Buffer` from wasm-runtime and reuse it for the overall polyfill in `@rspack/browser`
// @ts-expect-error
export { Buffer } from "@napi-rs/wasm-runtime/fs";
