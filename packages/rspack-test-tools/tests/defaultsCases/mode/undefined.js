/** @type {import('../../..').TDefaultsCaseConfig} */
module.exports = {
	description: "no mode provided",
	options: () => ({ mode: undefined }),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "mode": "none",
		+   "mode": undefined,
		@@ ... @@
		-     "chunkIds": "natural",
		-     "concatenateModules": false,
		-     "innerGraph": false,
		-     "mangleExports": false,
		+     "chunkIds": "deterministic",
		+     "concatenateModules": true,
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
