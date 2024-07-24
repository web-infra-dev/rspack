const path = require('path');

module.exports.pitch = function (request) {
	const callback = this.async();
	this.importModule(`${this.resourcePath}.webpack[javascript/auto]!=!!!${request}`, {}).then((exports) => {
		console.log(exports.default);
		callback(null, `export default "${exports.default}"`);
	});

}
