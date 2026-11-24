/** @type {import('@rspack/test-tools').TStatsAPICaseConfig} */
export default {
	description: "should respect stats.colors for precolored errors and warnings",
	options(context) {
		return { context: context.getSource(), entry: "./fixtures/a", mode: "development" };
	},
	check(stats) {
		const colored = "value: \x1b[33m42\x1b[39m.";
		stats.compilation.errors.push(new Error(colored));
		stats.compilation.warnings.push(new Error(colored));

		for (const colors of [false, true]) {
			const options = { all: false, errors: true, warnings: true, colors };
			const { errors, warnings } = stats.toJson(options);
			const output = stats.toString(options);
			for (const message of [errors[0].message, warnings[0].message, output]) {
				expect(message).toContain(colors ? colored : "value: 42.");
				if (!colors) expect(message).not.toContain("\x1b[");
			}
		}
	}
};
