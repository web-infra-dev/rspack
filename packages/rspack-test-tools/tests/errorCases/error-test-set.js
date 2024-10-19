/** @type {import('../..').TErrorCaseConfig} */
module.exports = {
	description: "Testing set errors",
	options() {
		return {
			entry: "./resolve-fail-esm",
			plugins: [
				compiler => {
					compiler.hooks.afterCompile.tap("set errors", compilation => {
						compilation.errors = [new Error("error 1"), new Error("error 2")];
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
		      "message": "  × Error: error 1/n  │     at xxx/n  │     at xxx/n  │     at xxx/n  │     at xxx/n  │     at xxx/n  │     at xxx/n",
		      "moduleTrace": Array [],
		      "stack": "Error: error 1/n    at Object.fn (<ROOT>/tests/errorCases/error-test-set.js<LINE_COL>)/n    at next (<WORKSPACE>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at AsyncSeriesHook.callAsyncStageRange (<WORKSPACE>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at AsyncSeriesHook.callAsync (<WORKSPACE>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at <WORKSPACE>/packages/rspack/dist/index.js<LINE_COL>/n    at <WORKSPACE>/packages/rspack/dist/index.js<LINE_COL>",
		    },
		    Object {
		      "message": "  × Error: error 2/n  │     at xxx/n  │     at xxx/n  │     at xxx/n  │     at xxx/n  │     at xxx/n  │     at xxx/n",
		      "moduleTrace": Array [],
		      "stack": "Error: error 2/n    at Object.fn (<ROOT>/tests/errorCases/error-test-set.js<LINE_COL>)/n    at next (<WORKSPACE>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at AsyncSeriesHook.callAsyncStageRange (<WORKSPACE>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at AsyncSeriesHook.callAsync (<WORKSPACE>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at <WORKSPACE>/packages/rspack/dist/index.js<LINE_COL>/n    at <WORKSPACE>/packages/rspack/dist/index.js<LINE_COL>",
		    },
		  ],
		  "warnings": Array [],
		}
	`);
	}
};
