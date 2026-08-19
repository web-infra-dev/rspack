const fs = require("fs");
const path = require("path");
const { createRequire } = require("module");

function runtimeRequire() {
	return createRequire(path.join(__dirname, "__external_runtime__.cjs"));
}

module.exports = {
	moduleScope(scope) {
		scope.require = runtimeRequire();
	},
	afterExecute(options) {
		const source = fs.readFileSync(
			path.join(options.output.path, "main.mjs"),
			"utf-8"
		);

		expect(source).toContain('external ["external","value"]');
		expect(source).toMatch(
			/(?:__webpack_require__|__rspack_context\.r|rspackRequire)\s*\(\s*[^)]*["']external-value["']/
		);
	}
};
