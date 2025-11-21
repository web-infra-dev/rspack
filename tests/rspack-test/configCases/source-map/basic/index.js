it("basic", () => {
	const fs = require("fs");
	const source = fs.readFileSync(__filename + ".map", "utf-8");
	const map = JSON.parse(source);
	expect(map.sources).toContain("webpack:///./index.js");
	expect(map.file).toEqual("bundle0.js");
});
