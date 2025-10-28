const rspack = require("../../packages/rspack/dist/index.js");

module.exports = {
	context: __dirname,
	entry: {
		main: "./index.js"
	},
	mode: 'development',
	devtool: false,
	optimization: {
		runtimeChunk: 'single'
	},
	experiments: {
		mfAsyncStartup: true
	},
	plugins: [
		new rspack.container.ModuleFederationPlugin({
			name: "basic_example",
			filename: "remoteEntry.js",
			shared: {
				react: {
					singleton: true,
					requiredVersion: "^18.0.0"
				}
			}
		})
	]
};
