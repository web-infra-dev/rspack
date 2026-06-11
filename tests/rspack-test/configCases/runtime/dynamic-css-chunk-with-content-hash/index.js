it("load dynamic css chunk with content hash", async function () {
	await import("./dynamic").then(module => {
		expect(module.value).toBe("dynamic");
		// test is only for css loading
		if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
			if (typeof document !== "undefined" && __rspack_context.f.css) {
				expect(document.getElementsByTagName("link").length).toBe(1);
			}
		} else if (__webpack_require__.f.css) {
			expect(document.getElementsByTagName("link").length).toBe(1);
		}
	});
});
