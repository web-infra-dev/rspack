const { ModuleFederationPlugin } = require("@rspack/core").container;

module.exports = {
	context: __dirname,
	entry: {
		main: "./index.js"
	},
	optimization: {
		runtimeChunk: "single"
	},
	mode: "development",
	target: "node",
	output: {
		publicPath: "/",
		library: {
			type: "commonjs"
		},
		path: require("node:path").resolve(__dirname, "dist")
	},
	// optimization: {
	// 	runtimeChunk: "single"
	// },
	plugins: [
		new ModuleFederationPlugin({
			name: "host",
			remotes: {
				remoteApp: "remoteApp@http://localhost:3001/remoteEntry.js"
			},
			shared: ["./lib"]
		})
	]
};
