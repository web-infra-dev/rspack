/** @type {import('../../..').TDefaultsCaseConfig} */
module.exports = {
	description: "none mode",
	options: () => ({ mode: "none" }),
	diff: e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
};
