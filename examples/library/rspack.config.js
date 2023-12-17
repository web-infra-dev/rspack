const noNameLibraryTypes = ["amd-require", "module"]

/** @type {import('@rspack/cli').Configuration} */
const config = [
	// "var",
	// "module",
	// "assign",
	// "assign-properties",
	// "this",
	// "window",
	// "self",
	// "global",
	// "commonjs",
	// "commonjs2",
	// "commonjs-module",
	// "commonjs-static",
	// "amd",
	// "amd-require",
	"umd",
	// "umd2"
].map(type => ({
	context: __dirname,
	mode: "development",
	devtool: false,
	entry: {
		main: "./src/index.js"
	},
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		}
	},
	optimization: {
		usedExports: true,
		sideEffects: true
	},
	builtins: {
		html: [{}]
	},
	output: {
		filename: `${type}.js`,
		library: {
			name: noNameLibraryTypes.includes(type) ? undefined : "MyLibrary",
			type: type
		}
	}
}));
module.exports = config;
