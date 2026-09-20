/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
export default {
	description: "records",
	options: () => ({ recordsPath: "some-path" }),
	diff: e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
};
