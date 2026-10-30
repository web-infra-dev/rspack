import fs from "node:fs";
import path from "node:path";

const context = require.context("./dir", false, /\.js$/);

function load(name) {
	return context("./" + name + ".js");
}

it("should reuse relocated require.context importers and context modules and invalidate renamed children", async () => {
	expect(load(COMPILER_INDEX < 2 ? "a" : "b")).toBe(42);
	if (COMPILER_INDEX === 0) {
		const stats = await NEXT_MOVE_DIR_START();
		expect(stats.modules.filter(module => module.built)).toHaveLength(0);
	}
	if (COMPILER_INDEX === 1) {
		fs.renameSync(path.join(CURRENT_CONTEXT, "dir/a.js"), path.join(CURRENT_CONTEXT, "dir/b.js"));
		const stats = await NEXT_START(false);
		expect(stats.modules.some(module => module.built && module.name.includes("sync"))).toBe(true);
	}
	if (COMPILER_INDEX === 2) {
		const stats = await NEXT_START(false);
		expect(stats.modules.filter(module => module.built)).toHaveLength(0);
		expect(stats.logging["rspack.persistentCache"].entries.some(entry =>
			entry.message === "snapshot restored with no changed dependencies"
		)).toBe(true);
	}
	if (COMPILER_INDEX === 3) {
		const assets = READ_ASSETS();
		await NEXT_COLD_START();
		expect(READ_ASSETS()).toEqual(assets);
	}
});
