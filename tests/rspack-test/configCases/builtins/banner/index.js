const fs = require("fs");
const path = require("path");
import png from "./empty.png";
import("./b.js").then(res => {
	// xxxx
});

it("add comment should works", () => {
	const mainFile = fs.readFileSync(__filename, "utf-8");
	expect(mainFile.startsWith("/*! MMMMMMM */")).toBeTruthy();
	expect(
		mainFile.endsWith("/** MMMMMMM */\n//# sourceMappingURL=main.js.map")
	).toBeTruthy();

	const aFile = fs.readFileSync(path.resolve(__dirname, "a.js"), "utf-8");
	expect(aFile.startsWith("/*! MMMMMMM */")).toBeTruthy();
	expect(
		aFile.endsWith("/** MMMMMMM */\n//# sourceMappingURL=a.js.map")
	).toBeFalsy();

	const asyncFile = fs.readFileSync(
		path.resolve(__dirname, "b_js.js"),
		"utf-8"
	);
	expect(asyncFile.startsWith("/*! MMMMMMM */")).toBeTruthy();
	expect(
		asyncFile.endsWith("/** MMMMMMM */\n//# sourceMappingURL=b_js.js.map")
	).toBeFalsy();
});

it("should keep source map", () => {
	expect(fs.existsSync(path.resolve(__dirname, "main.js.map"))).toBe(true);
});

it("should not inject placeholder to asset", () => {
	const pngContent = fs.readFileSync(
		path.resolve(__dirname, "./empty.png"),
		"utf-8"
	);
	expect(pngContent.startsWith("/*! MMMMMMM */")).toBeFalsy();
});
