/** @type {import('../..').TErrorCaseConfig} */
module.exports = {
	description: "Testing proxy methods on errors: test splice 2",
	options() {
		return {
			entry: "./resolve-fail-esm",
			plugins: [
				compiler => {
					compiler.hooks.afterCompile.tap("test splice", compilation => {
						compilation.errors.splice(0, 0, new Error("test splice"));
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
		      "message": "  × Error: test splice\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n",
		      "stack": "Error: test splice\\n    at Object.fn (<cwd>packages/rspack-test-tools/tests/errorCases/error-test-splice-2.js:10:39)\\n    at next (<cwd>packages/rspack-lite-tapable/dist/index.js:530:25)\\n    at AsyncSeriesHook.callAsyncStageRange (<cwd>packages/rspack-lite-tapable/dist/index.js:550:9)\\n    at AsyncSeriesHook.callAsync (<cwd>packages/rspack-lite-tapable/dist/index.js:88:21)\\n    at <cwd>packages/rspack/dist/Compiler.js:467:41\\n    at <cwd>packages/rspack/dist/Compiler.js:534:23",
		    },
		    Object {
		      "message": "  × Resolve error: Can't resolve './answer' in '<cwd>packages/rspack-test-tools/tests/fixtures/errors/resolve-fail-esm'\\n   ╭────\\n 1 │ import { answer } from './answer'\\n   ·                        ──────────\\n   ╰────\\n  help: Did you mean './answer.js'?\\n        \\n        The request './answer' failed to resolve only because it was resolved as fully specified,\\n        probably because the origin is strict EcmaScript Module,\\n        e. g. a module with javascript mimetype, a '*.mjs' file, or a '*.js' file where the package.json contains '\\"type\\": \\"module\\"'.\\n        \\n        The extension in the request is mandatory for it to be fully specified.\\n        Add the extension to the request.\\n",
		      "moduleId": "./resolve-fail-esm/index.js",
		      "moduleIdentifier": "javascript/esm|<cwd>packages/rspack-test-tools/tests/fixtures/errors/resolve-fail-esm/index.js",
		      "moduleName": "./resolve-fail-esm/index.js",
		    },
		  ],
		  "warnings": Array [],
		}
	`);
	}
};
