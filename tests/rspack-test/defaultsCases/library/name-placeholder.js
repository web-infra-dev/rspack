defineDefaultsCase(Utils.casename(__filename), {
	description: "library.name contains [name] placeholder",
	options: () => ({
		output: {
			library: {
				name: ["my[name]Lib", "[name]", "lib"],
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
		+     "chunkLoadingGlobal": "webpackChunkmyLib_lib",
		@@ ... @@
		-     "devtoolNamespace": "",
		+     "devtoolNamespace": "myLib.lib",
		@@ ... @@
		-     "enabledLibraryTypes": Array [],
		+     "enabledLibraryTypes": Array [
		+       "var",
		+     ],
		@@ ... @@
		-     "hotUpdateGlobal": "webpackHotUpdate",
		+     "hotUpdateGlobal": "webpackHotUpdatemyLib_lib",
		@@ ... @@
		-     "library": undefined,
		+     "library": Object {
		+       "amdContainer": undefined,
		+       "auxiliaryComment": undefined,
		+       "export": undefined,
		+       "name": Array [
		+         "my[name]Lib",
		+         "[name]",
		+         "lib",
		+       ],
		+       "type": "var",
		+       "umdNamedDefine": undefined,
		+     },
		@@ ... @@
		-     "uniqueName": "",
		+     "uniqueName": "myLib.lib",
	`)
});
