module.exports = {
	stats: "errors-warnings",
	module: {
		rules: [
			{
				test: /.js/,
				type: "javascript/esm"
			}
		]
	}
};
