const fs = require("fs");
const path = require("path");
const { createRequire } = require("module");

function runtimeRequire() {
	return createRequire(path.join(__dirname, "__external_runtime__.cjs"));
}

module.exports = {
	beforeExecute() {
		const require = runtimeRequire();
		delete require.cache[require.resolve("optional-external")];
	},
	moduleScope(scope) {
		scope.require = runtimeRequire();
	},
	afterExecute(options) {
		const source = fs.readFileSync(
			path.join(options.output.path, "main.mjs"),
			"utf-8"
		);

		expect(source).toContain('external "optional-external"');
		expect(source).toContain('external "missing-external"');
		expect(source).toMatch(
			/(?:__webpack_require__|__rspack_context\.r|rspackRequire)\s*\(\s*[^)]*["']optional-external["']/
		);
		expect(source).toMatch(
			/(?:__webpack_require__|__rspack_context\.r|rspackRequire)\s*\(\s*[^)]*["']missing-external["']/
		);
		expect(source).not.toMatch(
			/value\s*=\s*require\s*\(\s*["']optional-external["']\s*\)/
		);

		const require = runtimeRequire();
		delete require.cache[require.resolve("optional-external")];
	}
};
