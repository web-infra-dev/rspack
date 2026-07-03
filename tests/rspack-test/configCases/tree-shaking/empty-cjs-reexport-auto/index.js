export * from "./barrel";

import "./access-exports";
import "./access-module";
import "./access-this";
import "./access-eval";

const findModule = name => __STATS__.modules.find(m => m.name.endsWith(name));

it("should infer empty exports for auto modules without cjs export access", () => {
	expect(findModule("empty.ts").providedExports).toEqual([]);
	expect(findModule("barrel.js").providedExports).toEqual(["live"]);
});

it("should keep unknown exports when cjs export access is observed", () => {
	expect(findModule("access-exports.js").providedExports).toBe(null);
	expect(findModule("access-module.js").providedExports).toBe(null);
	expect(findModule("access-this.js").providedExports).toBe(null);
	expect(findModule("access-eval.js").providedExports).toBe(null);
});
