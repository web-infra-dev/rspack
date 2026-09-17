module.exports = function () {
	const { buildInfo } = this._module;
	delete buildInfo.onlyKey;
	return `module.exports = ${this.parallel}`;
};
