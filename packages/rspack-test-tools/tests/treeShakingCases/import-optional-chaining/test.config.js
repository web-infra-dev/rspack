module.exports = {
	optimization: {
		sideEffects: true
	},
	builtins: {
		treeShaking: true,
		define: {
			"process.env.NODE_ENV": "'production'"
		}
	}
};
