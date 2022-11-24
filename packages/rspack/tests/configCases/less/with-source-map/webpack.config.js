const lessLoader = require("@rspack/plugin-less").default;

module.exports = {
	devtool: "source-map",
	module: {
		rules: [
			{
				test: /\.less$/,
				uses: [{ loader: lessLoader }],
				type: "css"
			}
		]
	}
};
