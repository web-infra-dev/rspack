module.exports = {
	description: "should have module profile when profile is true",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/abc",
			profile: true
		};
	},
	async check(stats) {
		expect(
			stats?.toString({ all: false, modules: true }).replace(/\d+ ms/g, "X ms")
		).toMatchInlineSnapshot(`
		"./fixtures/a.js
		  X ms (resolving: X ms, integration: X ms, building: X ms)
		./fixtures/b.js
		  X ms (resolving: X ms, integration: X ms, building: X ms)
		./fixtures/c.js
		  X ms (resolving: X ms, integration: X ms, building: X ms)
		./fixtures/abc.js
		  X ms (resolving: X ms, integration: X ms, building: X ms)"
	`);
	}
};
