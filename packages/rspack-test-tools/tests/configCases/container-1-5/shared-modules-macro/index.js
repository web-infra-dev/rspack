it("should work with shared modules in Module Federation", async () => {
	// Test CJS module imports
	const cjsModule = await import("./cjs-module.js");
	expect(cjsModule.usedFunction()).toBe("This function is used");
	expect(cjsModule.usedConstant).toBe("used constant");
	expect(typeof cjsModule.createObject()).toBe("object");
	expect(cjsModule.processCjsData("test")).toBe("processed: test");
	
	// Test ESM module imports
	const esmModule = await import("./esm-utils.js");
	expect(esmModule.usedUtil()).toBe("This utility is used");
	expect(esmModule.processEsmData({ value: 10 })).toBe(20); // pureFunction(1) returns 2, so 10 * 2 = 20
	expect(esmModule.validateData("test")).toBe(true);
	
	// Test mixed exports module
	const mixedModule = await import("./mixed-exports.js");
	expect(mixedModule.namedExport).toBe("named value");
	expect(mixedModule.default.defaultValue).toBe("default export value");
	
	// Test pure helper module
	const pureHelper = await import("./pure-helper.js");
	expect(pureHelper.pureFunction(5)).toBe(10);
	expect(pureHelper.PURE_CONSTANT).toBe("pure constant value");
	
	// Note: PURE annotations and tree-shaking macros are validated in the afterBuild hook
});