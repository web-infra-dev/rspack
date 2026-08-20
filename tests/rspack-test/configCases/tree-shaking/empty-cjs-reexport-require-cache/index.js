import { cacheValue } from "./barrel";

const findModule = name => __STATS__.modules.find(m => m.name.endsWith(`/${name}`));

it("should keep exports unknown when require.cache can mutate them", () => {
	expect(cacheValue).toBe("cache");
	expect(findModule("empty.js").providedExports).toBe(null);
});
