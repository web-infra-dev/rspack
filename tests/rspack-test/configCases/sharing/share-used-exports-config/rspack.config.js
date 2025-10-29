const { container } = require("@rspack/core");

const { ModuleFederationPlugin } = container;

process.env.MF_CUSTOM_REFERENCED_EXPORTS = JSON.stringify({
  react: ["useMemo"]
});

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		chunkIds: "named",
		moduleIds: "named"
	},
	output: {
		chunkFilename: "[id].js"
	},
	plugins: [
		new ModuleFederationPlugin({
			name: "container",
			filename: "container.[chunkhash:8].js",
			library: { type: "commonjs-module" },
			exposes: {
				"./entry": "./src/entry.js"
			},
			remoteType: "script",
			remotes: {
				"@remote/alias": "remote@http://localhost:8000/remoteEntry.js"
			},
			shared: {
        'ui-lib': {
          treeshake: true,
          // usedExports: ["default", "version"]
        }
      }
    })
  ]
};
