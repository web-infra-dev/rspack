/** @type {import('../../..').TDefaultsCaseConfig} */
module.exports = {
	description: "array defaults",
	options: () => ({
		output: {
			enabledChunkLoadingTypes: ["require", "..."],
			enabledWasmLoadingTypes: ["...", "async-node"]
		}
	}),
	diff: e =>
		e.toMatchInlineSnapshot(`
    - Expected
    + Received

    @@ ... @@
    +       "require",
    @@ ... @@
    +       "async-node",
  `)
};
