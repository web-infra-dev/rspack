import { pathToFileURL } from "node:url";
import path from "node:path";
import { assetUrl } from "./url.mjs";

const resourceUrl = import.meta.url;

it("should rebuild root-sensitive modules after relocation and evict the old entries", async () => {
	expect(resourceUrl).toBe(pathToFileURL(path.join(CURRENT_CONTEXT, "index.js")).href);
	expect(assetUrl.href).toMatch(/^https:\/\/test\.cases\/path\/.+\.txt$/);
	if (COMPILER_INDEX === 0) {
		const stats = await NEXT_START(false);
		expect(stats.modules.filter(module => module.built)).toHaveLength(0);
	}
	if (COMPILER_INDEX === 1) {
		const stats = await NEXT_MOVE_DIR_START();
		const entries = stats.logging["rspack.persistentCache"].entries;
		expect(entries.some(entry => /make persistent cache dropped [1-9]\d* entries bound to another root/.test(entry.message))).toBe(true);
		expect(stats.modules.find(module => module.name.endsWith("/index.js")).built).toBe(true);
	}
	if (COMPILER_INDEX === 2) {
		const stats = await NEXT_START(false);
		const entries = stats.logging["rspack.persistentCache"].entries;
		expect(entries.filter(entry => /dropped \d+ entries bound to another root/.test(entry.message))).toEqual([]);
		expect(stats.modules.filter(module => module.built)).toHaveLength(0);
	}
});
