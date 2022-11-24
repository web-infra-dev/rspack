const path = require("path");
const resolve = filename => path.resolve(__dirname, filename);

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
				test: resolve("index.js"),
				uses: [
					{
						loader: testLoader
					}
				]
			}
		]
	}
};
