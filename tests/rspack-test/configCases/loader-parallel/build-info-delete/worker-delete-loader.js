module.exports = function () {
	const { buildInfo } = this._module;
	delete buildInfo.dropMe;
	buildInfo.added = "added";
	return `module.exports = ${this.parallel}`;
};
