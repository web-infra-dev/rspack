module.exports = {
	mode: "development",
	entry: {
		main: {
			import: ["./index.js"]
		}
	},
	builtins: {
		html: [{}],
		define: {
			"process.env.NODE_ENV": "'development'"
		}
	}
};
