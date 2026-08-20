import { cacheValue } from "./barrel";

const findModule = name => __STATS__.modules.find(m => m.name.endsWith(`/${name}`));

it("should keep exports unknown when createRequire.cache can mutate them", () => {
	expect(cacheValue).toBe("create-require-cache");
	expect(findModule("empty.js").providedExports).toBe(null);
});
