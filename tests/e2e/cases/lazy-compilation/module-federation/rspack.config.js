const { rspack } = require("@rspack/core");
const ReactRefreshPlugin = require("@rspack/plugin-react-refresh");

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
	context: __dirname,
	entry: "./src/index.jsx",
	mode: "development",
	devtool:false,
	resolve: {
		extensions: ["...", ".jsx"]
	},
	module: {
		rules: [
			{
				test: /\.(jsx?|tsx?)$/,
				use: [
					{
						loader: "builtin:swc-loader",
						options: {
							jsc: {
								parser: {
									syntax: "typescript",
									tsx: true
								},
								transform: {
									react: {
										runtime: "automatic",
										development: true,
										refresh: true,
									}
								},
							},
						}
					},
				]
			}
		]
	},
	plugins: [
		new rspack.HtmlRspackPlugin({ template: "./src/index.html" }),
		new rspack.container.ModuleFederationPlugin({
			name:"host",
			remotes: {
				remote: "remote@http://localhost:5679/remoteEntry.js"
			},
			// prevent init remote entry
			shareStrategy: 'loaded-first',
			shared: {
				react: {},
				'react-dom': {}
			},
			runtimePlugins: [require.resolve('./runtime-plugin.js')]
		}),
		new ReactRefreshPlugin(),
	],
	lazyCompilation:true,
	devServer: {
		hot: true,
		port: 5678,
		devMiddleware: {
			writeToDisk: true
		}
	}
};
