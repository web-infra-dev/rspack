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
		-   "mode": "none",
		+   "mode": "development",
		@@ ... @@
		-     "nodeEnv": false,
		+     "nodeEnv": "development",
		@@ ... @@
		-     "pathinfo": false,
		+     "pathinfo": true,
		@@ ... @@
		-           "production",
		+           "development",
		@@ ... @@
		-       "production",
		+       "development",
	`)
};
