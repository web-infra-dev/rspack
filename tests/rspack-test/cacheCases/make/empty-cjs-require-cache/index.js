import { cacheValue } from "./barrel";

it("should restore untracked module-exports access from persistent cache", async () => {
	expect(cacheValue).toBe("cache");
	expect(globalThis.persistentCacheTargetVersion).toBe(COMPILER_INDEX);
	if (COMPILER_INDEX === 0) {
		await NEXT_START();
	}
});
