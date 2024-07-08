/** @type {import('../../..').TDefaultsCaseConfig} */
module.exports = {
	description: "records",
	options: () => ({ recordsPath: "some-path" }),
	diff: e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
};
