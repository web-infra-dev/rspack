module.exports = {
	mode: "development",
	entry: {
		main: "./index.js",
	},
	define: {
		"process.env.NODE_ENV": "'development'",
	},
	builtins: {
		html: [{}],
	},
};
