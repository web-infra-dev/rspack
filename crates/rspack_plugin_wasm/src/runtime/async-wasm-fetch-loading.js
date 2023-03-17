__webpack_require__.v = (exports, wasmModuleId, wasmModuleHash, importsObj) => {
	var req = fetch(__webpack_require__.p + "" + wasmModuleHash + ".wasm");
	if (typeof WebAssembly.instantiateStreaming === "function") {
		return WebAssembly.instantiateStreaming(req, importsObj).then(res =>
			Object.assign(exports, res.instance.exports)
		);
	}
	return req
		.then(x => x.arrayBuffer())
		.then(bytes => WebAssembly.instantiate(bytes, importsObj))
		.then(res => Object.assign(exports, res.instance.exports));
};
