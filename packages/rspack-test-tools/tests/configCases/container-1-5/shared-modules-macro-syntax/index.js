it("should produce valid JavaScript syntax after macro processing", async () => {
	// Import modules to trigger module federation
	const cjsModule = await import("./cjs-module.js");
	const esmModule = await import("./esm-utils.js");

	// Basic functionality test
	expect(cjsModule.usedFunction()).toBe("This function is used");
	expect(esmModule.usedUtil()).toBe("This utility is used");

	// Note: Syntax validation happens in the afterBuild hook
});
