import fs from "node:fs";
import url from "node:url";
import path from "node:path";

export default function () {
	console.info("hello world");
}

export const add = (a, b) => {
	return a + b;
};

it("should run", function () {});

it("should export module library", function () {
	const __filename = url.fileURLToPath(import.meta.url);
	const source = fs.readFileSync(
		path.join(path.dirname(__filename), "dist/main.js"),
		"utf-8"
	);
	const exportedAdd = "__webpack_exports__add as add";
	const exportedDefault = "__webpack_exports__default as default";
	expect(source).toContain(`export { ${exportedAdd}, ${exportedDefault} }`);
});
