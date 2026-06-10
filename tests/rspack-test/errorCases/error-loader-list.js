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
				      "message": "  × Module parse failed:  ╰─▶   × JavaScript parse error: Expression expected         ╭─[1:0]       1 │ <!DOCTYPE html>         · ─       2 │ <html>       3 │     <body>         ╰────        help:         File was processed with these loaders:         * <TEST_ROOT>/fixtures/errors/identity-loader.js        You may need an additional loader to handle the result of these loaders.",
				      "moduleId": "./abc.html",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/identity-loader.js!<TEST_ROOT>/fixtures/errors/abc.html",
				      "moduleName": "./abc.html",
				      "moduleTrace": Array [],
				      "stack": undefined,
				    },
				    Object {
				      "code": "ModuleParseError",
				      "message": "  × Module parse failed:  ╰─▶   × JavaScript parse error: Expected ';', '}' or <eof>         ╭─[1:10]       1 │ <!DOCTYPE html>         ·           ────       2 │ <html>       3 │     <body>         ╰────        help:         File was processed with these loaders:         * <TEST_ROOT>/fixtures/errors/identity-loader.js        You may need an additional loader to handle the result of these loaders.",
				      "moduleId": "./abc.html",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/identity-loader.js!<TEST_ROOT>/fixtures/errors/abc.html",
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
				      "message": "  × Module parse failed:  ╰─▶   × JavaScript parse error: Expression expected         ╭─[1:0]       1 │ <!DOCTYPE html>         · ─       2 │ <html>       3 │     <body>         ╰────        help:         File was processed with these loaders:         * <TEST_ROOT>/fixtures/errors/identity-loader.js         * <TEST_ROOT>/fixtures/errors/add-comment-loader.js        You may need an additional loader to handle the result of these loaders.",
				      "moduleId": "./abc.html",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/identity-loader.js!<TEST_ROOT>/fixtures/errors/add-comment-loader.js!<TEST_ROOT>/fixtures/errors/abc.html",
				      "moduleName": "./abc.html",
				      "moduleTrace": Array [],
				      "stack": undefined,
				    },
				    Object {
				      "code": "ModuleParseError",
				      "message": "  × Module parse failed:  ╰─▶   × JavaScript parse error: Expected ';', '}' or <eof>         ╭─[1:10]       1 │ <!DOCTYPE html>         ·           ────       2 │ <html>       3 │     <body>         ╰────        help:         File was processed with these loaders:         * <TEST_ROOT>/fixtures/errors/identity-loader.js         * <TEST_ROOT>/fixtures/errors/add-comment-loader.js        You may need an additional loader to handle the result of these loaders.",
				      "moduleId": "./abc.html",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/identity-loader.js!<TEST_ROOT>/fixtures/errors/add-comment-loader.js!<TEST_ROOT>/fixtures/errors/abc.html",
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
				      "message": "  × Module parse failed:  ╰─▶   × JavaScript parse error: Expression expected         ╭─[1:0]       1 │ <!DOCTYPE html>         · ─       2 │ <html>       3 │     <body>         ╰────        help:         File was processed with these loaders:         * <TEST_ROOT>/fixtures/errors/identity-loader.js         * <TEST_ROOT>/fixtures/errors/add-comment-loader.js        You may need an additional loader to handle the result of these loaders.",
				      "moduleId": "./abc.html",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/identity-loader.js!<TEST_ROOT>/fixtures/errors/add-comment-loader.js!<TEST_ROOT>/fixtures/errors/abc.html",
				      "moduleName": "./abc.html",
				      "moduleTrace": Array [],
				      "stack": undefined,
				    },
				    Object {
				      "code": "ModuleParseError",
				      "message": "  × Module parse failed:  ╰─▶   × JavaScript parse error: Expected ';', '}' or <eof>         ╭─[1:10]       1 │ <!DOCTYPE html>         ·           ────       2 │ <html>       3 │     <body>         ╰────        help:         File was processed with these loaders:         * <TEST_ROOT>/fixtures/errors/identity-loader.js         * <TEST_ROOT>/fixtures/errors/add-comment-loader.js        You may need an additional loader to handle the result of these loaders.",
				      "moduleId": "./abc.html",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/identity-loader.js!<TEST_ROOT>/fixtures/errors/add-comment-loader.js!<TEST_ROOT>/fixtures/errors/abc.html",
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
				      "message": "  × Module parse failed:  ╰─▶   × JavaScript parse error: Expression expected         ╭─[1:0]       1 │ <!DOCTYPE html>         · ─       2 │ <html>       3 │     <body>         ╰────        help:         You may need an appropriate loader to handle this file type.",
				      "moduleId": "./abc.html",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/abc.html",
				      "moduleName": "./abc.html",
				      "moduleTrace": Array [],
				      "stack": undefined,
				    },
				    Object {
				      "code": "ModuleParseError",
				      "message": "  × Module parse failed:  ╰─▶   × JavaScript parse error: Expected ';', '}' or <eof>         ╭─[1:10]       1 │ <!DOCTYPE html>         ·           ────       2 │ <html>       3 │     <body>         ╰────        help:         You may need an appropriate loader to handle this file type.",
				      "moduleId": "./abc.html",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/errors/abc.html",
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
				      "message": "  × Module parse failed:  ╰─▶   × JavaScript parse error: Unexpected character '/0'         ╭─[1:0]       1 │              · ▲       2 │  �  PGPOS�       3 │ ��  \`X  :XGSUB!?-�  ��  �OS/2k��  X   \`cmap)9�8  $  �gasp  !  \`H   glyf��H    �  PheadNE   �   6hhea�C     $hmtx}�*   �  lloca�d��  �  8maxp7�  8    name P<�  Z  �post#_�s  [�  {    N��O�_<� �    �Vn/    �њU�P���               ��  ��P�P�                   �          ╰────        help:         You may need an appropriate loader to handle this file type.",
				      "moduleId": "../font.ttf",
				      "moduleIdentifier": "<TEST_ROOT>/fixtures/font.ttf",
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
