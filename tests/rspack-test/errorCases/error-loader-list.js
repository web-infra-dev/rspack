const path = require("path");

const identityLoader = path.resolve(
	__dirname,
	"../fixtures/errors/identity-loader.js"
);
const addCommentLoader = path.resolve(
	__dirname,
	"../fixtures/errors/add-comment-loader.js"
);

/** @type {import('@rspack/test-tools').TErrorCaseConfig[]} */
module.exports = [
	{
		description:
			"should show loader used if it is present when module parsing fails",
		options() {
			return {
				mode: "development",
				entry: "./abc.html",
				module: {
					rules: [
						{
							test: /\.html$/,
							use: [{ loader: identityLoader }]
						}
					]
				}
			};
		},
		async check(diagnostics) {
			expect(diagnostics).toMatchInlineSnapshot(`
			Object {
			  "errors": Array [
			    Object {
			      "code": "ModuleParseError",
			      "message": "  × Module parse failed:\\n  ╰─▶   × JavaScript parse error: Expected ';', '}' or <eof>\\n         ╭─[1:10]\\n       1 │ <!DOCTYPE html>\\n         ·           ────\\n       2 │ <html>\\n       3 │     <body>\\n         ╰────\\n      \\n  help: \\n        File was processed with these loaders:\\n         * <TEST_TOOLS_ROOT>/fixtures/errors/identity-loader.js\\n        You may need an additional loader to handle the result of these loaders.\\n",
			      "moduleId": "./abc.html",
			      "moduleIdentifier": "<TEST_TOOLS_ROOT>/fixtures/errors/identity-loader.js!<TEST_TOOLS_ROOT>/fixtures/errors/abc.html",
			      "moduleName": "./abc.html",
			      "moduleTrace": Array [],
			      "stack": undefined,
			    },
			    Object {
			      "code": "ModuleParseError",
			      "message": "  × Module parse failed:\\n  ╰─▶   × JavaScript parse error: Expression expected\\n         ╭─[1:0]\\n       1 │ <!DOCTYPE html>\\n         · ─\\n       2 │ <html>\\n       3 │     <body>\\n         ╰────\\n      \\n  help: \\n        File was processed with these loaders:\\n         * <TEST_TOOLS_ROOT>/fixtures/errors/identity-loader.js\\n        You may need an additional loader to handle the result of these loaders.\\n",
			      "moduleId": "./abc.html",
			      "moduleIdentifier": "<TEST_TOOLS_ROOT>/fixtures/errors/identity-loader.js!<TEST_TOOLS_ROOT>/fixtures/errors/abc.html",
			      "moduleName": "./abc.html",
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
		description:
			"should show all loaders used if they are in config when module parsing fails",
		options() {
			return {
				mode: "development",
				entry: "./abc.html",
				module: {
					rules: [
						{
							test: /\.html$/,
							use: [{ loader: identityLoader }, { loader: addCommentLoader }]
						}
					]
				}
			};
		},
		async check(diagnostics) {
			expect(diagnostics).toMatchInlineSnapshot(`
			Object {
			  "errors": Array [
			    Object {
			      "code": "ModuleParseError",
			      "message": "  × Module parse failed:\\n  ╰─▶   × JavaScript parse error: Expected ';', '}' or <eof>\\n         ╭─[1:10]\\n       1 │ <!DOCTYPE html>\\n         ·           ────\\n       2 │ <html>\\n       3 │     <body>\\n         ╰────\\n      \\n  help: \\n        File was processed with these loaders:\\n         * <TEST_TOOLS_ROOT>/fixtures/errors/identity-loader.js\\n         * <TEST_TOOLS_ROOT>/fixtures/errors/add-comment-loader.js\\n        You may need an additional loader to handle the result of these loaders.\\n",
			      "moduleId": "./abc.html",
			      "moduleIdentifier": "<TEST_TOOLS_ROOT>/fixtures/errors/identity-loader.js!<TEST_TOOLS_ROOT>/fixtures/errors/add-comment-loader.js!<TEST_TOOLS_ROOT>/fixtures/errors/abc.html",
			      "moduleName": "./abc.html",
			      "moduleTrace": Array [],
			      "stack": undefined,
			    },
			    Object {
			      "code": "ModuleParseError",
			      "message": "  × Module parse failed:\\n  ╰─▶   × JavaScript parse error: Expression expected\\n         ╭─[1:0]\\n       1 │ <!DOCTYPE html>\\n         · ─\\n       2 │ <html>\\n       3 │     <body>\\n         ╰────\\n      \\n  help: \\n        File was processed with these loaders:\\n         * <TEST_TOOLS_ROOT>/fixtures/errors/identity-loader.js\\n         * <TEST_TOOLS_ROOT>/fixtures/errors/add-comment-loader.js\\n        You may need an additional loader to handle the result of these loaders.\\n",
			      "moduleId": "./abc.html",
			      "moduleIdentifier": "<TEST_TOOLS_ROOT>/fixtures/errors/identity-loader.js!<TEST_TOOLS_ROOT>/fixtures/errors/add-comment-loader.js!<TEST_TOOLS_ROOT>/fixtures/errors/abc.html",
			      "moduleName": "./abc.html",
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
		description: "should show all loaders used if use is a string",
		options() {
			return {
				mode: "development",
				entry: "./abc.html",
				module: {
					rules: [
						{ test: /\.html$/, use: identityLoader },
						{ test: /\.html$/, use: addCommentLoader }
					]
				}
			};
		},
		async check(diagnostics) {
			expect(diagnostics).toMatchInlineSnapshot(`
			Object {
			  "errors": Array [
			    Object {
			      "code": "ModuleParseError",
			      "message": "  × Module parse failed:\\n  ╰─▶   × JavaScript parse error: Expected ';', '}' or <eof>\\n         ╭─[1:10]\\n       1 │ <!DOCTYPE html>\\n         ·           ────\\n       2 │ <html>\\n       3 │     <body>\\n         ╰────\\n      \\n  help: \\n        File was processed with these loaders:\\n         * <TEST_TOOLS_ROOT>/fixtures/errors/identity-loader.js\\n         * <TEST_TOOLS_ROOT>/fixtures/errors/add-comment-loader.js\\n        You may need an additional loader to handle the result of these loaders.\\n",
			      "moduleId": "./abc.html",
			      "moduleIdentifier": "<TEST_TOOLS_ROOT>/fixtures/errors/identity-loader.js!<TEST_TOOLS_ROOT>/fixtures/errors/add-comment-loader.js!<TEST_TOOLS_ROOT>/fixtures/errors/abc.html",
			      "moduleName": "./abc.html",
			      "moduleTrace": Array [],
			      "stack": undefined,
			    },
			    Object {
			      "code": "ModuleParseError",
			      "message": "  × Module parse failed:\\n  ╰─▶   × JavaScript parse error: Expression expected\\n         ╭─[1:0]\\n       1 │ <!DOCTYPE html>\\n         · ─\\n       2 │ <html>\\n       3 │     <body>\\n         ╰────\\n      \\n  help: \\n        File was processed with these loaders:\\n         * <TEST_TOOLS_ROOT>/fixtures/errors/identity-loader.js\\n         * <TEST_TOOLS_ROOT>/fixtures/errors/add-comment-loader.js\\n        You may need an additional loader to handle the result of these loaders.\\n",
			      "moduleId": "./abc.html",
			      "moduleIdentifier": "<TEST_TOOLS_ROOT>/fixtures/errors/identity-loader.js!<TEST_TOOLS_ROOT>/fixtures/errors/add-comment-loader.js!<TEST_TOOLS_ROOT>/fixtures/errors/abc.html",
			      "moduleName": "./abc.html",
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
		description:
			"should show 'no loaders are configured to process this file' if loaders are not included in config when module parsing fails",
		options() {
			return {
				mode: "development",
				entry: "./abc.html",
				module: {}
			};
		},
		async check(diagnostics) {
			expect(diagnostics).toMatchInlineSnapshot(`
			Object {
			  "errors": Array [
			    Object {
			      "code": "ModuleParseError",
			      "message": "  × Module parse failed:\\n  ╰─▶   × JavaScript parse error: Expected ';', '}' or <eof>\\n         ╭─[1:10]\\n       1 │ <!DOCTYPE html>\\n         ·           ────\\n       2 │ <html>\\n       3 │     <body>\\n         ╰────\\n      \\n  help: \\n        You may need an appropriate loader to handle this file type.\\n",
			      "moduleId": "./abc.html",
			      "moduleIdentifier": "<TEST_TOOLS_ROOT>/fixtures/errors/abc.html",
			      "moduleName": "./abc.html",
			      "moduleTrace": Array [],
			      "stack": undefined,
			    },
			    Object {
			      "code": "ModuleParseError",
			      "message": "  × Module parse failed:\\n  ╰─▶   × JavaScript parse error: Expression expected\\n         ╭─[1:0]\\n       1 │ <!DOCTYPE html>\\n         · ─\\n       2 │ <html>\\n       3 │     <body>\\n         ╰────\\n      \\n  help: \\n        You may need an appropriate loader to handle this file type.\\n",
			      "moduleId": "./abc.html",
			      "moduleIdentifier": "<TEST_TOOLS_ROOT>/fixtures/errors/abc.html",
			      "moduleName": "./abc.html",
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
		description:
			"should show 'source code omitted for this binary file' when module parsing fails for binary files",
		options() {
			return {
				mode: "development",
				entry: path.resolve(__dirname, "../fixtures/font.ttf"),
				module: {}
			};
		},
		async check(diagnostics) {
			expect(diagnostics).toMatchInlineSnapshot(`
			Object {
			  "errors": Array [
			    Object {
			      "code": "ModuleParseError",
			      "message": "  × Module parse failed:\\n  ╰─▶   × JavaScript parse error: Unexpected character '/0'\\n         ╭─[1:0]\\n       1 │     \\n         · ▲\\n       2 │  �  PGPOS�\\n       3 │ ��  \`X  :XGSUB!?-�  ��  �OS/2k��  X   \`cmap)9�8  $  �gasp  !  \`H   glyf��H    �  PheadNE   �   6hhea�C     $hmtx}�*   �  lloca�d��  �  8maxp7�  8    name P<�  Z  �post#_�s  [�  {    N��O�_<� �    �Vn/    �њU�P���               ��  ��P�P�                   � \\n         ╰────\\n      \\n  help: \\n        You may need an appropriate loader to handle this file type.\\n",
			      "moduleId": "../font.ttf",
			      "moduleIdentifier": "<TEST_TOOLS_ROOT>/fixtures/font.ttf",
			      "moduleName": "../font.ttf",
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
