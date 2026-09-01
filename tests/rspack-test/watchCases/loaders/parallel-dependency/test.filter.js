// Parallel loaders use Node.js worker threads, which are unavailable in the WASM build.
module.exports = () => !process.env.WASM;
