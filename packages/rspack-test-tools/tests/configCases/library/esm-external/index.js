import fs from "node:fs";
import path from "node:path";
import url from "node:url";

export default function () {
	console.info("hello world");
}

export const add = (a, b) => {
	return a + b;
};

it("should run", function () { });

it("should export module library", function () {
	const __filename = url.fileURLToPath(import.meta.url);
	const source = fs.readFileSync(
		path.join(
			__filename,
			"../../../../js/config/library/esm-external/bundle0.mjs"
		),
		"utf-8"
	);
	const createRequire = "__WEBPACK_EXTERNAL_createRequire";
	expect(source).toContain(`${createRequire}(import.meta.url)("node:fs")`);
});
