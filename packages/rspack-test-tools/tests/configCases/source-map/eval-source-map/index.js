const path = require("path");

it("basic", () => {
	const fs = require("fs");
	const source = fs.readFileSync(__filename, "utf-8");
	const base64 =
		/sourceMappingURL\s*=\s*data:application\/json;charset=utf-8;base64,(.*)\\n\/\/#/.exec(
			source
		)[1];
	const map = JSON.parse(Buffer.from(base64, "base64").toString("utf-8"));
	expect(source.includes('//# sourceURL=webpack-internal:///./index.js'))
	expect(map.sources[0]).toMatch(/webpack:\/\/\/\.\/index.js?[a-zA-Z0-9]+/);
	expect(map.file).toBe(path.join(CONTEXT, "index.js"));
});
