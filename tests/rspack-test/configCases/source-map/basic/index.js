import data, { name, nested } from "./data.json";
import { helper } from "./helper.js";

it("basic", () => {
	const fs = require("fs");
	const source = fs.readFileSync(__filename + ".map", "utf-8");
	const map = JSON.parse(source);
	let sourceUrl = source => `webpack:///${source}`;
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		sourceUrl = source => `rspack:///${source}`;
	}
	const indexSource = map.sources.indexOf(sourceUrl("./index.js"));
	const helperSource = map.sources.indexOf(sourceUrl("./helper.js"));
	expect(indexSource).toBeGreaterThanOrEqual(0);
	expect(helperSource).toBeGreaterThanOrEqual(0);
	expect(map.sources.some(source => source.includes("data.json"))).toBe(false);
	expect(map.sourcesContent[indexSource]).toContain("ordinaryObjectUnaffected");
	expect(map.sourcesContent[helperSource]).toContain("helper:");
	expect(map.file).toEqual(require("path").basename(__filename));

	expect(data.name).toBe("synthetic");
	expect(name).toBe("synthetic");
	expect(nested.value).toBe(7);
	expect(helper(name)).toBe("helper:synthetic");
	expect(Object.prototype.hasOwnProperty.call(data, "__proto__")).toBe(true);
	expect(data.__proto__.safe).toBe(true);
	const ordinaryObjectUnaffected = Object.prototype.safe === undefined;
	expect(ordinaryObjectUnaffected).toBe(true);
});
