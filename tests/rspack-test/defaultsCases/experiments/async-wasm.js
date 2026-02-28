/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
module.exports = {
	description: "async wasm",
	options: () => ({ experiments: { asyncWebAssembly: true } }),
	diff: e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
};
