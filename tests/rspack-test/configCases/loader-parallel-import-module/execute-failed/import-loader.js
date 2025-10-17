const assert = require("assert")
module.exports.pitch = function () {
	const callback = this.async();
	this.importModule(`${this.resourcePath}.webpack[javascript/auto]!=!!!./index.js`, {}).then((_exports) => {
		throw new Error("This should not be executed");
	}).catch((err) => {
		assert.match(err.message, /execute failed/)
		callback(null, `export default "${err}"`);
	})
}
