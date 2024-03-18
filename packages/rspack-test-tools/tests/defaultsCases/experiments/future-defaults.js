module.exports = {
	description: "experiments.futureDefaults",
	options: () => ({
		experiments: {
			futureDefaults: true
		}
	}),
	diff: e =>
		e.toMatchInlineSnapshot(`
    - Expected
    + Received

    @@ ... @@
    +     "futureDefaults": true,
    @@ ... @@
    -     "hashDigestLength": 20,
    -     "hashFunction": "md4",
    +     "hashDigestLength": 16,
    +     "hashFunction": "xxhash64",
  `)
};
