/** @type {import('@rspack/test-tools').TErrorCaseConfig[]} */
module.exports = [{
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
		      "loc": "4:8-19",
		      "message": "  × Module not found: Can't resolve './missing' in '<TEST_TOOLS_ROOT>/fixtures/errors'\\n   ╭─[4:0]\\n 2 │ \\n 3 │ // on line 4\\n 4 │ require(\\"./missing\\");\\n   · ────────────────────\\n 5 │ \\n 6 │ \\n   ╰────\\n",
		      "moduleId": "./missingFile.js",
		      "moduleIdentifier": "<TEST_TOOLS_ROOT>/fixtures/errors/missingFile.js",
		      "moduleName": "./missingFile.js",
		      "moduleTrace": Array [],
		      "stack": undefined,
		    },
		    Object {
		      "loc": "12:17-33",
		      "message": "  × Module not found: Can't resolve './dir/missing2' in '<TEST_TOOLS_ROOT>/fixtures/errors'\\n    ╭─[12:9]\\n 10 │ \\n 11 │ // on line 12 char 10\\n 12 │          require(\\"./dir/missing2\\");\\n    ·          ─────────────────────────\\n    ╰────\\n",
		      "moduleId": "./missingFile.js",
		      "moduleIdentifier": "<TEST_TOOLS_ROOT>/fixtures/errors/missingFile.js",
		      "moduleName": "./missingFile.js",
		      "moduleTrace": Array [],
		      "stack": undefined,
		    },
		  ],
		  "warnings": Array [],
		}
	`);
	}
}, {
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
		      "loc": "4:8-19",
		      "message": "  × Module not found: Can't resolve './missing' in '<TEST_TOOLS_ROOT>/fixtures/errors'\\n   ╭─[4:0]\\n 2 │ \\n 3 │ // on line 4\\n 4 │ require(\\"./missing\\");\\n   · ────────────────────\\n 5 │ \\n 6 │ \\n   ╰────\\n",
		      "moduleId": "./missingFile.js",
		      "moduleIdentifier": "<TEST_TOOLS_ROOT>/fixtures/errors/missingFile.js",
		      "moduleName": "./missingFile.js",
		      "moduleTrace": Array [],
		      "stack": undefined,
		    },
		    Object {
		      "loc": "12:17-33",
		      "message": "  × Module not found: Can't resolve './dir/missing2' in '<TEST_TOOLS_ROOT>/fixtures/errors'\\n    ╭─[12:9]\\n 10 │ \\n 11 │ // on line 12 char 10\\n 12 │          require(\\"./dir/missing2\\");\\n    ·          ─────────────────────────\\n    ╰────\\n",
		      "moduleId": "./missingFile.js",
		      "moduleIdentifier": "<TEST_TOOLS_ROOT>/fixtures/errors/missingFile.js",
		      "moduleName": "./missingFile.js",
		      "moduleTrace": Array [],
		      "stack": undefined,
		    },
		  ],
		  "warnings": Array [],
		}
	`);
	}
}];
