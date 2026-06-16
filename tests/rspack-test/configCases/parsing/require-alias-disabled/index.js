var cjsRequire; // just to make it difficult
var cjsRequire = require, cjsRequire2 = typeof require !== "undefined" && require;

function test() {
	cjsRequire("./file");
}

function test2() {
	cjsRequire2("./file");
}

try {
	(function test3(cjsRequire3) {
		cjsRequire3("./file");
	})(require);
} catch (e) {
	// do nothing
}


test;
test2;
require;

it("should NOT rename require when requireAlias is false", function () {
	const fs = __non_webpack_require__("fs");
	const path = __non_webpack_require__("path");

	const content = fs.readFileSync(path.join(__dirname, "./bundle0.js"), "utf-8");
	const filename = "./file";
	const ok = "ok";

	expect(content).toContain(`cjsRequire("${filename}")`);
	expect(content).toContain(`cjsRequire2("${filename}")`);
	expect(content).toContain(`cjsRequire3("${filename}")`);
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		expect(content).toContain(`var cjsRequire = __rspack_context.r(641), cjsRequire2 =  true && __rspack_context.r(641);`);
	} else {
		expect(content).toContain(`var cjsRequire = __webpack_require__(641), cjsRequire2 =  true && __webpack_require__(641);`);
	}
	expect(content).not.toContain(`module.exports = "${ok}";`);
});
