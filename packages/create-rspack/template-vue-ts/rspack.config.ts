import {
	Configuration,
	HtmlRspackPlugin,
	DefinePlugin,
	RspackPluginFunction
} from "@rspack/core";
import { VueLoaderPlugin } from "vue-loader";

const isDev = process.env.NODE_ENV == "development";

const config: Configuration = {
	context: __dirname,
	entry: {
		main: "./src/main.ts"
	},
	resolve: {
		extensions: ["...", ".ts"]
	},
	plugins: [
		new VueLoaderPlugin() as RspackPluginFunction,
		new HtmlRspackPlugin({
			template: "./index.html"
		}),
		new DefinePlugin({
			__VUE_PROD_DEVTOOLS__: isDev
		})
	],
	module: {
		rules: [
			{
				test: /\.vue$/,
				loader: "vue-loader",
				options: {
					experimentalInlineMatchResource: true
				}
			},
			{
				test: /\.(js|ts)$/,
				use: [
					{
						loader: "builtin:swc-loader",
						options: {
							sourceMap: true,
							jsc: {
								parser: {
									syntax: "typescript",
									tsx: false
								}
							},
							env: {
								targets: [
									"chrome >= 87",
									"edge >= 88",
									"firefox >= 78",
									"safari >= 14"
								]
							}
						}
					}
				]
			},
			{
				test: /\.svg/,
				type: "asset/resource"
			}
		]
	}
};
module.exports = config;
