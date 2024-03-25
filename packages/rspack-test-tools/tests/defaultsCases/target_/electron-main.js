module.exports = {
	description: "target electron-main",
	options: () => ({ target: "electron-main" }),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "electron": false,
		-     "electronMain": false,
		+     "electron": true,
		+     "electronMain": true,
		@@ ... @@
		-     "node": false,
		-     "web": true,
		+     "node": true,
		+     "web": false,
		@@ ... @@
		-     "__dirname": "warn-mock",
		-     "__filename": "warn-mock",
		-     "global": "warn",
		+     "__dirname": "eval-only",
		+     "__filename": "eval-only",
		+     "global": false,
		@@ ... @@
		-     "chunkFormat": "array-push",
		-     "chunkLoading": "jsonp",
		+     "chunkFormat": "commonjs",
		+     "chunkLoading": "require",
		@@ ... @@
		-       "jsonp",
		-       "import-scripts",
		+       "require",
		@@ ... @@
		-       "fetch",
		+       "async-node",
		@@ ... @@
		-     "globalObject": "self",
		+     "globalObject": "global",
		@@ ... @@
		-     "publicPath": "auto",
		+     "publicPath": "",
		@@ ... @@
		-     "wasmLoading": "fetch",
		+     "wasmLoading": "async-node",
		@@ ... @@
		-     "workerChunkLoading": "import-scripts",
		+     "workerChunkLoading": "require",
		@@ ... @@
		-     "workerWasmLoading": "fetch",
		+     "workerWasmLoading": "async-node",
		@@ ... @@
		-         "aliasFields": Array [
		-           "browser",
		-         ],
		+         "aliasFields": Array [],
		@@ ... @@
		-           "browser",
		@@ ... @@
		-         "aliasFields": Array [
		-           "browser",
		-         ],
		+         "aliasFields": Array [],
		@@ ... @@
		-           "browser",
		@@ ... @@
		-         "aliasFields": Array [
		-           "browser",
		-         ],
		+         "aliasFields": Array [],
		@@ ... @@
		-           "browser",
		@@ ... @@
		-         "aliasFields": Array [
		-           "browser",
		-         ],
		+         "aliasFields": Array [],
		@@ ... @@
		-           "browser",
		@@ ... @@
		-         "aliasFields": Array [
		-           "browser",
		-         ],
		+         "aliasFields": Array [],
		@@ ... @@
		-           "browser",
		@@ ... @@
		-       "browser",
		+       "node",
		+       "electron",
		@@ ... @@
		-   "target": "web",
		+   "target": "electron-main",
	`)
};
