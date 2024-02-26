it("should not have map from url comments if hidden options is used", function () {
	const fs = require("fs");
	const source = fs.readFileSync(__filename, "utf-8");
	expect(/sourceMappingURL\s*=\s*(.*)/.test(source)).toBe(false);
	const mapSource = fs.readFileSync(__filename + ".map", "utf-8");
	const map = JSON.parse(mapSource);
	expect(map.sources).toContain("webpack:///./index.js");
	expect(map.file).toEqual("bundle0.js");
});
