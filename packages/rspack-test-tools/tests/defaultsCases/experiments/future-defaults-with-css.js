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
		-     "css": undefined,
		+     "css": false,
		+     "futureDefaults": true,
		@@ ... @@
		-     "hashDigestLength": 20,
		-     "hashFunction": "md4",
		+     "hashDigestLength": 16,
		+     "hashFunction": "xxhash64",
	`)
};
