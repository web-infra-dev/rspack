const fs = require("fs");

it("should replace runtime variables and collect runtime requirements", () => {
	expect(__webpack_require__.runtimeCompat()).toEqual({
		publicPath: "runtime-public/",
		cache: true
	});

	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		const content = fs.readFileSync(__filename, "utf-8");
		const rspackContext = "__rspack" + "_context";
		const webpackModuleCache = "__webpack" + "_module_cache__";
		expect(content).toContain(
			`Object.defineProperty(${rspackContext}, "p"`
		);
		expect(content).toContain("publicPath =");
		expect(content).toContain("moduleCache.runtimeCompat");
		expect(content).toContain(`${rspackContext}.runtimeCompat`);
		expect(content).not.toContain(webpackModuleCache);
	}
});
