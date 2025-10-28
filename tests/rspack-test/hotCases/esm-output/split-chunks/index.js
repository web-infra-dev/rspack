import.meta.webpackHot.accept(["./common/shared", "vendor-lib"]);

it("should handle HMR with split chunks in ESM format", async () => {
	const [commonModule, vendorModule] = await Promise.all([
		import("./common/shared"),
		import("vendor-lib")
	]);
	expect(commonModule.commonFunction("test")).toBe("Common function processed: test");
	expect(vendorModule.default.version).toBe("1.0.0");
	await NEXT_HMR();

	const [updatedCommonModule, updatedVendorModule] = await Promise.all([
		import("./common/shared"),
		import("vendor-lib")
	]);
	expect(updatedCommonModule.commonFunction("test")).toBe("Updated common function: test");
	expect(updatedVendorModule.default.version).toBe("2.0.0");
});
