/** @type {import('../..').TErrorCaseConfig} */
module.exports = {
	description: "Testing proxy methods on warnings: test shift&unshift",
	options() {
		return {
			entry: "./require.main.require",
			plugins: [
				compiler => {
					compiler.hooks.afterCompile.tap(
						"test shift and unshift",
						compilation => {
							compilation.warnings.shift();
							compilation.warnings.unshift(new Error("test unshift"));
						}
					);
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
		      "message": "  ⚠ Error: test unshift/n  │     at xxx/n  │     at xxx/n  │     at xxx/n  │     at xxx/n  │     at xxx/n  │     at xxx/n",
		      "moduleTrace": Array [],
		      "stack": "Error: test unshift/n    at Object.fn (<ROOT>/tests/errorCases/warning-test-shift.js<LINE_COL>)/n    at next (<WORKSPACE>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at AsyncSeriesHook.callAsyncStageRange (<WORKSPACE>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at AsyncSeriesHook.callAsync (<WORKSPACE>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at <WORKSPACE>/packages/rspack/dist/index.js<LINE_COL>/n    at <WORKSPACE>/packages/rspack/dist/index.js<LINE_COL>",
		    },
		  ],
		}
	`);
	}
};
