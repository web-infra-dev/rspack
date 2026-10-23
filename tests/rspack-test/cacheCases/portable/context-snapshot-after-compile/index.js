import fs from "node:fs";
import path from "node:path";

function load(name) {
	return require("./dir/" + name);
}

function generateDuringBuild(name) {
	fs.writeFileSync(path.join(CURRENT_CONTEXT, "generate"), name);
}

it("should keep the context snapshot read before afterCompile adds a child", async () => {
	expect(load("a")).toBe("a");
	if (COMPILER_INDEX === 0) {
		generateDuringBuild("b");
		const stats = await NEXT_START(false);
		expect(stats.modules.filter(module => module.built)).toHaveLength(0);
	}
	if (COMPILER_INDEX === 1) {
		expect(() => load("b")).toThrow();
		await NEXT_START(false);
	}
	if (COMPILER_INDEX === 2) {
		expect(load("b")).toBe("b");
		fs.writeFileSync(path.join(CURRENT_CONTEXT, "dir/c.js"), 'module.exports = "c";\n');
		generateDuringBuild("d");
		await NEXT_START(false);
	}
	if (COMPILER_INDEX === 3) {
		expect(load("c")).toBe("c");
		expect(() => load("d")).toThrow();
		await NEXT_START(false);
	}
	if (COMPILER_INDEX === 4) {
		expect(load("d")).toBe("d");
		const assets = READ_ASSETS();
		await NEXT_COLD_START();
		expect(READ_ASSETS()).toEqual(assets);
	}
});
