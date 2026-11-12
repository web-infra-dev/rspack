// Parallel loaders use Node.js worker threads, which are unavailable in the WASM build.
export default () => !process.env.WASM;
