/** @type {import('@rspack/test-tools').TErrorCaseConfig} */
module.exports = {
	description: "Testing set warnings",
	options() {
		return {
			entry: "./require.main.require",
			plugins: [
				compiler => {
					compiler.hooks.afterCompile.tap("set warnings", compilation => {
						compilation.warnings = [
							new Error("warning 1"),
							new Error("warning 2")
						];
					});
				}
			]
		};
	},
	async check(diagnostics) {
		expect(diagnostics).toMatchInlineSnapshot(`
		Object {
		  "errors": Array [],
		  "warnings": Array [
		    Object {
		      "code": "Error",
		      "message": "  ⚠ warning 1\\n",
		      "moduleTrace": Array [],
		      "stack": "Error: warning 1\\n    at <TEST_TOOLS_ROOT>/errorCases/warning-test-set.js<LINE_COL>\\n    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)\\n    at next (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsyncStageRange (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsync (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
		    },
		    Object {
		      "code": "Error",
		      "message": "  ⚠ warning 2\\n",
		      "moduleTrace": Array [],
		      "stack": "Error: warning 2\\n    at <TEST_TOOLS_ROOT>/errorCases/warning-test-set.js<LINE_COL>\\n    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)\\n    at next (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsyncStageRange (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsync (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
		    },
		  ],
		}
	`);
	}
};
