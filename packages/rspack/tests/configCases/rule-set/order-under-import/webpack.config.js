function testLoader(content) {
	const head = `
	globalThis.mockFn = jest.fn();
	mockFn();
	`;
	this.callback(null, head + "\n" + content);
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
