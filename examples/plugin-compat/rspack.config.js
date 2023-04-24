const BundleAnalyzerPlugin =
	require("webpack-bundle-analyzer").BundleAnalyzerPlugin;
const CopyPlugin = require("copy-webpack-plugin");
const HtmlPlugin = require("@rspack/plugin-html").default;
const { StatsWriterPlugin } = require("webpack-stats-plugin");
/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	target: "node",
	mode: "development",
	stats: { all: true },
	entry: {
		main: "./src/index.js"
	},
	builtins: {
		minify: false
	},
	plugins: [
		new BundleAnalyzerPlugin({
			openAnalyzer: false,
			analyzerMode: "json"
		}),
		new CopyPlugin([
			{
				from: "public",
				dist: "."
			}
		]),
		new HtmlPlugin({
			template: "./index.html"
		}),
		new StatsWriterPlugin({
			stats: { all: true },
			filename: "stats.json"
		})
	]
};
