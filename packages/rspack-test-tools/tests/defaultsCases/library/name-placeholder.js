module.exports = {
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
    -     "chunkLoadingGlobal": "webpackChunk_rspack_core",
    +     "chunkLoadingGlobal": "webpackChunkmyLib_lib",
    @@ ... @@
    -     "devtoolNamespace": "@rspack/core",
    +     "devtoolNamespace": "myLib.lib",
    @@ ... @@
    -     "enabledLibraryTypes": Array [],
    +     "enabledLibraryTypes": Array [
    +       "var",
    +     ],
    @@ ... @@
    -     "hotUpdateGlobal": "webpackHotUpdate_rspack_core",
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
    -     "uniqueName": "@rspack/core",
    +     "uniqueName": "myLib.lib",
  `)
};
