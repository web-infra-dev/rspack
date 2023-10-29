const { stringifyRequest } = require("loader-utils");

module.exports = function () {};
module.exports.pitch = function (remainingRequest, precedingRequest, data) {
	return `import { lib } from ${stringifyRequest(this, `!!${remainingRequest}`)}
export const lib2 = "lib2";
export { lib }`;
};
