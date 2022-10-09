it("should not include sourcesContent if noSources option is used", () => {
	const fs = require("fs");
	const source = fs.readFileSync(__filename + ".map", "utf-8");
	const map = JSON.parse(source);
	expect(map).not.toHaveProperty("sourcesContent");
});
