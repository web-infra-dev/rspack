/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
export default {
	description: "empty config",
	options: () => ({}),
	diff: e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
};
