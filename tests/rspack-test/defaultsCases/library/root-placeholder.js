defineDefaultsCase(Utils.casename(__filename), {
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
		-         "force": true,
		+         "force": false,
		@@ ... @@
		-     "chunkLoadingGlobal": "webpackChunk",
		+     "chunkLoadingGlobal": "webpackChunkmyLib",
		@@ ... @@
		-     "devtoolNamespace": "",
		+     "devtoolNamespace": "myLib",
		@@ ... @@
		-     "enabledLibraryTypes": Array [],
		+     "enabledLibraryTypes": Array [
		+       "var",
		+     ],
		@@ ... @@
		-     "hotUpdateGlobal": "webpackHotUpdate",
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
		-     "uniqueName": "",
		+     "uniqueName": "myLib",
	`)
});
