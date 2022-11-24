const lessLoader = require("@rspack/plugin-less").default;

module.exports = {
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
