/** @type {import('@rspack/cli').Configuration} */
const config = {
	mode: "development",
	entry: {
		main: ["./src/index.jsx"]
	},
	module: {
		rules: [
			{
				test: /\.s[ac]ss$/,
				use: ["sass-loader"],
				type: "css"
			}
		]
	},
	builtins: {
		html: [
			{
				template: "index.html"
			}
		]
	}
};
module.exports = config;
