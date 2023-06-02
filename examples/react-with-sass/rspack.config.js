/** @type {import('@rspack/cli').Configuration} */
const config = {
	mode: "development",
	entry: {
		main: ["./src/index.jsx"]
	},
	output: {
		publicPath: "http://localhost:3000"
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
		html: [{}]
	}
};
module.exports = config;
