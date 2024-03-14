module.exports = {
	description: "ecmaVersion",
	options: () => ({ output: { ecmaVersion: 2020 } }),
	diff: e =>
		e.toMatchInlineSnapshot(`Compared values have no visual difference.`)
};
