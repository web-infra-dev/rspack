const assert = require("assert")
module.exports.pitch = function () {
	const callback = this.async();
	this.importModule(`${this.resourcePath}.webpack[javascript/auto]!=!!!./index.js`, {}, err => {
		assert.equal(err.message, 'Error: execute failed')
		callback(null, `export default "${err}"`);
	})
}
