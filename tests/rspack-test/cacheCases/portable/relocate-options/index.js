import fs from "node:fs";
import path from "node:path";
import value from "./resource";

it("should reuse a loader with an absolute in-root option", async () => {
	expect(value).toBe("portable value");
	if (COMPILER_INDEX < 2) {
		expect(fs.readFileSync(path.join(CURRENT_CONTEXT, "build-counts"), "utf8")).toBe("built\n");
	}
	if (COMPILER_INDEX === 0) {
		const stats = await NEXT_MOVE_DIR_START();
		expect(stats.modules.filter(module => module.built)).toHaveLength(0);
	}
	if (COMPILER_INDEX === 1) {
		const assets = READ_ASSETS();
		await NEXT_COLD_START();
		expect(READ_ASSETS()).toEqual(assets);
	}
});
