module.exports = function testLoader(content) {
	const head = `
	globalThis.mockFn = jest.fn();
	mockFn();
	`;
	this.callback(null, head + "\n" + content);
};
