async function testLoader(context) {
	const head = "const head = 'head';";
	const code = context.source.getCode();
	return {
		content: head + "\n" + code,
		meta: "",
		sourceMap: "{}"
	};
}

module.exports = {
	entry: {
		main: "./index.js"
	},
	module: {
		rules: [
			{
				test: ".js$",
				uses: [
					{
						loader: testLoader
					}
				]
			}
		]
	}
};
