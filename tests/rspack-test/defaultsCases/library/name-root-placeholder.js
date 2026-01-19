/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
module.exports = {
	description: "library.name.root contains escaped placeholder",
	options: () => ({
		output: {
			library: {
				name: {
					root: ["[\\name\\]", "my[\\name\\]Lib[name]", "[\\name\\]"]
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
			-       "force": true,
			+       "force": false,
			@@ ... @@
			-     "chunkLoadingGlobal": "webpackChunk_rspack_tests",
			+     "chunkLoadingGlobal": "webpackChunk_name_my_name_Lib_name_",
			@@ ... @@
			-     "devtoolNamespace": "@rspack/tests",
			+     "devtoolNamespace": "[name].my[name]Lib.[name]",
			@@ ... @@
			+     ],
			+     "enabledLibraryTypes": Array [
			+       "var",
			@@ ... @@
			-     "enabledLibraryTypes": Array [],
			@@ ... @@
			-     "hotUpdateGlobal": "rspackHotUpdate_rspack_tests",
			+     "hotUpdateGlobal": "rspackHotUpdate_name_my_name_Lib_name_",
			@@ ... @@
			-     "library": undefined,
			+     "library": Object {
			+       "name": Object {
			+         "root": Array [
			+           "[\\\\name\\\\]",
			+           "my[\\\\name\\\\]Lib[name]",
			+           "[\\\\name\\\\]",
			+         ],
			+       },
			+       "type": "var",
			+     },
			@@ ... @@
			-     "uniqueName": "@rspack/tests",
			+     "uniqueName": "[name].my[name]Lib.[name]",
		`)
};
