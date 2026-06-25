it("should load css chunk", async function () {
	const module = await import("./share");
	expect(module.value).toBe(1);
	// test is only for css loading
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		if (typeof document !== "undefined" && __rspack_context.f.css) {
			expect(document.getElementsByTagName("link").length).toBe(2);
		}
	} else if (__webpack_require__.f.css) {
		expect(document.getElementsByTagName("link").length).toBe(2);
	}
});
