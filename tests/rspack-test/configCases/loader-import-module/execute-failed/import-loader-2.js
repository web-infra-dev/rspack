module.exports.pitch = function () {
	const callback = this.async();
	this.importModule(`${this.resourcePath}.webpack[javascript/auto]!=!!!./index.js`, {}, err=> {
		expect(err.message).toBe('Error: execute failed')
		callback(null, `export default "${err}"`);
	})
}
