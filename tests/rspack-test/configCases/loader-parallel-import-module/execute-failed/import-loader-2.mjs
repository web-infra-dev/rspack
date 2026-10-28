import assert from "node:assert";
export const pitch = function () {
	const callback = this.async();
	this.importModule(`${this.resourcePath}.webpack[javascript/auto]!=!!!./index.js`, {}, err => {
		assert.equal(err.message, 'Error: execute failed')
		callback(null, `export default "${err}"`);
	})
}
