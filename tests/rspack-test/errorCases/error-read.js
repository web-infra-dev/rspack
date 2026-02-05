let errors = [];

/** @type {import('@rspack/test-tools').TErrorCaseConfig[]} */
module.exports = [
	{
		description:
			"Testing map function on errors and warnings: test map of errors",
		options() {
			errors = [];
			return {
				entry: "./resolve-fail-esm",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap("test errors map", compilation => {
							errors = compilation.errors.map((item, index) => {
								item.index = index;
								return item;
							});
						});
					}
				]
			};
		},
		async check() {
			expect(errors).toMatchInlineSnapshot(`Array []`);
		}
	},
	{
		description:
			"Testing map function on errors and warnings: test map of errors",
		options() {
			errors = [];
			return {
				entry: "./resolve-fail-esm",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap("test errors map", compilation => {
							compilation.errors.push(new Error(""));
							compilation.errors = compilation.errors.filter(
								item => item.message
							);

							errors = compilation.errors.map((item, index) => {
								item.index = index;
								return item;
							});
						});
					}
				]
			};
		},
		async check() {
			expect(errors).toMatchInlineSnapshot(`
				Array [
				  Object {
				  "index": 0,
				  "message": "  × ",
				  "name": "Error",
				  "stack": "Error:     at <TEST_ROOT>/errorCases/error-read.js<LINE_COL>    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at next (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsyncStageRange (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsync (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
				},
				]
			`);
		}
	}
];
