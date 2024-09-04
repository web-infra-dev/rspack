const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");
const checkMap = __non_webpack_require__("../../../../../dist/helper/util/checkSourceMap.js").default;

import "./a"

const source = fs.readFileSync(__filename + ".map", "utf-8");
const map = JSON.parse(source);
const output = fs.readFileSync(__filename, "utf-8");
const input = fs.readFileSync(path.resolve(CONTEXT, "a.jsx"), "utf-8");

it("should keep the original content with `devtool: \"source-map\"` enabled", () => {
	expect(map.sources).toEqual([
		"webpack:///./a.jsx",
		"webpack:///./index.js",
	]);
	expect(map.sourcesContent[0]).toEqual(input)
})

it("should keep the mappings to the original content", async () => {
	expect(await checkMap(output, source, {
		"'*a0*'": "webpack:///a.jsx",
		"'*a1*'": "webpack:///a.jsx",
		"'*a2*'": "webpack:///a.jsx",
	})).toBe(true)
})

