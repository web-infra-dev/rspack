module.exports = {
	optimization: {
		sideEffects: true
	},
	module: {
		rules: [
			{
				test: /\.svg$/,
				type: "asset/inline"
			}
		]
	},
	builtins: {
		define: {
			"process.env.NODE_ENV": "'development'"
		}
	}
};
