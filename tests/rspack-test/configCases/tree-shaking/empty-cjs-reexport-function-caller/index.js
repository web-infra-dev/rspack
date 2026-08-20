import * as barrel from "./barrel";

const findModule = name => __STATS__.modules.find(m => m.name.endsWith(`/${name}`));

it("should keep exports unknown when function caller reflection exposes factory arguments", () => {
	expect(globalThis.functionCallerTargetLoaded).toBe(true);
	expect(barrel.functionCallerOwnValue).toBe("own exports");
	expect(findModule("empty.js").providedExports).toBe(null);
	expect(findModule("reflection.js").providedExports).toBe(null);
});
