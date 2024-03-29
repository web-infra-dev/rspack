module.exports = {
	entry: {
		main: {
			import: ["./index.js"]
		}
	},
	builtins: {
		treeShaking: true,
		define: {
			"process.env.NODE_ENV": "'development'"
		}
	},
	optimization: {
		sideEffects: "flag"
	}
};
