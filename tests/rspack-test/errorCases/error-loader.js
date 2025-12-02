/** @type {import('@rspack/test-tools').TErrorCaseConfig[]} */
module.exports = [
	{
		description: "should emit error for async-error-loader",
		options() {
			return {
				entry: "./async-error-loader!./entry-point.js"
			};
		},
		async check(diagnostics) {
			expect(diagnostics).toMatchInlineSnapshot(`
				Object {
				  "errors": Array [
				    Object {
				      "code": "ModuleBuildError",
				      "message": "  × Module build failed (from ./async-error-loader.js):  ╰─▶   × Error: this is a callback error        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx      ",
				      "moduleId": "./async-error-loader.js!./entry-point.js",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/async-error-loader.js!<TEST_ROOT>/fixtures/errors/entry-point.js",
				      "moduleName": "./async-error-loader.js!./entry-point.js",
				      "moduleTrace": Array [],
				      "stack": undefined,
				    },
				  ],
				  "warnings": Array [],
				}
			`);
		}
	},
	{
		description: "should emit error thrown from raw loader",
		options() {
			return {
				entry: "./throw-error-loader!./entry-point.js"
			};
		},
		async check(diagnostics) {
			expect(diagnostics).toMatchInlineSnapshot(`
				Object {
				  "errors": Array [
				    Object {
				      "code": "ModuleBuildError",
				      "message": "  × Module build failed (from ./throw-error-loader.js):  ╰─▶   × Error: this is a thrown error        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx      ",
				      "moduleId": "./throw-error-loader.js!./entry-point.js",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/throw-error-loader.js!<TEST_ROOT>/fixtures/errors/entry-point.js",
				      "moduleName": "./throw-error-loader.js!./entry-point.js",
				      "moduleTrace": Array [],
				      "stack": undefined,
				    },
				  ],
				  "warnings": Array [],
				}
			`);
		}
	},
	{
		description: "should emit error thrown from pitch loader",
		options() {
			return {
				entry: "./throw-error-pitch-loader!./entry-point.js"
			};
		},
		async check(diagnostics) {
			expect(diagnostics).toMatchInlineSnapshot(`
				Object {
				  "errors": Array [
				    Object {
				      "code": "ModuleBuildError",
				      "message": "  × Module build failed (from ./throw-error-pitch-loader.js):  ╰─▶   × Error: this is a thrown error        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx      ",
				      "moduleId": "./throw-error-pitch-loader.js!./entry-point.js",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/throw-error-pitch-loader.js!<TEST_ROOT>/fixtures/errors/entry-point.js",
				      "moduleName": "./throw-error-pitch-loader.js!./entry-point.js",
				      "moduleTrace": Array [],
				      "stack": undefined,
				    },
				  ],
				  "warnings": Array [],
				}
			`);
		}
	},
	{
		description: "should emit errors & warnings for irregular-error-loader",
		options() {
			return {
				entry: "./irregular-error-loader!./entry-point.js"
			};
		},
		async check(diagnostics) {
			expect(diagnostics).toMatchInlineSnapshot(`
				Object {
				  "errors": Array [
				    Object {
				      "code": "ModuleError",
				      "message": "  × Module Error (from <TEST_ROOT>/fixtures/errors/irregular-error-loader.js):  │ (Emitted value instead of an instance of Error) null",
				      "moduleId": "./irregular-error-loader.js!./entry-point.js",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/irregular-error-loader.js!<TEST_ROOT>/fixtures/errors/entry-point.js",
				      "moduleName": "./irregular-error-loader.js!./entry-point.js",
				      "moduleTrace": Array [],
				      "stack": "ModuleError: Module Error (from <TEST_ROOT>/fixtures/errors/irregular-error-loader.js):(Emitted value instead of an instance of Error) null    at Object.loaderContext.emitError (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at Object.module.exports (<TEST_ROOT>/fixtures/errors/irregular-error-loader.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>    at node:internal/util<LINE_COL>    at new Promise (<anonymous>)    at node:internal/util<LINE_COL>    at isomorphoicRun (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at runLoaders (<RSPACK_ROOT>/dist/index.js<LINE_COL>)",
				    },
				    Object {
				      "code": "ModuleError",
				      "message": "  × Module Error (from <TEST_ROOT>/fixtures/errors/irregular-error-loader.js):  │ Error",
				      "moduleId": "./irregular-error-loader.js!./entry-point.js",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/irregular-error-loader.js!<TEST_ROOT>/fixtures/errors/entry-point.js",
				      "moduleName": "./irregular-error-loader.js!./entry-point.js",
				      "moduleTrace": Array [],
				      "stack": "ModuleError: Module Error (from <TEST_ROOT>/fixtures/errors/irregular-error-loader.js):Error    at Object.loaderContext.emitError (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at Object.module.exports (<TEST_ROOT>/fixtures/errors/irregular-error-loader.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>    at node:internal/util<LINE_COL>    at new Promise (<anonymous>)    at node:internal/util<LINE_COL>    at isomorphoicRun (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at runLoaders (<RSPACK_ROOT>/dist/index.js<LINE_COL>)",
				    },
				    Object {
				      "code": "ModuleBuildError",
				      "message": "  × Module build failed (from ./irregular-error-loader.js):  ╰─▶   × TypeError: Cannot use 'in' operator to search for 'hideStack' in a string error        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx      ",
				      "moduleId": "./irregular-error-loader.js!./entry-point.js",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/irregular-error-loader.js!<TEST_ROOT>/fixtures/errors/entry-point.js",
				      "moduleName": "./irregular-error-loader.js!./entry-point.js",
				      "moduleTrace": Array [],
				      "stack": undefined,
				    },
				  ],
				  "warnings": Array [
				    Object {
				      "code": "ModuleWarning",
				      "message": "  ⚠ Module Warning (from <TEST_ROOT>/fixtures/errors/irregular-error-loader.js):  │ (Emitted value instead of an instance of Error) null",
				      "moduleId": "./irregular-error-loader.js!./entry-point.js",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/irregular-error-loader.js!<TEST_ROOT>/fixtures/errors/entry-point.js",
				      "moduleName": "./irregular-error-loader.js!./entry-point.js",
				      "moduleTrace": Array [],
				      "stack": "ModuleWarning: Module Warning (from <TEST_ROOT>/fixtures/errors/irregular-error-loader.js):(Emitted value instead of an instance of Error) null    at Object.loaderContext.emitWarning (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at Object.module.exports (<TEST_ROOT>/fixtures/errors/irregular-error-loader.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>    at node:internal/util<LINE_COL>    at new Promise (<anonymous>)    at node:internal/util<LINE_COL>    at isomorphoicRun (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at runLoaders (<RSPACK_ROOT>/dist/index.js<LINE_COL>)",
				    },
				    Object {
				      "code": "ModuleWarning",
				      "message": "  ⚠ Module Warning (from <TEST_ROOT>/fixtures/errors/irregular-error-loader.js):  │ Error",
				      "moduleId": "./irregular-error-loader.js!./entry-point.js",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/irregular-error-loader.js!<TEST_ROOT>/fixtures/errors/entry-point.js",
				      "moduleName": "./irregular-error-loader.js!./entry-point.js",
				      "moduleTrace": Array [],
				      "stack": "ModuleWarning: Module Warning (from <TEST_ROOT>/fixtures/errors/irregular-error-loader.js):Error    at Object.loaderContext.emitWarning (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at Object.module.exports (<TEST_ROOT>/fixtures/errors/irregular-error-loader.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>    at node:internal/util<LINE_COL>    at new Promise (<anonymous>)    at node:internal/util<LINE_COL>    at isomorphoicRun (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at runLoaders (<RSPACK_ROOT>/dist/index.js<LINE_COL>)",
				    },
				  ],
				}
			`);
		}
	},
	{
		description: "should emit error for no-return-loader",
		options() {
			return { entry: "./no-return-loader!./entry-point.js" };
		},
		async check(diagnostics) {
			expect(diagnostics).toMatchInlineSnapshot(`
				Object {
				  "errors": Array [
				    Object {
				      "code": "ModuleBuildError",
				      "message": "  × Module build failed:  ╰─▶   × Final loader(<TEST_ROOT>/fixtures/errors/no-return-loader.js) didn't return a Buffer or String      ",
				      "moduleId": "./no-return-loader.js!./entry-point.js",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/no-return-loader.js!<TEST_ROOT>/fixtures/errors/entry-point.js",
				      "moduleName": "./no-return-loader.js!./entry-point.js",
				      "moduleTrace": Array [],
				      "stack": undefined,
				    },
				  ],
				  "warnings": Array [],
				}
			`);
		}
	},
	{
		description: "should emit error for doesnt-exist-loader",
		options() {
			return {
				entry: "./doesnt-exist-loader!./entry-point.js"
			};
		},
		async check(diagnostics) {
			expect(diagnostics).toMatchInlineSnapshot(`
			Object {
			  "errors": Array [
			    Object {
			      "message": "  × Unable to resolve loader ./doesnt-exist-loader",
			      "moduleTrace": Array [],
			      "stack": undefined,
			    },
			  ],
			  "warnings": Array [],
			}
		`);
		}
	},
	{
		description: "should emit error for return-undefined-loader",
		options() {
			return {
				entry: "./return-undefined-loader!./entry-point.js"
			};
		},
		async check(diagnostics) {
			expect(diagnostics).toMatchInlineSnapshot(`
				Object {
				  "errors": Array [
				    Object {
				      "code": "ModuleBuildError",
				      "message": "  × Module build failed:  ╰─▶   × Final loader(<TEST_ROOT>/fixtures/errors/return-undefined-loader.js) didn't return a Buffer or String      ",
				      "moduleId": "./return-undefined-loader.js!./entry-point.js",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/return-undefined-loader.js!<TEST_ROOT>/fixtures/errors/entry-point.js",
				      "moduleName": "./return-undefined-loader.js!./entry-point.js",
				      "moduleTrace": Array [],
				      "stack": undefined,
				    },
				  ],
				  "warnings": Array [],
				}
			`);
		}
	},
	{
		description: "should emit error for module-exports-object-loader",
		options() {
			return {
				entry: "./module-exports-object-loader!./entry-point.js"
			};
		},
		async check(diagnostics) {
			expect(diagnostics).toMatchInlineSnapshot(`
				Object {
				  "errors": Array [
				    Object {
				      "code": "ModuleBuildError",
				      "message": "  × Module build failed (from ./module-exports-object-loader.js):  ╰─▶   × LoaderRunnerError: Module '<TEST_ROOT>/fixtures/errors/module-exports-object-loader.js' is not a loader (must have normal or pitch function)        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx      ",
				      "moduleId": "./module-exports-object-loader.js!./entry-point.js",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/module-exports-object-loader.js!<TEST_ROOT>/fixtures/errors/entry-point.js",
				      "moduleName": "./module-exports-object-loader.js!./entry-point.js",
				      "moduleTrace": Array [],
				      "stack": undefined,
				    },
				  ],
				  "warnings": Array [],
				}
			`);
		}
	}
];
