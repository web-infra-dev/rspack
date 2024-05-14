/** @type {import('../../..').TDefaultsCaseConfig} */
module.exports = {
	description: "empty config",
	options: () => ({}),
	diff: e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
};
