module.exports = function () { };
module.exports.pitch = function (remainingRequest, precedingRequest, data) {
	return `import { lib } from ${JSON.stringify(this.utils.contextify(this.context, `!!${remainingRequest}`))}
export const lib2 = "lib2";
export { lib }`;
};
