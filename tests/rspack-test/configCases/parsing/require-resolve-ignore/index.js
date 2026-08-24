const fs = require("fs");
const path = require("path");

it("should be able to ignore require.resolve()", () => {
	const source = fs.readFileSync(path.join(__dirname, "bundle1.js"), "utf-8");
	expect(source.match(/\.resolve\(\/\* webpackIgnore: true \*\/ "\.\/non-exists"\)/g)).toHaveLength(3);
	expect(source.match(/\.resolve\(\/\* rspackIgnore: true \*\/ "\.\/non-exists"\)/g)).toHaveLength(3);
});
