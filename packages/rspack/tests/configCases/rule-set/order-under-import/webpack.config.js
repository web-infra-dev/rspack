async function testLoader(context) {
	const head = `
	globalThis.mockFn = jest.fn();
	mockFn();
	`;
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
				test: /index\.js/,
				use: [
					{
						loader: testLoader
					}
				]
			}
		]
	}
};
