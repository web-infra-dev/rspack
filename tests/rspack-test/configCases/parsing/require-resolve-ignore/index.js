const fs = require("fs");
const path = require("path");

it("should be able to ignore require.resolve()", () => {
	const source = fs.readFileSync(path.join(__dirname, "bundle1.js"), "utf-8");
	expect(source.match(/\.resolve\(\/\* webpackIgnore: true \*\/ "node:fs"\)/g)).toHaveLength(3);
	expect(source.match(/\.resolve\(\/\* rspackIgnore: true \*\/ "node:fs"\)/g)).toHaveLength(3);
	expect(source).toContain('require.resolve(/* webpackIgnore: true */ "./non-exists")');
	expect(source).toContain('require.resolve(/* rspackIgnore: true */ "./non-exists")');
	expect(source).toContain('require("node:module")');

	const nativeRequire = process
		.getBuiltinModule("node:module")
		.createRequire(path.join(__dirname, "bundle0.js"));
	expect(() => nativeRequire("./bundle1.js")).not.toThrow();
});
