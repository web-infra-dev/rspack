defineDefaultsCase(Utils.casename(__filename), {
	description: "library",
	options: () => ({ output: { library: ["myLib", "awesome"] } }),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-         "force": true,
		+         "force": false,
		@@ ... @@
		-     "chunkLoadingGlobal": "webpackChunk",
		+     "chunkLoadingGlobal": "webpackChunkmyLib_awesome",
		@@ ... @@
		-     "devtoolNamespace": "",
		+     "devtoolNamespace": "myLib.awesome",
		@@ ... @@
		-     "enabledLibraryTypes": Array [],
		+     "enabledLibraryTypes": Array [
		+       "var",
		+     ],
		@@ ... @@
		-     "hotUpdateGlobal": "webpackHotUpdate",
		+     "hotUpdateGlobal": "webpackHotUpdatemyLib_awesome",
		@@ ... @@
		-     "library": undefined,
		+     "library": Object {
		+       "amdContainer": undefined,
		+       "auxiliaryComment": undefined,
		+       "export": undefined,
		+       "name": Array [
		+         "myLib",
		+         "awesome",
		+       ],
		+       "type": "var",
		+       "umdNamedDefine": undefined,
		+     },
		@@ ... @@
		-     "uniqueName": "",
		+     "uniqueName": "myLib.awesome",
	`)
});
