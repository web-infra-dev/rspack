var cjsRequire; // just to make it difficult
var cjsRequire = require, cjsRequire2 = typeof require !== "undefined" && require;

function test() {
	cjsRequire("./file");
}

function test2() {
	cjsRequire2("./file");
}

test;
test2;
require;

it("should NOT rename require when requireAlias is false", function () {
	const fs = __non_webpack_require__("fs");
	const path = __non_webpack_require__("path");

	const content = fs.readFileSync(path.join(__dirname, "./bundle0.js"), "utf-8");
	const requireName = "undefined";
	const ok = "ok";

	expect(content).toMatch(/function test\(\) \{\s*__webpack_require__\(239\);\s*\}/i);
	expect(content).toMatch(/function test2\(\) \{\s*__webpack_require__\(239\);\s*\}/i);
	expect(content).toContain(`var cjsRequire = ${requireName}, cjsRequire2 = ${requireName};`);
	expect(content).toContain(`module.exports = "${ok}";`);
});