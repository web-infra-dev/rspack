/** @type {import('@rspack/test-tools').TErrorCaseConfig} */
module.exports = {
	description: "should emit warnings for resolve failure in esm",
	options() {
		return {
			entry: "./resolve-fail-esm"
		};
	},
	async check(diagnostics) {
		expect(diagnostics).toMatchInlineSnapshot(`
		Object {
		  "errors": Array [
		    Object {
		      "loc": "1:0-33",
		      "message": "  × Module not found: Can't resolve './answer' in '<TEST_ROOT>/fixtures/errors/resolve-fail-esm'   ╭──── 1 │ import { answer } from './answer'   · ─────────────────────────────────   ╰────  help: Did you mean './answer.js'?                The request './answer' failed to resolve only because it was resolved as fully specified,        probably because the origin is strict EcmaScript Module,        e. g. a module with javascript mimetype, a '*.mjs' file, or a '*.js' file where the package.json contains '\\"type\\": \\"module\\"'.                The extension in the request is mandatory for it to be fully specified.        Add the extension to the request.",
		      "moduleId": "./resolve-fail-esm/index.js",
		      "moduleIdentifier": "javascript/esm|<TEST_ROOT>/fixtures/errors/resolve-fail-esm/index.js",
		      "moduleName": "./resolve-fail-esm/index.js",
		      "moduleTrace": Array [],
		      "stack": undefined,
		    },
		  ],
		  "warnings": Array [],
		}
	`);
	}
};
