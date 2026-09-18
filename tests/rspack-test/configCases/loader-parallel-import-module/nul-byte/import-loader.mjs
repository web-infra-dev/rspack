import path from "node:path";

export const pitch = function (request) {
	const callback = this.async();
	this.importModule(`${this.resourcePath}.webpack[javascript/auto]!=!!!${request}`, {}).then((exports) => {
		callback(null, `export default "${exports.default}"`);
	});

}
