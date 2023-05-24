it("basic", () => {
	const fs = require("fs");
	const source = fs.readFileSync(__filename + ".map", "utf-8");
	const map = JSON.parse(source);
	expect(map.sources).toContain("./index.js");
	expect(map.file).toEqual("main.js");
});
