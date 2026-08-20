import "./empty";

eval(`
	const runtimeModuleCache =
		typeof __webpack_module_cache__ !== "undefined"
			? __webpack_module_cache__
			: __rspack_module_cache;
	runtimeModuleCache["./empty.js"].exports.evalValue = "eval";
`);

const findModule = name => __STATS__.modules.find(m => m.name.endsWith(`/${name}`));

it("should keep empty exports unknown when direct eval can mutate the module cache", async () => {
	const barrel = await import("./barrel");

	expect(globalThis.evalTargetLoaded).toBe(true);
	expect(barrel.evalValue).toBe("eval");
	expect(findModule("empty.js").providedExports).toBe(null);
});
