import fs from "node:fs";
import path from "node:path";
import value from "./importer";

it("should preserve the snapshot validated before afterCompile changes an input", async () => {
	expect(value).toBe(COMPILER_INDEX < 3 ? "old" : "new");
	if (COMPILER_INDEX === 0 || COMPILER_INDEX === 3) {
		const stats = await NEXT_START(false);
		expect(stats.modules.filter(module => module.built)).toHaveLength(0);
	}
	if (COMPILER_INDEX === 1) {
		const input = path.join(CURRENT_CONTEXT, "generated.txt");
		const timestamp = new Date(Date.now() + 10000);
		fs.utimesSync(input, timestamp, timestamp);
		fs.writeFileSync(path.join(CURRENT_CONTEXT, "mutate-input"), "");
		const stats = await NEXT_START(false);
		expect(stats.modules.filter(module => module.built)).toHaveLength(0);
	}
	if (COMPILER_INDEX === 2) {
		expect(fs.readFileSync(path.join(CURRENT_CONTEXT, "generated.txt"), "utf8")).toBe("new\n");
		const stats = await NEXT_START(false);
		expect(stats.modules.some(module => module.built && module.name.includes("importer.js"))).toBe(true);
	}
	if (COMPILER_INDEX === 4) {
		const assets = READ_ASSETS();
		await NEXT_COLD_START();
		expect(READ_ASSETS()).toEqual(assets);
	}
});
