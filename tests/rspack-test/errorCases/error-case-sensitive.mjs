import fs from "node:fs";
import path from "node:path";
import { CaseSensitivePlugin } from "@rspack/core";

const isCaseInsensitiveFilesystem = fs.existsSync(
	path.resolve(import.meta.dirname, "../fixtures", "errors", "FILE.js")
);

export default isCaseInsensitiveFilesystem
	? {
		description: "should emit warning for case-preserved disk",
		options() {
			return {
				mode: "development",
				entry: "./case-sensitive",
				plugins: [new CaseSensitivePlugin()]
			};
		},
		async check(diagnostics) {
			expect(diagnostics).toMatchInlineSnapshot(`
				Object {
				  "errors": Array [],
				  "warnings": Array [
				    Object {
				      "code": "Sensitive Warn",
				      "message": "  ⚠ There are multiple modules with names that only differ in casing.  │   - <TEST_ROOT>/fixtures/errors/FILE.js  │     - used by <TEST_ROOT>/fixtures/errors/case-sensitive.js  │   - <TEST_ROOT>/fixtures/errors/file.js  │     - used by <TEST_ROOT>/fixtures/errors/case-sensitive.js  │ ",
				      "moduleTrace": Array [],
				      "stack": undefined,
				    },
				  ],
				}
			`);
		}
	}
	: {
		description: "should emit error for case-sensitive",
		options() {
			return {
				mode: "development",
				entry: "./case-sensitive"
			};
		},
		async check(diagnostics) {
			expect(diagnostics).toMatchInlineSnapshot(`
			Object {
			  "errors": Array [
			    Object {
			      "loc": "2:9-17",
			      "message": "  × Module not found: Can't resolve './FILE' in '<TEST_ROOT>/fixtures/errors'   ╭─[2:0] 1 │ require(\\"./file\\"); 2 │ require(\\"./FILE\\");   · ─────────────────   ╰────",
			      "moduleId": "./case-sensitive.js",
			      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/case-sensitive.js",
			      "moduleName": "./case-sensitive.js",
			      "moduleTrace": Array [],
			      "stack": undefined,
			    },
			  ],
			  "warnings": Array [],
			}
		`);
		}
	};
