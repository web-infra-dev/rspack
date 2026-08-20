import * as barrel from "./barrel";

const findModule = name => __STATS__.modules.find(m => m.name.endsWith(`/${name}`));

it("should infer empty exports before module-cache access is added", () => {
	expect(Object.keys(barrel)).not.toContain("cacheValue");
	expect(findModule("empty.js").providedExports).toEqual([]);
});
