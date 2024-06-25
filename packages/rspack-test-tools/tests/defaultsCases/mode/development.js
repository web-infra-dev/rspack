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
		-   "mode": "none",
		+   "mode": "development",
		@@ ... @@
		-     "nodeEnv": false,
		+     "nodeEnv": "development",
		@@ ... @@
		-     "pathinfo": false,
		+     "pathinfo": true,
		@@ ... @@
		-       "production",
		+       "development",
	`)
};
