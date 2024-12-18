/** @type {import('../../dist').TStatsAPICaseConfig} */
module.exports = {
	description: "should output error chunk info",
	options(context) {
		return {
			context: context.getSource(),
			entry: {
				a: {
					import: "./fixtures/a",
					dependOn: "b"
				},
				b: {
					import: "./fixtures/b",
					dependOn: "a"
				}
			}
		};
	},
	async check(stats) {
		expect(
			stats?.toJson({
				errors: true
			}).errors
		).toMatchInlineSnapshot(`
		Array [
		  Object {
		  "chunkId": "74",
		  "chunkInitial": true,
		  "chunkName": "a",
		  "details": undefined,
		  "message": "  × Entrypoints 'a' and 'b' use 'dependOn' to depend on each other in a circular way.\\n",
		  "moduleTrace": Array [],
		  "stack": undefined,
		},
		]
	`);
	}
};
