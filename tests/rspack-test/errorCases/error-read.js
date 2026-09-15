let mappedErrors = [];
let mappedErrorsAfterWrite = [];

/** @type {import('@rspack/test-tools').TErrorCaseConfig[]} */
module.exports = [
	{
		description: "Testing map function on errors: test map of errors",
		options() {
			mappedErrors = [];
			return {
				entry: "./resolve-fail-esm",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap("test errors map", compilation => {
							mappedErrors = compilation.errors.map((item, index) => {
								item.index = index;
								return item;
							});
						});
					}
				]
			};
		},
		async check() {
			expect(mappedErrors).toMatchInlineSnapshot(`Array []`);
		}
	},
	{
		description:
			"Testing map function on errors: test map of errors after write",
		options() {
			mappedErrorsAfterWrite = [];
			return {
				entry: "./resolve-fail-esm",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap("test errors map", compilation => {
							compilation.errors.push(new Error(""));
							compilation.errors = compilation.errors.filter(
								item => item.message
							);

							mappedErrorsAfterWrite = compilation.errors.map((item, index) => {
								item.index = index;
								return item;
							});
						});
					}
				]
			};
		},
		async check() {
			expect(mappedErrorsAfterWrite).toMatchInlineSnapshot(`
				Array [
				  Object {
				  "index": 0,
				  "message": "  × ",
				  "name": "Error",
				  "stack": "Error:     at Object.fn (<TEST_ROOT>/errorCases/error-read.js<LINE_COL>)    at next (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsyncStageRange (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>    at new Promise (<anonymous>)    at AsyncSeriesHook.promiseStageRange (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at QueriedHook.promise (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>    at last.function (<RSPACK_ROOT>/dist/index.js<LINE_COL>)",
				},
				]
			`);
		}
	}
];
