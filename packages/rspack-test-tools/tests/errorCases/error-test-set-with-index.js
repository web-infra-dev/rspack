/** @type {import('../..').TErrorCaseConfig} */
module.exports = {
	description: "Testing set errors",
	options() {
		return {
			entry: "./resolve-fail-esm",
			plugins: [
				compiler => {
					compiler.hooks.afterCompile.tap("set errors", compilation => {
						compilation.errors[0] = new Error("error 1");
						expect(compilation.errors[0].message).toMatch(/error 1/);
						expect(compilation.errors[1]).toBe(undefined);
					});
				}
			]
		};
	},
	async check(diagnostics) {
		expect(diagnostics).toMatchInlineSnapshot(`
		Object {
		  "errors": Array [
		    Object {
		      "code": "Error",
		      "message": "  × Error: error 1\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n",
		      "moduleTrace": Array [],
		      "stack": "Error: error 1\\n    at <TEST_TOOLS_ROOT>/tests/errorCases/error-test-set-with-index.js<LINE_COL>\\n    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)\\n    at next (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsyncStageRange (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsync (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
		    },
		  ],
		  "warnings": Array [],
		}
	`);
	}
};
