let warnings = [];

/** @type {import('../..').TErrorCaseConfig} */
module.exports = {
	description:
		"Testing map function on errors and warnings: test map of warnings",
	options() {
		return {
			entry: "./require.main.require",
			plugins: [
				compiler => {
					compiler.hooks.afterCompile.tap("test warnings map", compilation => {
						warnings = compilation.warnings.map((item, index) => ({
							index,
							...item
						}));
					});
				}
			]
		};
	},
	async check() {
		expect(warnings).toMatchInlineSnapshot(`
		Array [
		  Object {
		  "index": 0,
		  "message": "  ⚠ Module parse warning:\\n  ╰─▶   ⚠ Module parse failed: require.main.require() is not supported by Rspack.\\n         ╭────\\n       1 │ require.main.require('./file');\\n         · ──────────────────────────────\\n         ╰────\\n      \\n",
		  "moduleId": "./require.main.require.js",
		  "moduleIdentifier": "<cwd>packages/rspack-test-tools/tests/fixtures/errors/require.main.require.js",
		  "moduleName": "./require.main.require.js",
		},
		]
	`);
	}
};
