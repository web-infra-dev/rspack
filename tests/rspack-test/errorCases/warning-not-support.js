/** @type {import('@rspack/test-tools').TErrorCaseConfig[]} */
module.exports = [{
	description: "should emit warning for module.parent.require",
	options() {
		return {
			entry: "./module.parent.require"
		};
	},
	async check(diagnostics) {
		expect(diagnostics).toMatchInlineSnapshot(`
			Object {
			  "errors": Array [],
			  "warnings": Array [
			    Object {
			      "code": "ModuleParseWarning",
			      "message": "  ⚠ Module parse warning:  ╰─▶   ⚠ Unsupported feature: module.parent.require() is not supported by Rspack.         ╭────       1 │ module.parent.require('./file');         · ───────────────────────────────         ╰────      ",
			      "moduleId": "./module.parent.require.js",
			      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/module.parent.require.js",
			      "moduleName": "./module.parent.require.js",
			      "moduleTrace": Array [],
			      "stack": undefined,
			    },
			  ],
			}
		`);
	}
}, {
	description: "should emit warning for require.extensions",
	options() {
		return {
			entry: "./require.extensions"
		};
	},
	async check(diagnostics) {
		expect(diagnostics).toMatchInlineSnapshot(`
			Object {
			  "errors": Array [],
			  "warnings": Array [
			    Object {
			      "code": "ModuleParseWarning",
			      "message": "  ⚠ Module parse warning:  ╰─▶   ⚠ Unsupported feature: require.extensions is not supported by Rspack.         ╭────       1 │ require.extensions[\\".js\\"] = function() {};         · ──────────────────         ╰────      ",
			      "moduleId": "./require.extensions.js",
			      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/require.extensions.js",
			      "moduleName": "./require.extensions.js",
			      "moduleTrace": Array [],
			      "stack": undefined,
			    },
			  ],
			}
		`);
	}
}, {
	description: "should emit warning for require.main.require",
	options() {
		return {
			entry: "./require.main.require"
		};
	},
	async check(diagnostics) {
		expect(diagnostics).toMatchInlineSnapshot(`
			Object {
			  "errors": Array [],
			  "warnings": Array [
			    Object {
			      "code": "ModuleParseWarning",
			      "message": "  ⚠ Module parse warning:  ╰─▶   ⚠ Unsupported feature: require.main.require() is not supported by Rspack.         ╭────       1 │ require.main.require('./file');         · ──────────────────────────────         ╰────      ",
			      "moduleId": "./require.main.require.js",
			      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/require.main.require.js",
			      "moduleName": "./require.main.require.js",
			      "moduleTrace": Array [],
			      "stack": undefined,
			    },
			  ],
			}
		`);
	}
}];
