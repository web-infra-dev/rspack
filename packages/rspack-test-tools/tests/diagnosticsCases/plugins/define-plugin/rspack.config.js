const rspack = require("@rspack/core")

module.exports = {
	optimization: {
		nodeEnv: "development"
	},
	plugins: [
		new rspack.DefinePlugin({
			"process.env.NODE_ENV": JSON.stringify("production")
		})
	]
}
