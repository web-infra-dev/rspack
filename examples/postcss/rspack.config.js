module.exports = {
	mode: "development",
	entry: "./index.js",
	builtins: {
		html: [{}],
		define: {
			"process.env.NODE_ENV": "'development'"
		}
	}
};
