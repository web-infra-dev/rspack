import { load } from "./importer";
import plain from "./plain";

it("should bind filtered contexts while preserving regexes and reusing unrelated modules", async () => {
	expect((await load("a")).default).toBe("a");
	await expect(load("denied")).rejects.toThrow();
	expect(plain).toBe(42);
	if (COMPILER_INDEX === 0) {
		await NEXT_START(false);
	}
	if (COMPILER_INDEX === 1) {
		const stats = await NEXT_MOVE_DIR_START();
		const context = stats.modules.find(module => module.name.includes("eager"));
		expect(context.built).toBe(true);
		expect(context.identifier).toContain("|include: /allowed/|exclude: /denied/");
		// Only the context depends on the absolute-path filters. Its importer is rebuilt
		// through the missing-target path, without an unnecessary root binding of its own.
		expect(stats.logging["rspack.persistentCache"].entries.some(entry =>
			entry.message === "make persistent cache dropped 1 entries bound to another root"
		)).toBe(true);
		for (const name of ["plain.js"]) {
			expect(stats.modules.find(module => module.name.endsWith(name)).built).toBe(false);
		}
	}
	if (COMPILER_INDEX === 2) {
		const stats = await NEXT_START(false);
		expect(stats.modules.filter(module => module.built)).toHaveLength(0);
	}
	if (COMPILER_INDEX === 3) {
		const assets = READ_ASSETS();
		await NEXT_COLD_START();
		expect(READ_ASSETS()).toEqual(assets);
	}
});
