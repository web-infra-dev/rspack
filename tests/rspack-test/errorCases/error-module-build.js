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
		      "message": "  × Module parse failed:\\n  ╰─▶   × JavaScript parse error: Unexpected token \`;\`. Expected identifier, string literal, numeric literal or [ for the computed key\\n         ╭─[2:12]\\n       1 │ window.foo = {\\n       2 │   bar: true,;\\n         ·             ─\\n       3 │ };\\n         ╰────\\n      \\n  help: \\n        You may need an appropriate loader to handle this file type.\\n",
		      "moduleId": "./has-syntax-error.js",
		      "moduleIdentifier": "<TEST_TOOLS_ROOT>/fixtures/errors/has-syntax-error.js",
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
