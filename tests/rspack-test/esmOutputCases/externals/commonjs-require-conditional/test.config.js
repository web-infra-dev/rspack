const fs = require("fs");
const path = require("path");
const { execFileSync } = require("child_process");
const { createRequire } = require("module");
const { pathToFileURL } = require("url");

function runtimeRequire() {
	return createRequire(path.join(__dirname, "__external_runtime__.cjs"));
}

function readOutput(options, file) {
	return fs.readFileSync(path.join(options.output.path, file), "utf-8");
}

module.exports = {
	beforeExecute() {
		const require = runtimeRequire();
		delete require.cache[require.resolve("external")];
		process.__mixedExternalLoadCount = 0;
		process.__mixedExternalValues = [];
	},
	findBundle() {
		return ["safe.mjs", "unsafe.mjs"];
	},
	moduleScope(scope) {
		scope.require = runtimeRequire();
	},
	afterExecute(options) {
		const safe = readOutput(options, "safe.mjs");
		const unsafe = readOutput(options, "unsafe.mjs");
		const source = `${safe}\n${unsafe}`;

		expect(safe).toMatch(
			/module\.exports\s*=\s*require\s*\(\s*["']external["']\s*\)/
		);
		expect(safe).not.toContain('external "external"');
		expect(unsafe).toContain('external "external"');
		expect(unsafe).toMatch(
			/(?:__webpack_require__|__rspack_context\.r|rspackRequire)\s*\(\s*[^)]*["']external["']/
		);
		expect(source.match(/external "external"/g)).toHaveLength(1);

		expect(process.__mixedExternalLoadCount).toBe(1);
		expect(process.__mixedExternalValues).toHaveLength(2);
		expect(process.__mixedExternalValues[0]).toBe(
			process.__mixedExternalValues[1]
		);

		const requireBase = path.join(__dirname, "__external_runtime__.cjs");
		const safeUrl = pathToFileURL(path.join(options.output.path, "safe.mjs")).href;
		const unsafeUrl = pathToFileURL(
			path.join(options.output.path, "unsafe.mjs")
		).href;
		const reverseOrderScript = `
			import { createRequire } from "node:module";
			globalThis.require = createRequire(${JSON.stringify(requireBase)});
			process.__mixedExternalLoadCount = 0;
			process.__mixedExternalValues = [];
			await import(${JSON.stringify(unsafeUrl)});
			await import(${JSON.stringify(safeUrl)});
			if (
				process.__mixedExternalLoadCount !== 1 ||
				process.__mixedExternalValues.length !== 2 ||
				process.__mixedExternalValues[0] !== process.__mixedExternalValues[1]
			) {
				throw new Error("mixed direct and wrapped external should share identity");
			}
		`;
		execFileSync(
			process.execPath,
			["--input-type=module", "--eval", reverseOrderScript],
			{ stdio: "pipe" }
		);

		const require = runtimeRequire();
		delete require.cache[require.resolve("external")];
		delete process.__mixedExternalLoadCount;
		delete process.__mixedExternalValues;
	}
};
