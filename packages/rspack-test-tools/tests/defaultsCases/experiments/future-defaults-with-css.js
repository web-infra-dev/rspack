/** @type {import('../../..').TDefaultsCaseConfig} */
module.exports = {
	description: "experiments.futureDefaults w/ experiments.css disabled",
	options: () => ({
		experiments: {
			css: false,
			futureDefaults: true
		}
	}),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "asyncWebAssembly": false,
		+     "asyncWebAssembly": true,
		@@ ... @@
		-     "css": undefined,
		-     "futureDefaults": false,
		+     "css": false,
		+     "futureDefaults": true,
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
		+         "test": //.wasm$/i,
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
