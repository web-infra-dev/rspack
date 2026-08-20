// Loader cache currently passes Buffer values through N-API. Emnapi's WASM
// binding cannot handle this path yet, so skip until WASM loader cache is supported.
module.exports = () => !process.env.WASM;
