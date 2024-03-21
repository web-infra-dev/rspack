module.exports = {
	description: "library contains [name] placeholder",
	options: () => ({
		output: {
			library: ["myLib", "[name]"]
		}
	}),
	diff: e =>
		e.toMatchInlineSnapshot(`
    - Expected
    + Received

    @@ ... @@
    -     "chunkLoadingGlobal": "webpackChunk_rspack_core",
    +     "chunkLoadingGlobal": "webpackChunkmyLib",
    @@ ... @@
    -     "devtoolNamespace": "@rspack/core",
    +     "devtoolNamespace": "myLib",
    @@ ... @@
    -     "enabledLibraryTypes": Array [],
    +     "enabledLibraryTypes": Array [
    +       "var",
    +     ],
    @@ ... @@
    -     "hotUpdateGlobal": "webpackHotUpdate_rspack_core",
    +     "hotUpdateGlobal": "webpackHotUpdatemyLib",
    @@ ... @@
    -     "library": undefined,
    +     "library": Object {
    +       "amdContainer": undefined,
    +       "auxiliaryComment": undefined,
    +       "export": undefined,
    +       "name": Array [
    +         "myLib",
    +         "[name]",
    +       ],
    +       "type": "var",
    +       "umdNamedDefine": undefined,
    +     },
    @@ ... @@
    -     "uniqueName": "@rspack/core",
    +     "uniqueName": "myLib",
  `)
};
