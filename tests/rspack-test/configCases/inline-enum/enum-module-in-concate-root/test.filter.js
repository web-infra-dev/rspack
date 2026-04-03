// WASM CI: sporadic `RuntimeError: memory access out of bounds` while converting the
// resolved JS loader promise back to `JsLoaderContext` (napi `FromNapiValue` on
// `Promise<JsLoaderContext>` inside an emnapi Worker). See e.g. GitHub Actions job logs
// around `wasm://wasm/rspack_node.wasm` + `JsLoaderContext` / `call_js_cb`.
module.exports = () => {
	return !process.env.WASM;
};
