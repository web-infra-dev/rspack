const fs = require("fs");
const path = require("path");

import("./b.js").then(res => {
	// xxxx
});

it("add comment should works", () => {
	const content = fs.readFileSync(__filename, "utf-8");
	expect(content.endsWith("/** MMMMMMM */")).toBeTruthy();

	const asyncFile = fs.readFileSync(
		path.resolve(__dirname, "b_js.js"),
		"utf-8"
	);
	expect(asyncFile.endsWith("/** MMMMMMM */")).toBeFalsy();

	const aFile = fs.readFileSync(path.resolve(__dirname, "a.js"), "utf-8");
	expect(aFile.endsWith("/** MMMMMMM */")).toBeFalsy();
});
