/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = [
	"var",
	"module",
	"assign",
	"assign-properties",
	"this",
	"window",
	"self",
	"global",
	"commonjs",
	"commonjs2",
	"commonjs-module",
	"commonjs-static",
	"amd",
	"amd-require",
	"umd",
	"umd2"
].map(type => ({
	context: __dirname,
	mode: "development",
	devtool: false,
	entry: {
		main: "./src/index.js"
	},
	output: {
		filename: `${type}.js`,
		library: {
			name: "MyLibrary",
			type: type
		}
	}
}));
