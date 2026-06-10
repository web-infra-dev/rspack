var cjsRequire; // just to make it difficult
var cjsRequire = require, cjsRequire2 = typeof require !== "undefined" && require;

function test() {
	cjsRequire("./file");
}

function test2() {
	cjsRequire2("./file");
}

(function test3(cjsRequire3) {
	cjsRequire3("./file");
})(require);

test;
test2;
require;

it("should NOT rename require when requireAlias is false", function () {
	const fs = __non_webpack_require__("fs");
	const path = __non_webpack_require__("path");

	const content = fs.readFileSync(path.join(__dirname, "./bundle0.js"), "utf-8");
	const requireCall = globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK
		? "__rspack_context\\.r\\(239\\)"
		: "__webpack_require__\\(239\\)";
	const ok = "ok";

	expect(content).toMatch(new RegExp(`function test\\(\\) \\{\\s*${requireCall};\\s*\\}`, "i"));
	expect(content).toMatch(new RegExp(`function test2\\(\\) \\{\\s*${requireCall};\\s*\\}`, "i"));
	expect(content).toMatch(new RegExp(`function test3\\(cjsRequire3\\) \\{\\s*${requireCall};\\s*\\}`, "i"));
	expect(content).toContain(`var cjsRequire = undefined, cjsRequire2 = undefined;`);
	expect(content).toContain(`module.exports = "${ok}";`);
});
