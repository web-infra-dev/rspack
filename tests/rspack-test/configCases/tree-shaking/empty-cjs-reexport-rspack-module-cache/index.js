import "./empty";
import "./mutator";
import * as barrel from "./barrel";

const findModule = name => __STATS__.modules.find(m => m.name.endsWith(`/${name}`));

it("should keep empty exports unknown for the raw Rspack runtime binding", () => {
	expect(globalThis.rspackModuleCacheValueTargetLoaded).toBe(true);
	expect(barrel.rspackModuleCacheValue).toBe("rspack module cache");
	expect(findModule("empty.js").providedExports).toBe(null);
});
