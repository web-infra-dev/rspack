it("should have runtime __webpack_require__.f.j", async () => {
	// Don't really call it, it will cause error in runtime
	() => {
		__webpack_chunk_load__("main");
	};
	const path = __non_webpack_require__("path");
	const fs = __non_webpack_require__("fs");
	const code = await fs.promises.readFile(path.resolve(__dirname, "runtime.js"), "utf-8");
	const isRspackRuntime = typeof __rspack_context !== "undefined";
	expect(
			code.includes(
				isRspackRuntime
					? "ensureChunkHandlers.j"
					: "__webpack_require__.f.j"
			)
	).toBe(true);
})
