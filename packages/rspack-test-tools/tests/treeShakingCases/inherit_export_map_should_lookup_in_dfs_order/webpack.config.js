module.exports = {
	optimization: {
		sideEffects: false
	},
	builtins: {
		define: {
			"process.env.NODE_ENV": "'development'"
		}
	}
};
