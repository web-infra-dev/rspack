import "./amd-module";

it("should keep AMD module pseudo-dependencies conservative", async () => {
	await Promise.resolve();
	const values = await import("./barrel");

	expect(values.fromAmdRequire).toBe("amd-require");
	const amdModule = __STATS__.modules.find(module =>
		module.name.endsWith("/amd-module.js")
	);
	expect(amdModule.providedExports).toBe(null);
});
