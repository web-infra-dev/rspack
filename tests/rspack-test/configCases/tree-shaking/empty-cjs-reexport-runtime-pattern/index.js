import "./mutator";
import "./empty";
import * as barrel from "./barrel";

const findModule = name => __STATS__.modules.find(m => m.name.endsWith(`/${name}`));

it("should keep empty exports unknown when a pattern can replace the runtime cache", () => {
	expect(globalThis.runtimePatternTargetLoaded).toBe(true);
	expect(findModule("empty.js").providedExports).toBe(null);
});
