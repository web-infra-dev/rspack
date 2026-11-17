import fs from "node:fs";
import path from "node:path";

function load(name) {
	return require("./dir/" + name);
}

function loadLazy(name) {
	return import("./dir/" + name);
}

function loadLazyOnce(name) {
	return import(/* webpackMode: "lazy-once" */ "./dir/" + name);
}

it("should reuse relocated sync, lazy, and lazy-once context modules and invalidate renamed children", async () => {
	expect(load(COMPILER_INDEX < 2 ? "a" : "b")).toBe(42);
	expect((await loadLazy(COMPILER_INDEX < 2 ? "a" : "b")).default).toBe(42);
	expect((await loadLazyOnce(COMPILER_INDEX < 2 ? "a" : "b")).default).toBe(42);
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
