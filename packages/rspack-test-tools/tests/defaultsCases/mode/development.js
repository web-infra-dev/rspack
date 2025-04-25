/** @type {import('../../..').TDefaultsCaseConfig} */
module.exports = {
	description: "development",
	options: () => ({ mode: "development" }),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "cache": false,
		+   "cache": true,
		@@ ... @@
		-   "devtool": false,
		+   "devtool": "eval",
		@@ ... @@
		-     "cache": false,
		+     "cache": true,
		@@ ... @@
		-     "incremental": false,
		+     "incremental": Object {
		+       "buildChunkGraph": false,
		+       "chunkIds": false,
		+       "chunksHashes": false,
		+       "chunksRender": false,
		+       "chunksRuntimeRequirements": false,
		+       "dependenciesDiagnostics": false,
		+       "emitAssets": true,
		+       "inferAsyncModules": false,
		+       "make": true,
		+       "moduleIds": false,
		+       "modulesCodegen": false,
		+       "modulesHashes": false,
		+       "modulesRuntimeRequirements": false,
		+       "providedExports": false,
		+       "sideEffects": false,
		+     },
		@@ ... @@
		-   "mode": "none",
		+   "mode": "development",
		@@ ... @@
		-         "exportsDepth": 9007199254740991,
		+         "exportsDepth": 1,
		@@ ... @@
		-     "chunkIds": "natural",
		+     "chunkIds": "named",
		@@ ... @@
		-     "moduleIds": "natural",
		-     "nodeEnv": false,
		+     "moduleIds": "named",
		+     "nodeEnv": "development",
		@@ ... @@
		-     "pathinfo": false,
		+     "pathinfo": true,
		@@ ... @@
		-       "production",
		+       "development",
	`)
};
