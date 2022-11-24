const lessLoader = require("@rspack/plugin-less").default;

module.exports = {
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
