import { cacheValue } from "./barrel";

const findModule = name => __STATS__.modules.find(m => m.name.endsWith(`/${name}`));

it("should invalidate empty exports when module-cache access is added", () => {
	expect(cacheValue).toBe("cache");
	expect(findModule("empty.js").providedExports).toBe(null);
});
