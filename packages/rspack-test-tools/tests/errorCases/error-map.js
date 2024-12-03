let errors = [];

/** @type {import('../..').TErrorCaseConfig} */
module.exports = {
	description:
		"Testing map function on errors and warnings: test map of errors",
	options() {
		return {
			entry: "./resolve-fail-esm",
			plugins: [
				compiler => {
					compiler.hooks.afterCompile.tap("test errors map", compilation => {
						errors = compilation.errors.map((item, index) => ({
							index,
							...item
						}));
					});
				}
			]
		};
	},
	async check() {
		expect(errors).toMatchInlineSnapshot(`
		Array [
		  Object {
		  "index": 0,
		  "loc": "1:0-33",
		  "message": "  × Module not found: Can't resolve './answer' in '<TEST_TOOLS_ROOT>/tests/fixtures/errors/resolve-fail-esm'\\n   ╭────\\n 1 │ import { answer } from './answer'\\n   ·                        ──────────\\n   ╰────\\n  help: Did you mean './answer.js'?\\n        \\n        The request './answer' failed to resolve only because it was resolved as fully specified,\\n        probably because the origin is strict EcmaScript Module,\\n        e. g. a module with javascript mimetype, a '*.mjs' file, or a '*.js' file where the package.json contains '\\"type\\": \\"module\\"'.\\n        \\n        The extension in the request is mandatory for it to be fully specified.\\n        Add the extension to the request.\\n",
		  "moduleIdentifier": "javascript/esm|<TEST_TOOLS_ROOT>/tests/fixtures/errors/resolve-fail-esm/index.js",
		  "name": "Error",
		},
		]
	`);
	}
};
