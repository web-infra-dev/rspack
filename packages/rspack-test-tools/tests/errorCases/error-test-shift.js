/** @type {import('../..').TErrorCaseConfig} */
module.exports = {
	description: "Testing proxy methods on errors: test shift&unshift",
	options() {
		return {
			entry: "./resolve-fail-esm",
			plugins: [
				compiler => {
					compiler.hooks.afterCompile.tap(
						"test shift and unshift",
						compilation => {
							compilation.errors.shift();
							compilation.errors.unshift(new Error("test unshift"));
						}
					);
				}
			]
		};
	},
	async check(diagnostics) {
		expect(diagnostics).toMatchInlineSnapshot(`
		Object {
		  "errors": Array [
		    Object {
		      "message": "  × Error: test unshift/n  │     at xxx/n  │     at xxx/n  │     at xxx/n  │     at xxx/n  │     at xxx/n  │     at xxx/n",
		      "moduleTrace": Array [],
		      "stack": "Error: test unshift/n    at Object.fn (<ROOT>/tests/errorCases/error-test-shift.js<LINE_COL>)/n    at next (<HOME>/rspack-dev/rspack/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at AsyncSeriesHook.callAsyncStageRange (<HOME>/rspack-dev/rspack/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at AsyncSeriesHook.callAsync (<HOME>/rspack-dev/rspack/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at <WORKSPACE>/rspack/dist/index.js<LINE_COL>/n    at <WORKSPACE>/rspack/dist/index.js<LINE_COL>",
		    },
		  ],
		  "warnings": Array [],
		}
	`);
	}
};
