module.exports = function testLoader(content) {
	const head = `
	global.mockFn = jest.fn();
	mockFn();
	`;
	this.callback(null, head + "\n" + content);
};
