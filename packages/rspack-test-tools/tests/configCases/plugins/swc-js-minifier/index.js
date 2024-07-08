const fs = require("fs");
const path = require("path");

it("should remove extracted comments and insert a banner", function () {
	const source = fs.readFileSync(path.join(__dirname, "extract.js"), "utf-8");
	expect(source).not.toMatch("comment should be extracted extract-test.1");
	expect(source).not.toMatch("comment should be stripped extract-test.2");
	expect(source).not.toMatch("comment should be extracted extract-test.3");
	expect(source).not.toMatch("comment should be stripped extract-test.4");
	expect(source).toMatch(
		"/*! For license information please see extract.js.LICENSE.txt */"
	);
});

it("should not generated comment banner when there no comments", function () {
	const dir = fs.readdirSync(__dirname);
	expect(dir).toStrictEqual([
		"extract.js",
		"extract.js.LICENSE.txt",
		"main.js",
		"no-extract.js",
		"stats.json",
		"stats.txt"
	]);
});
