import missing from "./module.js?missing";
import explicitNull from "./module.js?null";
import empty from "./module.js?empty";
import text from "./module.js?text";
import fs from "node:fs";

it("preserves missing, null, empty and present loader source contents", () => {
	expect([missing, explicitNull, empty, text]).toEqual([42, 42, 42, 42]);
	const map = JSON.parse(fs.readFileSync(__filename + ".map", "utf8"));
	for (const [kind, content] of [
		["missing", null],
		["null", null],
		["empty", ""],
		["text", "export default 42;\n"]
	]) {
		const index = map.sources.findIndex(source => source.endsWith(`/original-${kind}.js`));
		expect(index).toBeGreaterThanOrEqual(0);
		expect(map.sourcesContent[index]).toBe(content);
	}
});
