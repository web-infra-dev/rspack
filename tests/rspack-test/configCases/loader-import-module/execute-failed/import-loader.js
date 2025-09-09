module.exports.pitch = function () {
	const callback = this.async();
	this.importModule(`${this.resourcePath}.webpack[javascript/auto]!=!!!./index.js`, {}).then((_exports) => {
		throw new Error("This should not be executed");
	}).catch((err) => {
		expect(err.message).toBe('Error: execute failed')
		// expect(err).toBe('execute failed')
		callback(null, `export default "${err}"`);
	})
}
