/** @type {import('@rspack/test-tools').TErrorCaseConfig[]} */
module.exports = [
	{
		description: "Testing proxy methods on errors: test pop",
		options() {
			return {
				entry: "./resolve-fail-esm",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap("test pop", compilation => {
							compilation.errors.pop();
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
		description: "Testing proxy methods on errors: test push",
		options() {
			return {
				entry: "./resolve-fail-esm",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap("test push", compilation => {
							compilation.errors.push(new Error("test push"));
							compilation.errors.push("test push 2");
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
			      "code": "Error",
			      "message": "  × test push\\n",
			      "moduleTrace": Array [],
			      "stack": "Error: test push\\n    at <TEST_TOOLS_ROOT>/errorCases/error-write.js<LINE_COL>\\n    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)\\n    at next (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsyncStageRange (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsync (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
			    },
			    Object {
			      "code": "Error",
			      "message": "  × test push 2\\n",
			      "moduleTrace": Array [],
			      "stack": undefined,
			    },
			    Object {
			      "loc": "1:0-33",
			      "message": "  × Module not found: Can't resolve './answer' in '<TEST_TOOLS_ROOT>/fixtures/errors/resolve-fail-esm'\\n   ╭────\\n 1 │ import { answer } from './answer'\\n   · ─────────────────────────────────\\n   ╰────\\n  help: Did you mean './answer.js'?\\n        \\n        The request './answer' failed to resolve only because it was resolved as fully specified,\\n        probably because the origin is strict EcmaScript Module,\\n        e. g. a module with javascript mimetype, a '*.mjs' file, or a '*.js' file where the package.json contains '\\"type\\": \\"module\\"'.\\n        \\n        The extension in the request is mandatory for it to be fully specified.\\n        Add the extension to the request.\\n",
			      "moduleId": "./resolve-fail-esm/index.js",
			      "moduleIdentifier": "javascript/esm|<TEST_TOOLS_ROOT>/fixtures/errors/resolve-fail-esm/index.js",
			      "moduleName": "./resolve-fail-esm/index.js",
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
		description: "Testing set errors",
		options() {
			return {
				entry: "./resolve-fail-esm",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap("set errors", compilation => {
							compilation.errors[0] = new Error("error 1");
							expect(compilation.errors[0].message).toMatch(/error 1/);
							expect(compilation.errors[1]).toBe(undefined);
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
			      "code": "Error",
			      "message": "  × error 1\\n",
			      "moduleTrace": Array [],
			      "stack": "Error: error 1\\n    at <TEST_TOOLS_ROOT>/errorCases/error-write.js<LINE_COL>\\n    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)\\n    at next (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsyncStageRange (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsync (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
			    },
			  ],
			  "warnings": Array [],
			}
		`);
		}
	},
	{
		description: "Testing set errors",
		options() {
			return {
				entry: "./resolve-fail-esm",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap("set errors", compilation => {
							compilation.errors = [new Error("error 1"), new Error("error 2")];
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
			      "code": "Error",
			      "message": "  × error 1\\n",
			      "moduleTrace": Array [],
			      "stack": "Error: error 1\\n    at <TEST_TOOLS_ROOT>/errorCases/error-write.js<LINE_COL>\\n    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)\\n    at next (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsyncStageRange (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsync (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
			    },
			    Object {
			      "code": "Error",
			      "message": "  × error 2\\n",
			      "moduleTrace": Array [],
			      "stack": "Error: error 2\\n    at <TEST_TOOLS_ROOT>/errorCases/error-write.js<LINE_COL>\\n    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)\\n    at next (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsyncStageRange (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsync (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
			    },
			  ],
			  "warnings": Array [],
			}
		`);
		}
	},
	{
		description: "Testing proxy methods on errors: test shift&unshift",
		options() {
			return {
				entry: "./resolve-fail-esm",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap(
							"test shift and unshift",
							compilation => {
								compilation.errors.shift();
								compilation.errors.unshift(new Error("test unshift"));
							}
						);
					}
				]
			};
		},
		async check(diagnostics) {
			expect(diagnostics).toMatchInlineSnapshot(`
			Object {
			  "errors": Array [
			    Object {
			      "code": "Error",
			      "message": "  × test unshift\\n",
			      "moduleTrace": Array [],
			      "stack": "Error: test unshift\\n    at <TEST_TOOLS_ROOT>/errorCases/error-write.js<LINE_COL>\\n    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)\\n    at next (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsyncStageRange (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsync (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
			    },
			  ],
			  "warnings": Array [],
			}
		`);
		}
	},
	{
		description: "Testing proxy methods on errors: test splice 1",
		options() {
			return {
				entry: "./resolve-fail-esm",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap("test splice", compilation => {
							compilation.errors.splice(0, 1, new Error("test splice"));
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
			      "code": "Error",
			      "message": "  × test splice\\n",
			      "moduleTrace": Array [],
			      "stack": "Error: test splice\\n    at <TEST_TOOLS_ROOT>/errorCases/error-write.js<LINE_COL>\\n    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)\\n    at next (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsyncStageRange (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsync (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
			    },
			  ],
			  "warnings": Array [],
			}
		`);
		}
	},
	{
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
			      "code": "Error",
			      "message": "  × test splice\\n",
			      "moduleTrace": Array [],
			      "stack": "Error: test splice\\n    at <TEST_TOOLS_ROOT>/errorCases/error-write.js<LINE_COL>\\n    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)\\n    at next (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsyncStageRange (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at AsyncSeriesHook.callAsync (<ROOT>/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)\\n    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
			    },
			    Object {
			      "loc": "1:0-33",
			      "message": "  × Module not found: Can't resolve './answer' in '<TEST_TOOLS_ROOT>/fixtures/errors/resolve-fail-esm'\\n   ╭────\\n 1 │ import { answer } from './answer'\\n   · ─────────────────────────────────\\n   ╰────\\n  help: Did you mean './answer.js'?\\n        \\n        The request './answer' failed to resolve only because it was resolved as fully specified,\\n        probably because the origin is strict EcmaScript Module,\\n        e. g. a module with javascript mimetype, a '*.mjs' file, or a '*.js' file where the package.json contains '\\"type\\": \\"module\\"'.\\n        \\n        The extension in the request is mandatory for it to be fully specified.\\n        Add the extension to the request.\\n",
			      "moduleId": "./resolve-fail-esm/index.js",
			      "moduleIdentifier": "javascript/esm|<TEST_TOOLS_ROOT>/fixtures/errors/resolve-fail-esm/index.js",
			      "moduleName": "./resolve-fail-esm/index.js",
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
