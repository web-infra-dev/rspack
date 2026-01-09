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

it("should NOT rename require when requireRename is false", function () {
	const fs = __non_webpack_require__("fs");
	const path = __non_webpack_require__("path");

	const content = fs.readFileSync(path.join(__dirname, "./bundle0.js"), "utf-8");
	const filename = "./file";
	const requireName = "require";

	expect(content).toContain(`cjsRequire("${filename}")`);
	expect(content).toContain(`cjsRequire2("${filename}")`);
	expect(content).toContain(`var cjsRequire = ${requireName}, cjsRequire2 =  true && ${requireName};`);
});