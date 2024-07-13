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
		      "message": "  ⚠ Error: test push\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n",
		      "moduleTrace": Array [],
		      "stack": "Error: test push\\n    at Object.fn (<cwd>packages/rspack-test-tools/tests/errorCases/warning-test-push.js:10:33)\\n    at next (<cwd>packages/rspack-lite-tapable/dist/index.js:530:25)\\n    at AsyncSeriesHook.callAsyncStageRange (<cwd>packages/rspack-lite-tapable/dist/index.js:550:9)\\n    at AsyncSeriesHook.callAsync (<cwd>packages/rspack-lite-tapable/dist/index.js:88:21)\\n    at <cwd>packages/rspack/dist/Compiler.js:466:41\\n    at <cwd>packages/rspack/dist/Compiler.js:533:23",
		    },
		    Object {
		      "message": "  ⚠ Module parse warning:\\n  ╰─▶   ⚠ Unsupported feature: require.main.require() is not supported by Rspack.\\n         ╭────\\n       1 │ require.main.require('./file');\\n         · ──────────────────────────────\\n         ╰────\\n      \\n",
		      "moduleId": "./require.main.require.js",
		      "moduleIdentifier": "<cwd>packages/rspack-test-tools/tests/fixtures/errors/require.main.require.js",
		      "moduleName": "./require.main.require.js",
		      "moduleTrace": Array [],
		    },
		  ],
		}
	`);
	}
};
