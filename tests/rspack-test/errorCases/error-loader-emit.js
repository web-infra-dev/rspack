/** @type {import('@rspack/test-tools').TErrorCaseConfig[]} */
module.exports = [
	{
		description: "should emit error thrown at module level",
		options() {
			return {
				entry: "./module-level-throw-error-loader!./no-errors-deprecate"
			};
		},
		async check(diagnostics) {
			expect(diagnostics).toMatchInlineSnapshot(`
				Object {
				  "errors": Array [
				    Object {
				      "code": "ModuleBuildError",
				      "message": "  × Module build failed (from ./module-level-throw-error-loader.js):  ╰─▶   × Error: this is a thrown error from module level        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx      ",
				      "moduleId": "./module-level-throw-error-loader.js!./no-errors-deprecate.js",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/module-level-throw-error-loader.js!<TEST_ROOT>/fixtures/errors/no-errors-deprecate.js",
				      "moduleName": "./module-level-throw-error-loader.js!./no-errors-deprecate.js",
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
		description: "should emit errors & warnings for emit-error-loader",
		options() {
			return {
				entry: "./entry-point-error-loader-required.js"
			};
		},
		async check(diagnostics) {
			expect(diagnostics).toMatchInlineSnapshot(`
				Object {
				  "errors": Array [
				    Object {
				      "code": "ModuleError",
				      "message": "  × Module Error (from <TEST_ROOT>/fixtures/errors/emit-error-loader.js):  │ this is an error",
				      "moduleId": "./emit-error-loader.js!./file.js",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/emit-error-loader.js!<TEST_ROOT>/fixtures/errors/file.js",
				      "moduleName": "./emit-error-loader.js!./file.js",
				      "moduleTrace": Array [
				        Object {
				          "dependencies": Array [
				            Object {},
				          ],
				          "moduleId": "./emit-error-loader.js!./file.js",
				          "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/emit-error-loader.js!<TEST_ROOT>/fixtures/errors/file.js",
				          "moduleName": "./emit-error-loader.js!./file.js",
				          "originId": "./entry-point-error-loader-required.js",
				          "originIdentifier": "<TEST_ROOT>/fixtures/errors/entry-point-error-loader-required.js",
				          "originName": "./entry-point-error-loader-required.js",
				        },
				      ],
				      "stack": "ModuleError: Module Error (from <TEST_ROOT>/fixtures/errors/emit-error-loader.js):this is an error    at Object.loaderContext.emitError (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at Object.module.exports (<TEST_ROOT>/fixtures/errors/emit-error-loader.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>    at node:internal/util<LINE_COL>    at new Promise (<anonymous>)    at node:internal/util<LINE_COL>    at isomorphoicRun (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at runLoaders (<RSPACK_ROOT>/dist/index.js<LINE_COL>)",
				    },
				  ],
				  "warnings": Array [
				    Object {
				      "code": "ModuleWarning",
				      "message": "  ⚠ Module Warning (from <TEST_ROOT>/fixtures/errors/emit-error-loader.js):  │ this is a warning",
				      "moduleId": "./emit-error-loader.js!./file.js",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/emit-error-loader.js!<TEST_ROOT>/fixtures/errors/file.js",
				      "moduleName": "./emit-error-loader.js!./file.js",
				      "moduleTrace": Array [
				        Object {
				          "dependencies": Array [
				            Object {},
				          ],
				          "moduleId": "./emit-error-loader.js!./file.js",
				          "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/emit-error-loader.js!<TEST_ROOT>/fixtures/errors/file.js",
				          "moduleName": "./emit-error-loader.js!./file.js",
				          "originId": "./entry-point-error-loader-required.js",
				          "originIdentifier": "<TEST_ROOT>/fixtures/errors/entry-point-error-loader-required.js",
				          "originName": "./entry-point-error-loader-required.js",
				        },
				      ],
				      "stack": "ModuleWarning: Module Warning (from <TEST_ROOT>/fixtures/errors/emit-error-loader.js):this is a warning    at Object.loaderContext.emitWarning (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at Object.module.exports (<TEST_ROOT>/fixtures/errors/emit-error-loader.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>    at node:internal/util<LINE_COL>    at new Promise (<anonymous>)    at node:internal/util<LINE_COL>    at isomorphoicRun (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at runLoaders (<RSPACK_ROOT>/dist/index.js<LINE_COL>)",
				    },
				  ],
				}
			`);
		}
	},
	{
		description: "should emit error & warning for emit-error-loader",
		options() {
			return {
				entry: "./emit-error-loader!./entry-point.js"
			};
		},
		async check(diagnostics) {
			expect(diagnostics).toMatchInlineSnapshot(`
				Object {
				  "errors": Array [
				    Object {
				      "code": "ModuleError",
				      "message": "  × Module Error (from <TEST_ROOT>/fixtures/errors/emit-error-loader.js):  │ this is an error",
				      "moduleId": "./emit-error-loader.js!./entry-point.js",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/emit-error-loader.js!<TEST_ROOT>/fixtures/errors/entry-point.js",
				      "moduleName": "./emit-error-loader.js!./entry-point.js",
				      "moduleTrace": Array [],
				      "stack": "ModuleError: Module Error (from <TEST_ROOT>/fixtures/errors/emit-error-loader.js):this is an error    at Object.loaderContext.emitError (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at Object.module.exports (<TEST_ROOT>/fixtures/errors/emit-error-loader.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>    at node:internal/util<LINE_COL>    at new Promise (<anonymous>)    at node:internal/util<LINE_COL>    at isomorphoicRun (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at runLoaders (<RSPACK_ROOT>/dist/index.js<LINE_COL>)",
				    },
				  ],
				  "warnings": Array [
				    Object {
				      "code": "ModuleWarning",
				      "message": "  ⚠ Module Warning (from <TEST_ROOT>/fixtures/errors/emit-error-loader.js):  │ this is a warning",
				      "moduleId": "./emit-error-loader.js!./entry-point.js",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/emit-error-loader.js!<TEST_ROOT>/fixtures/errors/entry-point.js",
				      "moduleName": "./emit-error-loader.js!./entry-point.js",
				      "moduleTrace": Array [],
				      "stack": "ModuleWarning: Module Warning (from <TEST_ROOT>/fixtures/errors/emit-error-loader.js):this is a warning    at Object.loaderContext.emitWarning (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at Object.module.exports (<TEST_ROOT>/fixtures/errors/emit-error-loader.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>    at node:internal/util<LINE_COL>    at new Promise (<anonymous>)    at node:internal/util<LINE_COL>    at isomorphoicRun (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at runLoaders (<RSPACK_ROOT>/dist/index.js<LINE_COL>)",
				    },
				  ],
				}
			`);
		}
	}
];
