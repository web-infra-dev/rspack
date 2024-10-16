/** @type {import('../..').TErrorCaseConfig} */
module.exports = {
	description: "Testing proxy methods on warnings: test push",
	options() {
		return {
			entry: "./require.main.require",
			plugins: [
				compiler => {
					compiler.hooks.afterCompile.tap("test push", compilation => {
						compilation.warnings.push(new Error("test push"));
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
		      "message": "  ⚠ Error: test push/n  │     at xxx/n  │     at xxx/n  │     at xxx/n  │     at xxx/n  │     at xxx/n  │     at xxx/n",
		      "moduleTrace": Array [],
		      "stack": "Error: test push/n    at Object.fn (<ROOT>/tests/errorCases/warning-test-push.js<LINE_COL>)/n    at next (<HOME>/rspack-dev/rspack/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at AsyncSeriesHook.callAsyncStageRange (<HOME>/rspack-dev/rspack/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at AsyncSeriesHook.callAsync (<HOME>/rspack-dev/rspack/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at <WORKSPACE>/rspack/dist/index.js<LINE_COL>/n    at <WORKSPACE>/rspack/dist/index.js<LINE_COL>",
		    },
		    Object {
		      "message": "  ⚠ Module parse warnin/n  ╰─▶   ⚠ Unsupported feature: require.main.require() is not supported by Rspack./n         ╭────/n       1 │ require.main.require('./file');/n         · ──────────────────────────────/n         ╰────/n      /n",
		      "moduleId": "./require.main.require.js",
		      "moduleIdentifier": "<ROOT>/tests/fixtures/errors/require.main.require.js",
		      "moduleName": "./require.main.require.js",
		      "moduleTrace": Array [],
		      "stack": undefined,
		    },
		  ],
		}
	`);
	}
};
