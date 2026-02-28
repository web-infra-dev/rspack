const path = require('path');

module.exports.pitch = function (request) {
	const callback = this.async();
	this.importModule(`${this.resourcePath}.webpack[javascript/auto]!=!!!${request}`, {}).then((exports) => {
		callback(null, `export default "${exports.default}"`);
	});

}
