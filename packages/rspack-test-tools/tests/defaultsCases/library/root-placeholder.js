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
    -     "uniqueName": "@rspack/core",
    +     "uniqueName": "myLib",
  `)
};
