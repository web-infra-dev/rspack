module.exports = {
	entry: {
		main: "./index.js"
	},
	builtins: {
		define: {
			"process.env.NODE_ENV": "'development'"
		}
	}
};
