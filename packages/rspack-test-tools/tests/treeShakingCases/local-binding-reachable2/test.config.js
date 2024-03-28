module.exports = {
	optimization: {
		sideEffects: false
	},
	builtins: {
		treeShaking: true,
		define: {
			"process.env.NODE_ENV": "'development'"
		}
	}
};
