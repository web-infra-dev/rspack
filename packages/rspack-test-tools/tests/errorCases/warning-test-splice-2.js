/** @type {import('../..').TErrorCaseConfig} */
module.exports = {
	description: "Testing proxy methods on warnings: test splice 2",
	options() {
		return {
			entry: "./require.main.require",
			plugins: [
				compiler => {
					compiler.hooks.afterCompile.tap("test splice", compilation => {
						compilation.warnings.splice(0, 0, new Error("test splice"));
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
		      "formatted": "  ⚠ Error: test splice\\n  │     at <cwd>packages/rspack-test-tools/tests/errorCases/warning-test-splice-2.js:10:41\\n  │     at Hook.eval [as callAsync] (eval at create (<cwd>node_modules/tapable/lib/HookCodeFactory.js:33:10), <anonymous>:9:1)\\n  │     at Hook.CALL_ASYNC_DELEGATE [as _callAsync] (<cwd>node_modules/tapable/lib/Hook.js:18:14)\\n  │     at <cwd>packages/rspack/dist/Compiler.js:419:41\\n  │     at <cwd>packages/rspack/dist/Compiler.js:751:65\\n",
		      "message": "  ⚠ Error: test splice\\n  │     at <cwd>packages/rspack-test-tools/tests/errorCases/warning-test-splice-2.js:10:41\\n  │     at Hook.eval [as callAsync] (eval at create (<cwd>node_modules/tapable/lib/HookCodeFactory.js:33:10), <anonymous>:9:1)\\n  │     at Hook.CALL_ASYNC_DELEGATE [as _callAsync] (<cwd>node_modules/tapable/lib/Hook.js:18:14)\\n  │     at <cwd>packages/rspack/dist/Compiler.js:419:41\\n  │     at <cwd>packages/rspack/dist/Compiler.js:751:65\\n",
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
