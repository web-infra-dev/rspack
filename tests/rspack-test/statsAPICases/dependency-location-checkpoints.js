const { source } = require("../fixtures/dependency-location-loader");

/** @type {import('@rspack/test-tools').TStatsAPICaseConfig} */
module.exports = {
	description: "should preserve UTF-16 dependency locations across source checkpoints",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/a",
			optimization: { minimize: false, concatenateModules: false },
			module: {
				rules: [{
					test: /[/\\]fixtures[/\\]a\.js$/,
					loader: require.resolve("../fixtures/dependency-location-loader")
				}]
			}
		};
	},
	async check(stats) {
		expect(stats.hasErrors()).toBe(false);
		const { modules } = stats.toJson({ all: false, modules: true, reasons: true });
		const imported = modules.find(module => module.name === "./fixtures/b.js");
		const actual = imported.reasons
			.filter(reason => reason.type === "cjs require")
			.map(reason => reason.loc)
			.sort();
		const expected = [...source.matchAll(/"\.\/b"/g)].map(match => {
			// JavaScript string lengths count UTF-16 units independently of Rust byte offsets.
			const lines = source.slice(0, match.index).split("\n");
			const column = lines.at(-1).length + 1;
			return `${lines.length}:${column}-${column + match[0].length}`;
		}).sort();
		expect(actual).toEqual(expected);
	}
};
