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
		process.__commonjsExternalLoadCount = 0;
		process.__commonjsExternalValues = [];
	},
	findBundle() {
		return ["entry-a.mjs", "entry-b.mjs"];
	},
	moduleScope(scope) {
		scope.require = runtimeRequire();
	},
	afterExecute(options) {
		const entryA = readOutput(options, "entry-a.mjs");
		const entryB = readOutput(options, "entry-b.mjs");
		const source = `${entryA}\n${entryB}`;

		expect(entryA).toMatch(
			/const\s+first\s*=\s*require\s*\(\s*["']external["']\s*\)/
		);
		expect(entryB).toMatch(
			/module\.exports\s*=\s*require\s*\(\s*["']external["']\s*\)/
		);
		expect(source).not.toContain('external "external"');
		expect(source).not.toMatch(
			/(?:__webpack_require__|__rspack_context\.r|rspackRequire)\s*\(\s*[^)]*["']external["']/
		);

		expect(process.__commonjsExternalLoadCount).toBe(1);
		expect(process.__commonjsExternalValues).toHaveLength(6);
		for (const value of process.__commonjsExternalValues) {
			expect(value).toBe(process.__commonjsExternalValues[0]);
		}

		const requireBase = path.join(__dirname, "__external_runtime__.cjs");
		const entryAUrl = pathToFileURL(path.join(options.output.path, "entry-a.mjs")).href;
		const entryBUrl = pathToFileURL(path.join(options.output.path, "entry-b.mjs")).href;
		const reverseOrderScript = `
			import { createRequire } from "node:module";
			globalThis.require = createRequire(${JSON.stringify(requireBase)});
			process.__commonjsExternalLoadCount = 0;
			process.__commonjsExternalValues = [];
			await import(${JSON.stringify(entryBUrl)});
			await import(${JSON.stringify(entryAUrl)});
			if (process.__commonjsExternalLoadCount !== 1) {
				throw new Error("external should initialize once in reverse entry order");
			}
			if (
				process.__commonjsExternalValues.length !== 6 ||
				!process.__commonjsExternalValues.every(
					value => value === process.__commonjsExternalValues[0]
				)
			) {
				throw new Error("external identity should be shared in reverse entry order");
			}
		`;
		execFileSync(
			process.execPath,
			["--input-type=module", "--eval", reverseOrderScript],
			{ stdio: "pipe" }
		);

		const require = runtimeRequire();
		delete require.cache[require.resolve("external")];
		delete process.__commonjsExternalLoadCount;
		delete process.__commonjsExternalValues;
	}
};
