import "./mutator";
import "./empty";
import * as barrel from "./barrel";

const findModule = name => __STATS__.modules.find(m => m.name.endsWith(`/${name}`));

it("should keep empty exports unknown for the raw Rspack runtime binding", () => {
	expect(globalThis.rspackRequireValueTargetLoaded).toBe(true);
	expect(barrel.rspackRequireValue).toBe("rspack require");
	expect(findModule("empty.js").providedExports).toBe(null);
});
