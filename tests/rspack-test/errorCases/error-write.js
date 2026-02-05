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
              "message": "  × test push",
              "moduleTrace": Array [],
              "stack": "Error: test push    at <TEST_ROOT>/errorCases/error-write.js<LINE_COL>    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at next (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsyncStageRange (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsync (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
            },
            Object {
              "code": "Error",
              "message": "  × test push 2",
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
              "message": "  × error 1",
              "moduleTrace": Array [],
              "stack": "Error: error 1    at <TEST_ROOT>/errorCases/error-write.js<LINE_COL>    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at next (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsyncStageRange (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsync (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
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
              "message": "  × error 1",
              "moduleTrace": Array [],
              "stack": "Error: error 1    at <TEST_ROOT>/errorCases/error-write.js<LINE_COL>    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at next (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsyncStageRange (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsync (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
            },
            Object {
              "code": "Error",
              "message": "  × error 2",
              "moduleTrace": Array [],
              "stack": "Error: error 2    at <TEST_ROOT>/errorCases/error-write.js<LINE_COL>    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at next (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsyncStageRange (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsync (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
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
              "message": "  × test unshift",
              "moduleTrace": Array [],
              "stack": "Error: test unshift    at <TEST_ROOT>/errorCases/error-write.js<LINE_COL>    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at next (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsyncStageRange (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsync (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
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
              "message": "  × test splice",
              "moduleTrace": Array [],
              "stack": "Error: test splice    at <TEST_ROOT>/errorCases/error-write.js<LINE_COL>    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at next (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsyncStageRange (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsync (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
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
              "message": "  × test splice",
              "moduleTrace": Array [],
              "stack": "Error: test splice    at <TEST_ROOT>/errorCases/error-write.js<LINE_COL>    at Object.fn (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at next (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsyncStageRange (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at AsyncSeriesHook.callAsync (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>",
            },
          ],
          "warnings": Array [],
        }
      `);
    }
  }
];
