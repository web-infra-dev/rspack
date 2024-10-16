/** @type {import('../..').TErrorCaseConfig} */
module.exports = {
	description: "Testing proxy methods on errors: test splice 1",
	options() {
		return {
			entry: "./resolve-fail-esm",
			plugins: [
				compiler => {
					compiler.hooks.afterCompile.tap("test splice", compilation => {
						compilation.errors.splice(0, 1, new Error("test splice"));
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
		      "message": "  × Error: test splice/n  │     at xxx/n  │     at xxx/n  │     at xxx/n  │     at xxx/n  │     at xxx/n  │     at xxx/n",
		      "moduleTrace": Array [],
		      "stack": "Error: test splice/n    at Object.fn (<ROOT>/tests/errorCases/error-test-splice-1.js<LINE_COL>)/n    at next (<HOME>/rspack-dev/rspack/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at AsyncSeriesHook.callAsyncStageRange (<HOME>/rspack-dev/rspack/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at AsyncSeriesHook.callAsync (<HOME>/rspack-dev/rspack/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at <WORKSPACE>/rspack/dist/index.js<LINE_COL>/n    at <WORKSPACE>/rspack/dist/index.js<LINE_COL>",
		    },
		  ],
		  "warnings": Array [],
		}
	`);
	}
};
