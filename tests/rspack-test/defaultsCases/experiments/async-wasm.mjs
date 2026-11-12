/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
export default {
	description: "async wasm",
	options: () => ({ experiments: { asyncWebAssembly: true } }),
	diff: e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
};
