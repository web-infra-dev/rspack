/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	context: __dirname,
	mode: "development",
	entry: {
		main: "./src/index.js"
	},
	builtins: {
		html: [{}]
	},
	module: {
		rules: [
			{
				test: /\.js$/,
				use: ["./pre-loader.js"],
				enforce: "pre"
			},
			{
				test: /\.js$/,
				use: ["./pre-loader2.js"],
				enforce: "pre"
			},
			{
				test: /\.js$/,
				use: ["./normal-loader.js", "./normal-loader2.js"]
			},
			{
				test: /\.js$/,
				use: ["./post-loader.js", "./post-loader2.js"],
				enforce: "post"
			}
		]
	}
};
