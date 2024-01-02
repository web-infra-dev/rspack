const rspack = require("@rspack/core");
const ReactRefreshPlugin = require("@rspack/plugin-react-refresh");

const isProduction = process.env.NODE_ENV === "production";

/** @type {import('@rspack/cli').Configuration} */
const config = {
	mode: isProduction ? "production" : "development",
	entry: { main: "./src/index.tsx" },
	devtool: false,
	optimization: {
		minimize: false // Disabling minification because it takes too long on CI
	},
	module: {
		rules: [
			{
				test: /\.tsx$/,
				use: {
					loader: "babel-loader",
					options: {
						presets: [
							["@babel/preset-react", { runtime: "automatic" }],
							"@babel/preset-typescript"
						],
						plugins: [
							!isProduction && require.resolve("react-refresh/babel")
						].filter(Boolean)
					}
				}
			}
		]
	},
	plugins: [
		new rspack.HtmlRspackPlugin({ template: "./index.html" }),
		new rspack.DefinePlugin({ "process.env.NODE_ENV": "'development'" }),
		!isProduction && new ReactRefreshPlugin()
	].filter(Boolean)
};

module.exports = config;
