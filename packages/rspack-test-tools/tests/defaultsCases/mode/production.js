/** @type {import('../../..').TDefaultsCaseConfig} */
module.exports = {
	description: "production",
	options: () => ({ mode: "production" }),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "incremental": Object {
		-       "buildChunkGraph": false,
		-       "chunksHashes": false,
		-       "chunksRender": false,
		-       "chunksRuntimeRequirements": false,
		-       "dependenciesDiagnostics": false,
		-       "emitAssets": true,
		-       "inferAsyncModules": false,
		-       "make": true,
		-       "modulesCodegen": false,
		-       "modulesHashes": false,
		-       "modulesRuntimeRequirements": false,
		-       "providedExports": false,
		-     },
		+     "incremental": false,
		@@ ... @@
		-   "mode": "none",
		+   "mode": "production",
		@@ ... @@
		-     "chunkIds": "natural",
		-     "concatenateModules": false,
		-     "emitOnErrors": true,
		-     "innerGraph": false,
		-     "mangleExports": false,
		+     "chunkIds": "deterministic",
		+     "concatenateModules": true,
		+     "emitOnErrors": false,
		+     "innerGraph": true,
		+     "mangleExports": true,
		@@ ... @@
		-     "minimize": false,
		+     "minimize": true,
		@@ ... @@
		-     "moduleIds": "natural",
		-     "nodeEnv": false,
		+     "moduleIds": "deterministic",
		+     "nodeEnv": "production",
		@@ ... @@
		-     "realContentHash": false,
		+     "realContentHash": true,
		@@ ... @@
		-     "sideEffects": "flag",
		+     "sideEffects": true,
		@@ ... @@
		-       "hidePathInfo": false,
		-       "maxAsyncRequests": Infinity,
		-       "maxInitialRequests": Infinity,
		+       "hidePathInfo": true,
		+       "maxAsyncRequests": 30,
		+       "maxInitialRequests": 30,
		@@ ... @@
		-       "minSize": 10000,
		-       "usedExports": false,
		+       "minSize": 20000,
		+       "usedExports": true,
		@@ ... @@
		-     "usedExports": false,
		+     "usedExports": true,
		@@ ... @@
		-   "performance": false,
		+   "performance": Object {
		+     "hints": "warning",
		+     "maxAssetSize": 250000,
		+     "maxEntrypointSize": 250000,
		+   },
	`)
};
