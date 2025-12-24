// @ts-nocheck
module.exports = function () {
	return `require(${JSON.stringify(this.resourcePath)})`;
};
