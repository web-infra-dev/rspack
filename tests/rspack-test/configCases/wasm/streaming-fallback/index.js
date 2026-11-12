it("should fall back when the wasm MIME type is wrong", function() {
	return import("./module").then(module => {
		expect(module.run()).toEqual(42);
		expect(window.__wasmStreamingFallbackWarnings).toHaveLength(1);
		expect(window.__wasmStreamingFallbackWarnings[0][0]).toContain(
			"application/wasm"
		);
	});
});
