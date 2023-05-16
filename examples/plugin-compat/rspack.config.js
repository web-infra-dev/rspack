const BundleAnalyzerPlugin =
	require("webpack-bundle-analyzer").BundleAnalyzerPlugin;
const CopyPlugin = require("copy-webpack-plugin");
const HtmlPlugin = require("@rspack/plugin-html").default;
const { StatsWriterPlugin } = require("webpack-stats-plugin");
const minifyPlugin = require("@rspack/plugin-minify");
const manifestPlugin = require("rspack-manifest-plugin").WebpackManifestPlugin;
const GeneratePackageJsonPlugin = require("generate-package-json-webpack-plugin");
const licensePlugin = require("license-webpack-plugin");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	target: "node",
	stats: { errors: true, warnings: true },
	entry: {
		main: "./src/index.js"
	},
	output: {
		filename: "[contenthash:8].js"
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
			template: "./index.ejs",
			templateParameters: (compilation, assets, assetTags, options) => {
				const cssFile = Object.keys(compilation.assets).filter(x =>
					x.endsWith(".css")
				)[0];
				return {
					inlineCss: compilation.assets[cssFile].source()
				};
			}
		}),
		new StatsWriterPlugin({
			stats: { all: true },
			filename: "stats.json"
		}),
		new GeneratePackageJsonPlugin(basePackage, {}),
		new licensePlugin.LicenseWebpackPlugin({
			stats: {
				warnings: false,
				errors: false
			},
			perChunkOutput: true,
			outputFilename: `3rdpartylicenses.txt`
		}),
		new manifestPlugin({
			fileName: "rspack-manifest.json",
			generate: (seed, files, entries) => {
				const manifestFiles = files.reduce((manifest, file) => {
					manifest[file.name] = file.path;
					return manifest;
				}, seed);
				const entrypointFiles = Object.keys(entries).reduce(
					(previous, name) =>
						previous.concat(
							entries[name].filter(fileName => !fileName.endsWith(".map"))
						),
					[]
				);
				return {
					files: manifestFiles,
					entrypoints: entrypointFiles
				};
			}
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
