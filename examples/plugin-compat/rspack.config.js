const BundleAnalyzerPlugin =
	require("webpack-bundle-analyzer").BundleAnalyzerPlugin;
const CopyPlugin = require("copy-webpack-plugin");
const HtmlPlugin = require("@rspack/plugin-html").default;
const { StatsWriterPlugin } = require("webpack-stats-plugin");
const minifyPlugin = require("@rspack/plugin-minify");
const GeneratePackageJsonPlugin = require("generate-package-json-webpack-plugin");
const GeneratePackageJsonPlugin = require('generate-package-json-webpack-plugin')
const GeneratePackageJsonPlugin = require('generate-package-json-webpack-plugin');
const licensePlugin = require('license-webpack-plugin');
/** @type {import('@rspack/cli').Configuration} */
const config = {
	target: "node",
	mode: "development",
	stats: { all: true },
	entry: {
		main: "./src/index.js"
	},
	optimization: {
		minimize: true,
		minimizer: [
			new minifyPlugin({
				minifier: "terser",
				target: "es6",
				css: true
			})
		]
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
		}),
		new GeneratePackageJsonPlugin(basePackage, {}),
		new licensePlugin.LicenseWebpackPlugin({
			stats: {
				warnings: false,
				errors: false,
			  },
			perChunkOutput: true,
			outputFilename: `3rdpartylicenses.txt`,
		})
	]
};
module.exports = config;

var basePackage = {
	name: "my-nodejs-module",
	version: "1.0.0",
	main: "./bundle.js",
	engines: {
		node: ">= 14"
	}
};
