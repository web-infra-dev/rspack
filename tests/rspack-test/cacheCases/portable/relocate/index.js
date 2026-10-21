import { pathToFileURL } from "node:url";
import path from "node:path";
import { assetUrl } from "./url.mjs";

import fs from "node:fs";
import plain from "./plain";
import plainTwo from "./plain-two";

const resourceUrl = import.meta.url;

it("should rebuild root-sensitive modules after relocation and evict the old entries", async () => {
	expect(plain).toBe(42);
	expect(plainTwo).toBe(43);
	if (COMPILER_INDEX < 4) {
		expect(fs.readFileSync(path.join(CURRENT_CONTEXT, "build-counts"), "utf8").trim().split("\n").sort()).toEqual(["plain-two.js", "plain.js"]);
	}
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
	if (COMPILER_INDEX === 3) {
		const assets = READ_ASSETS();
		await NEXT_COLD_START();
		expect(READ_ASSETS()).toEqual(assets);
	}
});
