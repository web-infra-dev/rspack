const lessLoader = require("@rspack/plugin-less").default;

module.exports = {
	devtool: "source-map",
	module: {
		rules: [
			{
				test: /\.less$/,
				use: [{ loader: lessLoader }],
				type: "css"
			}
		]
	}
};
