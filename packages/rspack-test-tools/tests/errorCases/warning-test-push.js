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
		      "formatted": "  ⚠ Error: test push\\n  │     at <cwd>packages/rspack-test-tools/tests/errorCases/warning-test-push.js:10:33\\n  │     at Hook.eval [as callAsync] (eval at create (<cwd>node_modules/tapable/lib/HookCodeFactory.js:33:10), <anonymous>:9:1)\\n  │     at Hook.CALL_ASYNC_DELEGATE [as _callAsync] (<cwd>node_modules/tapable/lib/Hook.js:18:14)\\n  │     at <cwd>packages/rspack/dist/Compiler.js:419:41\\n  │     at <cwd>packages/rspack/dist/Compiler.js:751:65\\n",
		      "message": "  ⚠ Error: test push\\n  │     at <cwd>packages/rspack-test-tools/tests/errorCases/warning-test-push.js:10:33\\n  │     at Hook.eval [as callAsync] (eval at create (<cwd>node_modules/tapable/lib/HookCodeFactory.js:33:10), <anonymous>:9:1)\\n  │     at Hook.CALL_ASYNC_DELEGATE [as _callAsync] (<cwd>node_modules/tapable/lib/Hook.js:18:14)\\n  │     at <cwd>packages/rspack/dist/Compiler.js:419:41\\n  │     at <cwd>packages/rspack/dist/Compiler.js:751:65\\n",
		    },
		    Object {
		      "formatted": "  ⚠ Module parse warning:\\n  ╰─▶   ⚠ Module parse failed: require.main.require() is not supported by Rspack.\\n         ╭────\\n       1 │ require.main.require('./file');\\n         · ──────────────────────────────\\n         ╰────\\n      \\n",
		      "message": "  ⚠ Module parse warning:\\n  ╰─▶   ⚠ Module parse failed: require.main.require() is not supported by Rspack.\\n         ╭────\\n       1 │ require.main.require('./file');\\n         · ──────────────────────────────\\n         ╰────\\n      \\n",
		      "moduleId": "./require.main.require.js",
		      "moduleIdentifier": "<cwd>packages/rspack-test-tools/tests/fixtures/errors/require.main.require.js",
		      "moduleName": "./require.main.require.js",
		    },
		  ],
		}
	`);
	}
};
