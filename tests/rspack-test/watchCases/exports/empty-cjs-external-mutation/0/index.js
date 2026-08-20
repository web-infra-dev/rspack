import * as barrel from "./barrel";

const findModule = name => __STATS__.modules.find(m => m.name.endsWith(`/${name}`));

it("should infer empty exports before an external mutation is added", () => {
	expect(Object.keys(barrel)).not.toContain("value");
	expect(findModule("empty.js").providedExports).toEqual([]);
});
