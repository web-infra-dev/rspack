var path = require("path");
// var LibManifestPlugin = require("../../../../").LibManifestPlugin;
const manifestPlugin = require("rspack-manifest-plugin").WebpackManifestPlugin;

/** @type {function(any, any): import("@rspack/core").Configuration} */
module.exports = (env, { testPath }) => {
	console.log(testPath)
return ({
	entry: {
		bundle0: ["./"],
	},
	plugins: [
		new manifestPlugin({
			fileName: path.resolve(testPath, "[name]-manifest.json"),
			generate: (seed, files, entries) => {
					console.log(files)
				const manifestFiles = files.reduce((manifest, file) => {
					manifest[file.name] = file.path;
					return manifest;
				}, seed);
				const entrypointFiles = Object.keys(entries).reduce(
					(previous, name) =>
						previous.concat(
							entries[name].filter((fileName) => !fileName.endsWith(".map")),
						),
					[],
				);
				return {
					files: manifestFiles,
					entrypoints: entrypointFiles,
				};
			},
		}),
		// new LibManifestPlugin({
		// 	path: path.resolve(testPath, "[name]-manifest.json"),
		// 	name: "[name]_[fullhash]"
		// })
	],
	stats: {
		all: false,
	},
	node: {
		__dirname: false,
	},
})
};
