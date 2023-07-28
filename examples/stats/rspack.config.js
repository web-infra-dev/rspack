class StatsPrinterTestPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("StatsPrinterTestPlugin", compilation => {
			compilation.hooks.statsPrinter.tap("StatsPrinterTestPlugin", stats => {
				stats.hooks.print
					.for("asset.emitted")
					.tap("StatsPrinterTestPlugin", (emitted, { red, formatFlag }) =>
						emitted ? red(formatFlag("emitted111")) : undefined
					);
				stats.hooks.print
					.for("asset.testA")
					.tap("StatsPrinterTestPlugin", (data, { red, formatFlag }) =>
						data ? red(formatFlag(`testA: ${data}`)) : undefined
					);
			});
			compilation.hooks.statsFactory.tap(
				"StatsFactoryTestPlugin",
				statsFactory => {
					statsFactory.hooks.extract
						.for("asset")
						.tap("StatsFactoryTestPlugin", (object, asset) => {
							object.testA = "aaaaaa";
						});

					statsFactory.hooks.sortResults
						.for("compilation.assets")
						.tap("StatsFactoryTestPlugin", comparators => {
							comparators.push((a1, a2) => (a1.size - a2.size > 0 ? 1 : -1));
						});
				}
			);
		});
	}
}

/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	mode: "development",
	entry: {
		main: "./src/index.js"
	},
	stats: true,
	plugins: [new StatsPrinterTestPlugin()],
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		]
	}
};
module.exports = config;
