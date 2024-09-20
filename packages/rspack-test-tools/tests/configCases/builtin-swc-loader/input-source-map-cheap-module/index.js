const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");
const checkMap = __non_webpack_require__("../../../../../dist/helper/util/checkSourceMap.js").default;

import "./a"

const source = fs.readFileSync(__filename + ".map", "utf-8");
const map = JSON.parse(source);
const output = fs.readFileSync(__filename, "utf-8");
const input = fs.readFileSync(path.resolve(CONTEXT, "a.jsx"), "utf-8");

it("should keep the original content with `devtool: \"cheap-module-source-map\"` enabled", () => {
	expect(map.sources).toEqual([
		"webpack:///./a.jsx",
		"webpack:///./index.js",
	]);
	expect(map.sourcesContent[0]).toEqual(input)
})
it("should keep the mappings to the original content", async () => {
	// does not checking columns for cheap source-map
	const CHECK_COLUMN = false;
	expect(await checkMap(output, source, {
		"'*a0*'": "webpack:///a.jsx",
		"'*a1*'": "webpack:///a.jsx",
		"'*a2*'": "webpack:///a.jsx",
	}, CHECK_COLUMN)).toBe(true)
})

