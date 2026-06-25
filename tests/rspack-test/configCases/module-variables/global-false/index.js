it("global false", function () {
	global;
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		expect(__rspack_context.g).toBe(undefined);
	} else {
		expect(__webpack_require__.g).toBe(undefined);
	}
});
