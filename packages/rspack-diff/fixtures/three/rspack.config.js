/**
 * @type {import('webpack').Configuration}
 */
module.exports = {
	mode: "development",
	devtool: false,
	entry: {
		main: "./index.js"
	},
	target: ["web", "es5"],
	optimization: {
		minimize: false,
		concatenateModules: false
	},
	output: {
		path: "rspack-dist"
	}
};
