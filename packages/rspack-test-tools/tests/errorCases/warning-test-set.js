/** @type {import('../..').TErrorCaseConfig} */
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
		      "message": "  ⚠ Error: warning 1/n  │     at xxx/n  │     at xxx/n  │     at xxx/n  │     at xxx/n  │     at xxx/n  │     at xxx/n",
		      "moduleTrace": Array [],
		      "stack": "Error: warning 1/n    at Object.fn (<ROOT>/tests/errorCases/warning-test-set.js<LINE_COL>)/n    at next (<WORKSPACE>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at AsyncSeriesHook.callAsyncStageRange (<WORKSPACE>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at AsyncSeriesHook.callAsync (<WORKSPACE>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at <WORKSPACE>/packages/rspack/dist/index.js<LINE_COL>/n    at <WORKSPACE>/packages/rspack/dist/index.js<LINE_COL>",
		    },
		    Object {
		      "message": "  ⚠ Error: warning 2/n  │     at xxx/n  │     at xxx/n  │     at xxx/n  │     at xxx/n  │     at xxx/n  │     at xxx/n",
		      "moduleTrace": Array [],
		      "stack": "Error: warning 2/n    at Object.fn (<ROOT>/tests/errorCases/warning-test-set.js<LINE_COL>)/n    at next (<WORKSPACE>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at AsyncSeriesHook.callAsyncStageRange (<WORKSPACE>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at AsyncSeriesHook.callAsync (<WORKSPACE>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at <WORKSPACE>/packages/rspack/dist/index.js<LINE_COL>/n    at <WORKSPACE>/packages/rspack/dist/index.js<LINE_COL>",
		    },
		  ],
		}
	`);
	}
};
