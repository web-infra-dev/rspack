import { pathToFileURL } from "node:url";
import path from "node:path";

const resourceUrl = import.meta.url;

it("should rebuild root-sensitive modules after relocation", async () => {
	expect(resourceUrl).toBe(pathToFileURL(path.join(CURRENT_CONTEXT, "index.js")).href);
	if (COMPILER_INDEX === 0) {
		const stats = await NEXT_START(false);
		expect(stats.modules.filter(module => module.built)).toHaveLength(0);
	}
	if (COMPILER_INDEX === 1) {
		const stats = await NEXT_MOVE_DIR_START();
		const entries = stats.logging["rspack.persistentCache"].entries;
		expect(entries.some(entry => /make persistent cache dropped [1-9]\d* entries bound to another root/.test(entry.message))).toBe(true);
		expect(stats.modules.some(module => module.built)).toBe(true);
	}
});
