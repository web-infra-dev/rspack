module.exports = {
	target: ["web", "es5"],
	optimization: {
		sideEffects: true
	},
	builtins: {
		treeShaking: true,
		define: {
			"process.env.NODE_ENV": "'development'"
		}
	}
};
