/** @type {import('../..').TErrorCaseConfig} */
module.exports = {
	description: "Testing proxy methods on errors: test push",
	options() {
		return {
			entry: "./resolve-fail-esm",
			plugins: [
				compiler => {
					compiler.hooks.afterCompile.tap("test push", compilation => {
						compilation.errors.push("test push");
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
		      "message": "  × test push\\n",
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
