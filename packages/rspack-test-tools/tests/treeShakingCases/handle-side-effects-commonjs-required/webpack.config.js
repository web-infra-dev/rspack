module.exports = {
	optimization: {
		sideEffects: true
	},
	builtins: {
		define: {
			"process.env.NODE_ENV": "'development'"
		}
	},
	target: ["web", "es5"]
};
