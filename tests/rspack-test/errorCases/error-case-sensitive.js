const fs = require("fs");
const path = require("path");
const { WarnCaseSensitiveModulesPlugin } = require("@rspack/core");

const isCaseInsensitiveFilesystem = fs.existsSync(
  path.resolve(__dirname, "../fixtures", "errors", "FILE.js")
);

module.exports = isCaseInsensitiveFilesystem
  ? {
    description: "should emit warning for case-preserved disk",
    options() {
      return {
        mode: "development",
        entry: "./case-sensitive",
        plugins: [new WarnCaseSensitiveModulesPlugin()]
      };
    },
    async check(diagnostics) {
      expect(diagnostics).toMatchInlineSnapshot(`
			Object {
			  "errors": Array [],
			  "warnings": Array [
			    Object {
			      "code": "Sensitive Modules Warn",
			      "message": "  ⚠ There are multiple modules with names that only differ in casing.\\n  │   - <TEST_TOOLS_ROOT>/fixtures/errors/FILE.js\\n  │     - used by <TEST_TOOLS_ROOT>/fixtures/errors/case-sensitive.js\\n  │   - <TEST_TOOLS_ROOT>/fixtures/errors/file.js\\n  │     - used by <TEST_TOOLS_ROOT>/fixtures/errors/case-sensitive.js\\n  │ \\n",
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
			      "loc": "2:8-16",
			      "message": "  × Module not found: Can't resolve './FILE' in '<TEST_TOOLS_ROOT>/fixtures/errors'\\n   ╭─[2:0]\\n 1 │ require(\\"./file\\");\\n 2 │ require(\\"./FILE\\");\\n   · ─────────────────\\n   ╰────\\n",
			      "moduleId": "./case-sensitive.js",
			      "moduleIdentifier": "<TEST_TOOLS_ROOT>/fixtures/errors/case-sensitive.js",
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
