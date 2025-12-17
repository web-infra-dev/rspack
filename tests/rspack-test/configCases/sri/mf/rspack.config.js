const { SubresourceIntegrityPlugin, container } = require("@rspack/core");

module.exports = {
	target: "web",
	optimization: {
		moduleIds: "named"
	},
	plugins: [
		new SubresourceIntegrityPlugin(),
		new container.ModuleFederationPlugin({
			name: "app",
			filename: "remoteEntry.js",
			exposes: {
				"./render": "./render.js"
			},
			shared: {
				react: {
					singleton: true,
					requiredVersion: "^19.0.0"
				},
				"react-dom": {
					singleton: true,
					requiredVersion: "^19.0.0"
				}
			}
		})
	],
	output: {
		crossOriginLoading: "anonymous"
	}
};
