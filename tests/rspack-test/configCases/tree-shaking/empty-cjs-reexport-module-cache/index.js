import "./empty";

const runtimeModuleCache =
	typeof __webpack_module_cache__ !== "undefined"
		? __webpack_module_cache__
		: __rspack_module_cache;
runtimeModuleCache["./empty.js"].exports.moduleCacheValue = "module cache";

const findModule = name => __STATS__.modules.find(m => m.name.endsWith(`/${name}`));

it("should keep empty exports unknown when the runtime module cache can mutate them", async () => {
	const barrel = await import("./barrel");

	expect(globalThis.moduleCacheTargetLoaded).toBe(true);
	expect(barrel.moduleCacheValue).toBe("module cache");
	expect(findModule("empty.js").providedExports).toBe(null);
});
