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
		-     "css": true,
		+     "css": false,
		+     "futureDefaults": true,
		@@ ... @@
		-       },
		-       Object {
		-         "resolve": Object {
		-           "fullySpecified": true,
		-           "preferRelative": true,
		-         },
		-         "test": /\\.css$/i,
		-         "type": "css/auto",
		@@ ... @@
		-         "mimetype": "text/css+module",
		-         "resolve": Object {
		-           "fullySpecified": true,
		-           "preferRelative": true,
		-         },
		-         "type": "css/module",
		-       },
		-       Object {
		-         "mimetype": "text/css",
		-         "resolve": Object {
		-           "fullySpecified": true,
		-           "preferRelative": true,
		-         },
		-         "type": "css",
		-       },
		-       Object {
		@@ ... @@
		-     "generator": Object {
		-       "css": Object {
		-         "exportsConvention": "as-is",
		-         "exportsOnly": false,
		-       },
		-       "css/auto": Object {
		-         "exportsConvention": "as-is",
		-         "exportsOnly": false,
		-         "localIdentName": "[uniqueName]-[id]-[local]",
		-       },
		-       "css/module": Object {
		-         "exportsConvention": "as-is",
		-         "exportsOnly": false,
		-         "localIdentName": "[uniqueName]-[id]-[local]",
		-       },
		-     },
		+     "generator": Object {},
		@@ ... @@
		-         },
		-       },
		-       "css": Object {
		-         "namedExports": true,
		@@ ... @@
		-       "css/auto": Object {
		-         "namedExports": true,
		-       },
		-       "css/module": Object {
		-         "namedExports": true,
		@@ ... @@
		-         "css",
		@@ ... @@
		-     "hashDigestLength": 20,
		-     "hashFunction": "md4",
		+     "hashDigestLength": 16,
		+     "hashFunction": "xxhash64",
		@@ ... @@
		-         ],
		-       },
		-       "css-import": Object {
		-         "conditionNames": Array [
		-           "webpack",
		-           "production",
		-           "style",
		@@ ... @@
		-         "extensions": Array [
		-           ".css",
		-         ],
		-         "mainFields": Array [
		-           "style",
		-           "...",
		-         ],
		-         "mainFiles": Array [],
		-         "preferRelative": true,
	`)
};
