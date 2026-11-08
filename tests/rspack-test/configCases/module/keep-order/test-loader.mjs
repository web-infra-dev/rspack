export default function testLoader(content) {
	const head = `
	global.mockFn = rstest.fn();
	mockFn();
	`;
	this.callback(null, head + "\n" + content);
};
