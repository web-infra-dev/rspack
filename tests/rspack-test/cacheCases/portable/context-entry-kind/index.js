import fs from "node:fs";
import path from "node:path";

function load(name) {
	return require("./dir/" + name);
}

it("should invalidate context modules when a child changes kind", async () => {
	const dir = path.join(CURRENT_CONTEXT, "dir");
	const other = path.join(CURRENT_CONTEXT, "other");
	if (COMPILER_INDEX === 0) {
		expect(load("x")).toBe("./dir/x/index.js");
		// Same files behind a symlink: only the entry kind tells the two apart.
		fs.mkdirSync(other);
		fs.renameSync(path.join(dir, "x"), path.join(other, "x"));
		fs.symlinkSync(path.join(other, "x"), path.join(dir, "x"));
		const stats = await NEXT_START(false);
		expect(stats.modules.some(module => module.built && module.name.includes("sync"))).toBe(true);
	}
	if (COMPILER_INDEX === 1) {
		expect(load("x")).toBe("./other/x/index.js");
		fs.unlinkSync(path.join(dir, "x"));
		fs.renameSync(path.join(other, "x"), path.join(dir, "x"));
		const stats = await NEXT_START(false);
		expect(stats.modules.some(module => module.built && module.name.includes("sync"))).toBe(true);
	}
	if (COMPILER_INDEX === 2) {
		expect(load("x")).toBe("./dir/x/index.js");
		expect(load("y")).toEqual({});
		fs.rmSync(path.join(dir, "y"));
		fs.mkdirSync(path.join(dir, "y"));
		fs.writeFileSync(path.join(dir, "y/index.js"), "module.exports = module.id;\n");
		await NEXT_START(false);
	}
	if (COMPILER_INDEX === 3) {
		expect(load("y")).toBe("./dir/y/index.js");
		expect(load("z")).toEqual({});
		// An empty file and an empty directory hash alike without the entry kind.
		fs.rmSync(path.join(dir, "z"));
		fs.mkdirSync(path.join(dir, "z"));
		const stats = await NEXT_START(false);
		expect(stats.modules.some(module => module.built && module.name.includes("sync"))).toBe(true);
	}
	if (COMPILER_INDEX === 4) {
		expect(() => load("z")).toThrow();
		const assets = READ_ASSETS();
		await NEXT_COLD_START();
		expect(READ_ASSETS()).toEqual(assets);
	}
});
