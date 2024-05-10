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
		treeShaking: true,
		define: {
			"process.env.NODE_ENV": "'development'"
		}
	}
};
