module.exports = {
	entry: {
		main: ["./index.js"]
	},
	builtins: {
		define: {
			TRUE: true
		}
	},
	experiments: {
		rspackFuture: {
			disableTransformByDefault: true
		}
	}
};
