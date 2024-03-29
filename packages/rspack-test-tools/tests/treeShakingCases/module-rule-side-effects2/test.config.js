module.exports = {
	optimization: {
		sideEffects: true
	},
	builtins: {
		treeShaking: true,
		define: {
			"process.env.NODE_ENV": "'development'"
		}
	},
	module: {
		rules: [
			{
				test: /b.js$/,
				sideEffects: true
			}
		]
	}
};
