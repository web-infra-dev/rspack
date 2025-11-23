// import refreshPlugin from '@rspack/plugin-react-refresh';
import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "@rspack/cli";
import { rspack } from "@rspack/core";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const _isDev = process.env.NODE_ENV === "development";
const _isProd = process.env.NODE_ENV === "production";

export default defineConfig({
	context: __dirname,
	mode: "development",
	devtool: false,
	entry: "./src/index.js",
	target: "web",
	resolve: {
		extensions: [".js", ".jsx", ".ts", ".tsx", ".json"]
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
									syntax: "ecmascript",
									jsx: true
								},
								transform: {
									react: {
										development: false,
										refresh: false
									}
								}
							}
						}
					}
				]
			},
			{
				test: /\.(png|jpg|jpeg|gif|svg)$/,
				type: "asset/resource"
			}
		]
	},
	optimization: {
		minimize: false,
		splitChunks: false,
		runtimeChunk: false
	},
	plugins: [
		new rspack.container.ModuleFederationPlugin({
			name: "host",
			remotes: {
				remote: "remote@http://localhost:3002/remoteEntry.js"
			},
			shared: {
				react: {
					singleton: true,
					requiredVersion: "^18.3.1",
					eager: false
				},
				"react-dom": {
					singleton: true,
					requiredVersion: "^18.3.1",
					eager: false
				},
				"react-router-dom": {
					singleton: true,
					requiredVersion: "^7.1.1",
					eager: false
				},
				antd: {
					singleton: true,
					requiredVersion: "^5.21.8",
					eager: false
				},
				"@ant-design/icons": {
					singleton: true,
					requiredVersion: "^5.5.2",
					eager: false
				},
				"@reduxjs/toolkit": {
					singleton: true,
					requiredVersion: "^2.5.0",
					eager: false
				},
				"react-redux": {
					singleton: true,
					requiredVersion: "^9.2.0",
					eager: false
				},
				"lodash-es": {
					singleton: true,
					requiredVersion: "^4.17.21",
					eager: false
				},
				"chart.js": {
					singleton: true,
					requiredVersion: "^4.4.7",
					eager: false
				},
				"react-chartjs-2": {
					singleton: true,
					requiredVersion: "^5.2.0",
					eager: false
				},
				dayjs: {
					singleton: true,
					requiredVersion: "^1.11.13",
					eager: false
				}
			}
		}),
		new rspack.HtmlRspackPlugin({
			template: "./src/index.html"
		})
		// isDev && new refreshPlugin() // Disabled
	].filter(Boolean),
	output: {
		path: path.resolve(__dirname, "dist"),
		publicPath: "http://localhost:3001/",
		clean: true
	},
	devServer: {
		port: 3001,
		hot: false,
		historyApiFallback: true,
		devMiddleware: {
			writeToDisk: true
		},
		headers: {
			"Access-Control-Allow-Origin": "*"
		}
	}
});
