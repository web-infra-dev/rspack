module.exports = {
	description: "cache filesystem development",
	options: () => ({ mode: "development", cache: { type: "filesystem" } }),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "cache": false,
		+   "cache": Object {
		+     "type": "filesystem",
		+   },
		@@ ... @@
		-   "mode": "none",
		+   "mode": "development",
		@@ ... @@
		-     "nodeEnv": false,
		+     "nodeEnv": "development",
		@@ ... @@
		-           "production",
		+           "development",
		@@ ... @@
		-       "production",
		+       "development",
	`)
};
