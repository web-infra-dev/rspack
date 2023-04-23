/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
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
