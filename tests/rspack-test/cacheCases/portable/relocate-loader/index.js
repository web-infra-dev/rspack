import path from "node:path";
import resourcePath from "./resource";

it("should rebuild loader output containing the writer root", async () => {
	expect(resourcePath).toBe(path.join(CURRENT_CONTEXT, "resource.js"));
	if (COMPILER_INDEX === 0) {
		const stats = await NEXT_START(false);
		expect(stats.modules.filter(module => module.built)).toHaveLength(0);
	}
	if (COMPILER_INDEX === 1) {
		const stats = await NEXT_MOVE_DIR_START();
		expect(stats.modules.some(module => module.name.endsWith("/resource.js") && module.built)).toBe(true);
		const entries = stats.logging["rspack.persistentCache"].entries;
		expect(entries.some(entry => /make persistent cache dropped [1-9]\d* entries bound to another root/.test(entry.message))).toBe(true);
	}
});
