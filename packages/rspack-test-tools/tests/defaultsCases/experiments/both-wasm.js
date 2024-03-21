module.exports = {
	description: "both wasm",
	options: () => ({
		experiments: { syncWebAssembly: true, asyncWebAssembly: true }
	}),
	diff: e =>
		e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			-     "asyncWebAssembly": false,
			+     "asyncWebAssembly": true,
			@@ ... @@
			+     "syncWebAssembly": true,
			@@ ... @@
			+       },
			+       Object {
			+         "rules": Array [
			+           Object {
			+             "descriptionData": Object {
			+               "type": "module",
			+             },
			+             "resolve": Object {
			+               "fullySpecified": true,
			+             },
			+           },
			+         ],
			+         "test": /\\.wasm$/i,
			+         "type": "webassembly/async",
			+       },
			+       Object {
			+         "mimetype": "application/wasm",
			+         "rules": Array [
			+           Object {
			+             "descriptionData": Object {
			+               "type": "module",
			+             },
			+             "resolve": Object {
			+               "fullySpecified": true,
			+             },
			+           },
			+         ],
			+         "type": "webassembly/async",
		`)
};
