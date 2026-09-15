import fs from "node:fs";
import url from "node:url";
import path from "node:path";

export default function () {
	console.info("hello world");
}

export const add = (a, b) => {
	return a + b;
};

it("should run", function () { });

it("should export module library", function () {
	const __filename = url.fileURLToPath(import.meta.url);
	let outputPath;
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		outputPath = "../../../../js/runtime-mode-config/library/esm-external/bundle0.mjs";
	} else {
		outputPath = "../../../../js/config/library/esm-external/bundle0.mjs";
	}
	const source = fs.readFileSync(
		path.join(
			__filename,
			outputPath
		),
		"utf-8"
	);
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		expect(source).toContain('__rspack_context.r("node:fs")');
	} else {
		const createRequire = "__rspack_createRequire_require";
		expect(source).toContain(`${createRequire}("node:fs")`);
	}
});
