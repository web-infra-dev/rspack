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
		      "code": "Error",
		      "message": "  ⚠ Error: test push\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n",
		      "moduleTrace": Array [],
		      "stack": "Error: test push\\n    at <TEST_TOOLS_ROOT>/tests/errorCases/warning-test-push.js<LINE_COL>\\n    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)\\n    at next (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsyncStageRange (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsync (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
		    },
		    Object {
		      "code": "ModuleParseWarning",
		      "message": "  ⚠ Module parse warning:\\n  ╰─▶   ⚠ Unsupported feature: require.main.require() is not supported by Rspack.\\n         ╭────\\n       1 │ require.main.require('./file');\\n         · ──────────────────────────────\\n         ╰────\\n      \\n",
		      "moduleId": "./require.main.require.js",
		      "moduleIdentifier": "<TEST_TOOLS_ROOT>/tests/fixtures/errors/require.main.require.js",
		      "moduleName": "./require.main.require.js",
		      "moduleTrace": Array [],
		      "stack": "ModuleParseWarning:   ⚠ Module parse warning:\\n  ╰─▶   ⚠ Unsupported feature: require.main.require() is not supported by Rspack.\\n         ╭────\\n       1 │ require.main.require('./file');\\n         · ──────────────────────────────\\n         ╰────\\n      \\n\\n    at warningFromStatsWarning (<RSPACK_ROOT>/dist/index.js<LINE_COL>)\\n    at Array.map (<anonymous>)\\n    at context.cachedGetWarnings (<RSPACK_ROOT>/dist/index.js<LINE_COL>)\\n    at warnings (<RSPACK_ROOT>/dist/index.js<LINE_COL>)\\n    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)\\n    at SyncBailHook.callAsyncStageRange (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at SyncBailHook.callStageRange (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at SyncBailHook.call (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at <RSPACK_ROOT>/dist/index.js<LINE_COL>\\n    at StatsFactory._forEachLevel (<RSPACK_ROOT>/dist/index.js<LINE_COL>)\\n    at StatsFactory._create (<RSPACK_ROOT>/dist/index.js<LINE_COL>)\\n    at StatsFactory.create (<RSPACK_ROOT>/dist/index.js<LINE_COL>)\\n    at Stats.toJson (<RSPACK_ROOT>/dist/index.js<LINE_COL>)\\n    at ErrorProcessor.check (<TEST_TOOLS_ROOT>/dist/processor/error.js<LINE_COL>)\\n    at run (<TEST_TOOLS_ROOT>/dist/test/simple.js<LINE_COL>)\\n    at Object.<anonymous> (<TEST_TOOLS_ROOT>/dist/case/error.js<LINE_COL>)",
		    },
		  ],
		}
	`);
	}
};
