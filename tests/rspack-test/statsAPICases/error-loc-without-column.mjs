/** @type {import('@rspack/test-tools').TStatsAPICaseConfig} */
export default {
	description: "should default missing JS error columns to zero",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/a",
			plugins: [
				compiler => {
					compiler.hooks.thisCompilation.tap("test", compilation => {
						compilation.hooks.processAssets.tap("test", () => {
							const lineOnly = new Error("line only");
							lineOnly.loc = { start: { line: 7 } };
							const lineRange = new Error("line range");
							lineRange.loc = { start: { line: 7 }, end: { line: 9 } };
							const zeroColumn = new Error("zero column");
							zeroColumn.loc = { start: { line: 7, column: 0 } };
							const withColumn = new Error("with column");
							withColumn.loc = {
								start: { line: 2, column: 3 },
								end: { line: 2, column: 8 }
							};
							compilation.errors.push(lineOnly, lineRange, zeroColumn, withColumn);
						});
					});
				}
			]
		};
	},
	async check(stats) {
		const locs = Object.fromEntries(
			stats
				.toJson({ all: false, errors: true })
				.errors.map(error => [
					error.message.match(/line only|line range|zero column|with column/)[0],
					error.loc
				])
		);
		expect(locs).toEqual({
			"line only": "7:0",
			"line range": "7:0-9:0",
			"zero column": "7:0",
			"with column": "2:3-8"
		});
		expect(stats.compilation.errors.map(error => error.loc)).toEqual([
			{ start: { line: 7, column: 0 } },
			{ start: { line: 7, column: 0 }, end: { line: 9, column: 0 } },
			{ start: { line: 7, column: 0 } },
			{ start: { line: 2, column: 3 }, end: { line: 2, column: 8 } }
		]);
	}
};
