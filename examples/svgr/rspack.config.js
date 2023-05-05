/** @type {import('@rspack/cli').Configuration} */
const config = {
	entry: {
		main: "./index.jsx"
	},
	module: {
		rules: [
			{
				test: /\.svg$/,
				use: ["@svgr/webpack", "url-loader"]
			}
		]
	},
	builtins: {
		html: [{ template: "./index.html" }]
	}
};
module.exports = config;
