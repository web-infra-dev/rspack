module.exports = {
	target: ["web", "es5"],
	optimization: {
		sideEffects: true
	},
	builtins: {
		define: {
			"process.env.NODE_ENV": "'development'"
		}
	}
};
