class StatsPrinterTestPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("StatsPrinterTestPlugin", compilation => {
			compilation.hooks.statsPrinter.tap("StatsPrinterTestPlugin", stats => {
				stats.hooks.print
					.for("asset.emitted")
					.tap("StatsPrinterTestPlugin", (emitted, { red, formatFlag }) =>
						emitted ? red(formatFlag("emitted111")) : undefined
					);
			});
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
