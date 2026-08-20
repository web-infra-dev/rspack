import "./empty";

__rspack_context.r("./empty.js").runtimeContextValue = "runtime context";

const findModule = name => __STATS__.modules.find(m => m.name.endsWith(`/${name}`));

it("should keep empty exports unknown when the rspack runtime context can mutate them", async () => {
	const barrel = await import("./barrel");

	expect(globalThis.runtimeContextTargetLoaded).toBe(true);
	expect(barrel.runtimeContextValue).toBe("runtime context");
	expect(findModule("empty.js").providedExports).toBe(null);
});
