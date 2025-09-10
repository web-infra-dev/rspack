module.exports = function () {
	this._module.buildInfo.LOADER_ACCESS = true;
	this._module.buildMeta.LOADER_ACCESS = true;
	return (
		"module.exports = " +
		JSON.stringify({
			request: this._module.request,
			userRequest: this._module.userRequest,
			rawRequest: this._module.rawRequest,
		})
	);
};
