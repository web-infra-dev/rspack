/** @type {import('@rspack/test-tools').TErrorCaseConfig} */
module.exports = {
	description: "should emit module build errors",
	options() {
		return {
			entry: "./has-syntax-error"
		};
	},
	async check(diagnostics) {
		expect(diagnostics).toMatchInlineSnapshot(`
			Object {
			  "errors": Array [
			    Object {
			      "code": "ModuleParseError",
			      "message": "  × Module parse failed:  ╰─▶   × JavaScript parse error: Unexpected token \`;\`. Expected identifier, string literal, numeric literal or [ for the computed key         ╭─[2:12]       1 │ window.foo = {       2 │   bar: true,;         ·             ─       3 │ };         ╰────        help:         You may need an appropriate loader to handle this file type.",
			      "moduleId": "./has-syntax-error.js",
			      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/has-syntax-error.js",
			      "moduleName": "./has-syntax-error.js",
			      "moduleTrace": Array [],
			      "stack": undefined,
			    },
			  ],
			  "warnings": Array [],
			}
		`);
	}
};
