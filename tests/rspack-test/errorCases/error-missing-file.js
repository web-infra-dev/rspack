/** @type {import('@rspack/test-tools').TErrorCaseConfig[]} */
module.exports = [
	{
		description: "should emit warning for missingFile",
		options() {
			return {
				entry: "./missingFile"
			};
		},
		async check(diagnostics) {
			expect(diagnostics).toMatchInlineSnapshot(`
				Object {
				  "errors": Array [
				    Object {
				      "loc": "4:9-20",
				      "message": "  × Module not found: Can't resolve './missing' in '<TEST_ROOT>/fixtures/errors'   ╭─[4:0] 2 │  3 │ // on line 4 4 │ require(\\"./missing\\");   · ──────────────────── 5 │  6 │    ╰────",
				      "moduleId": "./missingFile.js",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/missingFile.js",
				      "moduleName": "./missingFile.js",
				      "moduleTrace": Array [],
				      "stack": undefined,
				    },
				    Object {
				      "loc": "12:18-34",
				      "message": "  × Module not found: Can't resolve './dir/missing2' in '<TEST_ROOT>/fixtures/errors'    ╭─[12:9] 10 │  11 │ // on line 12 char 10 12 │          require(\\"./dir/missing2\\");    ·          ─────────────────────────    ╰────",
				      "moduleId": "./missingFile.js",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/missingFile.js",
				      "moduleName": "./missingFile.js",
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
		description: "should emit errors for missingFile for production",
		options() {
			return {
				mode: "production",
				entry: "./missingFile"
			};
		},
		async check(diagnostics) {
			expect(diagnostics).toMatchInlineSnapshot(`
				Object {
				  "errors": Array [
				    Object {
				      "loc": "4:9-20",
				      "message": "  × Module not found: Can't resolve './missing' in '<TEST_ROOT>/fixtures/errors'   ╭─[4:0] 2 │  3 │ // on line 4 4 │ require(\\"./missing\\");   · ──────────────────── 5 │  6 │    ╰────",
				      "moduleId": "./missingFile.js",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/missingFile.js",
				      "moduleName": "./missingFile.js",
				      "moduleTrace": Array [],
				      "stack": undefined,
				    },
				    Object {
				      "loc": "12:18-34",
				      "message": "  × Module not found: Can't resolve './dir/missing2' in '<TEST_ROOT>/fixtures/errors'    ╭─[12:9] 10 │  11 │ // on line 12 char 10 12 │          require(\\"./dir/missing2\\");    ·          ─────────────────────────    ╰────",
				      "moduleId": "./missingFile.js",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/missingFile.js",
				      "moduleName": "./missingFile.js",
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
