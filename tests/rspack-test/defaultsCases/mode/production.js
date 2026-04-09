/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
module.exports = {
	description: "production",
	options: () => ({ mode: "production" }),
	diff: e =>
		e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			-   "mode": "none",
			+   "mode": "production",
			@@ ... @@
			-     "chunkIds": "natural",
			-     "concatenateModules": false,
			-     "emitOnErrors": true,
			-     "inlineExports": false,
			-     "innerGraph": false,
			-     "mangleExports": false,
			+     "chunkIds": "deterministic",
			+     "concatenateModules": true,
			+     "emitOnErrors": false,
			+     "inlineExports": true,
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
			-       "enforceSizeThreshold": 30000,
			-       "hidePathInfo": false,
			-       "maxAsyncRequests": Infinity,
			-       "maxInitialRequests": Infinity,
			+       "enforceSizeThreshold": 50000,
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
