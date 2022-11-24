const path = require("path");
const resolve = filename => path.resolve(__dirname, filename);

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
				test: resolve("index.js"),
				use: [
					{
						loader: testLoader
					}
				]
			}
		]
	}
};
