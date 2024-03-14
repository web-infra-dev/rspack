module.exports = {
	description: "stats string",
	options: () => ({ stats: "minimal" }),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "stats": Object {},
		+   "stats": Object {
		+     "preset": "minimal",
		+   },
	`)
};
