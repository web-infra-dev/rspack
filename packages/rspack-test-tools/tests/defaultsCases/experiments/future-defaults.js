/** @type {import('../../..').TDefaultsCaseConfig} */
module.exports = {
	description: "experiments.futureDefaults",
	options: () => ({
		experiments: {
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
		+     "css": true,
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
		+       },
		+       Object {
		+         "resolve": Object {
		+           "fullySpecified": true,
		+           "preferRelative": true,
		+         },
		+         "test": /\\.css$/i,
		+         "type": "css/auto",
		+       },
		+       Object {
		+         "mimetype": "text/css+module",
		+         "resolve": Object {
		+           "fullySpecified": true,
		+           "preferRelative": true,
		+         },
		+         "type": "css/module",
		+       },
		+       Object {
		+         "mimetype": "text/css",
		+         "resolve": Object {
		+           "fullySpecified": true,
		+           "preferRelative": true,
		+         },
		+         "type": "css",
		@@ ... @@
		+       "css": Object {
		+         "esModule": true,
		+         "exportsOnly": false,
		+       },
		+       "css/auto": Object {
		+         "esModule": true,
		+         "exportsConvention": "as-is",
		+         "exportsOnly": false,
		+         "localIdentName": "[uniqueName]-[id]-[local]",
		+       },
		+       "css/module": Object {
		+         "esModule": true,
		+         "exportsConvention": "as-is",
		+         "exportsOnly": false,
		+         "localIdentName": "[uniqueName]-[id]-[local]",
		+       },
		@@ ... @@
		+         },
		+       },
		+       "css": Object {
		+         "namedExports": true,
		@@ ... @@
		+       "css/auto": Object {
		+         "namedExports": true,
		+       },
		+       "css/module": Object {
		+         "namedExports": true,
		@@ ... @@
		+         "css",
		@@ ... @@
		-     "charset": true,
		+     "charset": false,
		@@ ... @@
		+           "...",
		+         ],
		+       },
		+       "css-import": Object {
		+         "conditionNames": Array [
		+           "webpack",
		+           "production",
		+           "style",
		+         ],
		+         "extensions": Array [
		+           ".css",
		+         ],
		+         "mainFields": Array [
		+           "style",
		@@ ... @@
		+         "mainFiles": Array [],
		+         "preferRelative": true,
	`)
};
