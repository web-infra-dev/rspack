/** @type {import('@rspack/test-tools').TErrorCaseConfig[]} */
module.exports = [
	{
		description: "Testing proxy methods on warnings: test pop",
		options() {
			return {
				entry: "./require.main.require",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap("test pop", compilation => {
							compilation.warnings.pop();
						});
					}
				]
			};
		},
		async check(diagnostics) {
			expect(diagnostics).toMatchInlineSnapshot(`
					Object {
					  "errors": Array [],
					  "warnings": Array [],
					}
			`);
		}
	},
	{
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
				      "code": "Error",
				      "message": "  ⚠ test push",
				      "moduleTrace": Array [],
				      "stack": "Error: test push    at <TEST_ROOT>/errorCases/warning-modify.js<LINE_COL>    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at next (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsyncStageRange (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsync (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
				    },
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
	},
	{
		description: "Testing set warnings",
		options() {
			return {
				entry: "./require.main.require",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap("set warnings", compilation => {
							compilation.warnings = [
								new Error("warning 1"),
								new Error("warning 2")
							];
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
				      "code": "Error",
				      "message": "  ⚠ warning 1",
				      "moduleTrace": Array [],
				      "stack": "Error: warning 1    at <TEST_ROOT>/errorCases/warning-modify.js<LINE_COL>    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at next (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsyncStageRange (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsync (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
				    },
				    Object {
				      "code": "Error",
				      "message": "  ⚠ warning 2",
				      "moduleTrace": Array [],
				      "stack": "Error: warning 2    at <TEST_ROOT>/errorCases/warning-modify.js<LINE_COL>    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at next (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsyncStageRange (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsync (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
				    },
				  ],
				}
			`);
		}
	},
	{
		description: "Testing proxy methods on warnings: test shift&unshift",
		options() {
			return {
				entry: "./require.main.require",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap(
							"test shift and unshift",
							compilation => {
								compilation.warnings.shift();
								compilation.warnings.unshift(new Error("test unshift"));
							}
						);
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
				      "code": "Error",
				      "message": "  ⚠ test unshift",
				      "moduleTrace": Array [],
				      "stack": "Error: test unshift    at <TEST_ROOT>/errorCases/warning-modify.js<LINE_COL>    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at next (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsyncStageRange (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsync (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
				    },
				  ],
				}
			`);
		}
	},
	{
		description: "Testing proxy methods on warnings: test splice 1",
		options() {
			return {
				entry: "./require.main.require",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap("test splice", compilation => {
							compilation.warnings.splice(0, 1, new Error("test splice"));
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
				      "code": "Error",
				      "message": "  ⚠ test splice",
				      "moduleTrace": Array [],
				      "stack": "Error: test splice    at <TEST_ROOT>/errorCases/warning-modify.js<LINE_COL>    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at next (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsyncStageRange (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsync (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
				    },
				  ],
				}
			`);
		}
	},
	{
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
				      "code": "Error",
				      "message": "  ⚠ test splice",
				      "moduleTrace": Array [],
				      "stack": "Error: test splice    at <TEST_ROOT>/errorCases/warning-modify.js<LINE_COL>    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at next (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsyncStageRange (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsync (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
				    },
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
	}
];
