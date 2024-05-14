/** @type {import('../../..').TDefaultsCaseConfig} */
module.exports = {
	description: "library.name.root contains [name] placeholder",
	options: () => ({
		output: {
			library: {
				name: {
					root: ["[name]", "myLib"]
				},
				type: "var"
			}
		}
	}),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "chunkLoadingGlobal": "webpackChunk_rspack_test_tools",
		+     "chunkLoadingGlobal": "webpackChunkmyLib",
		@@ ... @@
		-     "devtoolNamespace": "@rspack/test-tools",
		+     "devtoolNamespace": "myLib",
		@@ ... @@
		-     "enabledLibraryTypes": Array [],
		+     "enabledLibraryTypes": Array [
		+       "var",
		+     ],
		@@ ... @@
		-     "hotUpdateGlobal": "webpackHotUpdate_rspack_test_tools",
		+     "hotUpdateGlobal": "webpackHotUpdatemyLib",
		@@ ... @@
		-     "library": undefined,
		+     "library": Object {
		+       "amdContainer": undefined,
		+       "auxiliaryComment": undefined,
		+       "export": undefined,
		+       "name": Object {
		+         "root": Array [
		+           "[name]",
		+           "myLib",
		+         ],
		+       },
		+       "type": "var",
		+       "umdNamedDefine": undefined,
		+     },
		@@ ... @@
		-     "uniqueName": "@rspack/test-tools",
		+     "uniqueName": "myLib",
	`)
};
